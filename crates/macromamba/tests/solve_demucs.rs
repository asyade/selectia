use macromamba::*;

#[tokio::test]
async fn solve_demucs() {
    tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .init();
    // let mut env = Environment::new("environments".into());
    // env.install(&Path::new("./tests/env-gpu.yml")).await.unwrap();
    // if env.activate_env("demucs-gpu").await.is_err() {
    //     env.create_env_from_file("./tests/env-gpu.yml").await.unwrap();
    // } else {
    //     tracing::info!("environment already exists");
    // }

    // env.activate_env("demucs-gpu").await.unwrap();

    // env.run_script_within_env("demucs-gpu","./tests/executor.py").await.unwrap();

}

// micro-mamba/envs/py312/python.exe -m demucs.separate -d cuda input.mp3
