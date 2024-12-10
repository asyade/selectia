#![allow(unused_imports)]

use eyre::Result;
use futures::StreamExt;
use std::hash::Hash;
use std::io::stdout;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::instrument;

#[cfg(target_os = "linux")]
const MAMBA_URL: &str = "https://micro.mamba.org/api/micromamba/linux-64/latest.json";

#[cfg(target_os = "macos")]
const MAMBA_URL: &str = "https://micro.mamba.org/api/micromamba/osx-64/latest.json";

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
const MAMBA_URL: &str =
    "https://github.com/mamba-org/micromamba-releases/releases/download/2.0.4-0/micromamba-win-64";

const MAMBA_VERSION: &str = "2.0.4";

#[derive(Debug, Clone)]
pub struct Environment {
    pub data_path: PathBuf,
    pub extra_environ: HashMap<String, String>,
}

impl Environment {
    pub fn new(data_path: PathBuf) -> Self {
        Self {
            data_path,
            extra_environ: HashMap::new(),
        }
    }

    pub async fn load(&self, environment_name: &str) -> Result<()> {
        self.solve_mamba().await?;
        self.activate_env(environment_name).await?;
        Ok(())
    }

    pub async fn install(&self, environment: &Path) -> Result<()> {
        self.solve_mamba().await?;
        self.create_env_from_file(environment).await?;
        Ok(())
    }

    async fn solve_mamba(&self) -> Result<()> {
        let mamba_binary_path = self.mamba_binary_path();

        if !mamba_binary_path.exists() || self.mamba_binary().version().await? != MAMBA_VERSION {
            tracing::warn!(
                "Mamba binary not found at {}, solving...",
                mamba_binary_path.display()
            );
            tokio::fs::create_dir_all(mamba_binary_path.parent().unwrap()).await?;

            let response = reqwest::get(MAMBA_URL).await?;
            let payload = response.bytes().await?;
            let _ = tokio::fs::remove_file(&mamba_binary_path).await;
            let mut mamba_file = tokio::fs::File::create(mamba_binary_path).await?;

            tracing::info!("Downloading mamba binary...");
            mamba_file.write_all(&payload).await?;
            tracing::info!("Mamba binary downloaded");
        } else {
            tracing::info!("Mamba binary already exists");
        }

        if !self.mamba_root_path().exists() {
            tokio::fs::create_dir_all(self.mamba_root_path()).await?;
        }
        self.mamba_binary().init().await?;
        Ok(())
    }

    pub async fn cmd(&self, command: &str) -> Result<ActivatedCommand> {
        Ok(ActivatedCommand::new(
            self.mamba_binary_path(),
            self.mamba_root_path(),
            self.extra_environ.clone(),
            command,
        ))
    }

    pub async fn create_env_from_file(&self, env_file: &Path) -> Result<()> {
        self.cmd("mamba")
            .await?
            .arg("-y")
            .arg("env")
            .arg("create")
            .arg("-f")
            .arg(env_file.to_str().unwrap())
            .is_success()
            .await
    }

    pub async fn activate_env(&self, env_name: &str) -> Result<()> {
        self.cmd("mamba")
            .await?
            .arg("activate")
            .arg(env_name)
            .is_success()
            .await
    }

    pub async fn run_script_within_env(
        &self,
        env_name: &str,
        script: &str,
    ) -> Result<ActivatedCommand> {
        let mut cmd: ActivatedCommand = self.cmd("mamba")
            .await?;
            cmd.arg("activate")
            .arg(env_name)
            .arg("&&")
            .arg("python")
            .arg(script);
        Ok(cmd)
    }

    fn mamba_binary(&self) -> MambaBinary {
        MambaBinary {
            environment: self.extra_environ.clone(),
            root_path: self.mamba_root_path(),
            path: self.mamba_binary_path(),
        }
    }

    fn mamba_root_path(&self) -> PathBuf {
        self.data_path.join("mamba")
    }

    fn mamba_binary_path(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        let path = self.data_path.join("bin").join("mamba.exe");

        #[cfg(not(target_os = "windows"))]
        let path = self.data_path.join("bin").join("mamba");

        path
    }
}

#[derive(Debug)]
struct MambaBinary {
    environment: HashMap<String, String>,
    root_path: PathBuf,
    path: PathBuf,
}

impl MambaBinary {
    pub async fn version(&self) -> Result<String> {
        let output = self
            .base_command()
            .await
            .arg("--version")
            .raw_stdout()
            .await?;
        Ok(output.trim().to_string())
    }

    #[cfg(target_os = "windows")]
    #[instrument]
    pub async fn init(&self) -> Result<()> {
        self.base_command()
            .await
            .arg("-y")
            .arg("shell")
            .arg("init")
            .arg("-s")
            .arg("cmd.exe")
            .raw_stdout()
            .await?;
        Ok(())
    }

    async fn base_command(&self) -> MambaCommand {
        let mut cmd = tokio::process::Command::new(&self.path);
        cmd.envs(self.environment.iter());

        cmd.arg("--root-prefix").arg(&self.root_path);
        MambaCommand { command: cmd }
    }
}

#[derive(Debug)]
struct MambaCommand {
    command: tokio::process::Command,
}

impl MambaCommand {
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.command.arg(arg);
        self
    }

    pub async fn run(&mut self) -> Result<CommandResult> {
        let output = self.command.output().await?;
        Ok(CommandResult {
            stdout: String::from_utf8(output.stdout)?,
            stderr: String::from_utf8(output.stderr)?,
            status: output.status,
        })
    }

    #[instrument]
    pub async fn raw_stdout(&mut self) -> Result<String> {
        tracing::info!("spawning command");
        let result = self.run().await?;
        if !result.stderr.is_empty() {
            tracing::error!(fd = "stderr", "{}", result.stderr);
        }
        if !result.status.success() {
            tracing::error!(
                "Output ignored as the command status is not success: {}",
                result.stdout
            );
            return Err(eyre::eyre!("Command failed"));
        }
        Ok(result.stdout)
    }
}

pub struct CommandResult {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus,
}

#[derive(Debug)]
pub struct ActivatedCommand {
    mamba_root_path: PathBuf,
    environment: HashMap<String, String>,
    args: Vec<String>,
}

impl ActivatedCommand {
    pub fn new(
        _mamba_path: PathBuf,
        mamba_root_path: PathBuf,
        environment: HashMap<String, String>,
        command_name: &str,
    ) -> Self {
        Self {
            mamba_root_path,
            environment,
            args: vec![command_name.to_string()],
        }
    }

    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_string());
        self
    }

    fn format_command(&self) -> String {
        let mamba_hook =
            std::path::absolute(&self.mamba_root_path.join("Scripts").join("activate.bat"))
                .unwrap();
        let activate_script = format!("call {}", mamba_hook.display());
        let args = self
            .args
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<_>>();
        format!("{} && {}", activate_script, args.join(" "))
    }

    pub fn env(&mut self, key: &str, value: &str) -> &mut Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }

    pub async fn run(&self) -> Result<CommandResult> {
        let sub_cmd = self.format_command();
        tracing::info!("spawning command: {}", sub_cmd);
        let mut cmd = Command::new("cmd.exe");
        cmd.envs(self.environment.iter())
            .args(&["/C", &sub_cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut stdout_lines = tokio::io::BufReader::new(stdout).lines();
        let mut stderr_lines = tokio::io::BufReader::new(stderr).lines();
        let mut collected_stdout = Vec::new();
        let mut collected_stderr = Vec::new();

        loop {
            tokio::select! {
                line = stdout_lines.next_line() => {
                    match line? {
                        Some(line) => {
                            tracing::info!(fd = "stdout", "{}", line);
                            collected_stdout.push(line);
                        }
                        None => break,
                    }
                }
                line = stderr_lines.next_line() => {
                    match line? {
                        Some(line) => {
                            tracing::error!(fd = "stderr", "{}", line);
                            collected_stderr.push(line);
                        }
                        None => break,
                    }
                }
            }
        }

        let status = child.wait().await?;

        Ok(CommandResult {
            stdout: collected_stdout.join("\n"),
            stderr: collected_stderr.join("\n"),
            status,
        })
    }

    #[instrument]
    pub async fn is_success(&self) -> Result<()> {
        let result = self.run().await?;
        if !result.stderr.is_empty() {
            tracing::error!(fd = "stderr", "{}", result.stderr);
        }
        if !result.stdout.is_empty() {
            tracing::info!(fd = "stdout", "{}", result.stdout);
        }
        if result.status.success() {
            Ok(())
        } else {
            Err(eyre::eyre!("Command failed"))
        }
    }
}
