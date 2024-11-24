use models::File;
// use rust_bert::pipelines::sentence_embeddings::{SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType};
use state_machine::StateMachineEvent;

use crate::prelude::*;
use crate::services::{CancelableTask, AddressableService, ThreadedService};

pub type Embedding = ThreadedService<StateMachineEvent>;

pub fn embedding(state_machine: StateMachine) -> Embedding {
    ThreadedService::new(move |receiver| embedding_task(receiver, state_machine))
}

fn embedding_task(_receiver: sync::mpsc::Receiver<StateMachineEvent>, _state_machine: StateMachine) -> Result<()> {
    // let model = SentenceEmbeddingsBuilder::remote(
    //     SentenceEmbeddingsModelType::AllMiniLmL12V2
    // ).create_model()?;
    // info!("Embedding task begin");

    // while let Some(task) = receiver.blocking_recv() {
    //     info!("Got task");
    //     match task {
    //         StateMachineEvent::FileIngested(file) => {
    //             let identifier = prepare_path_for_embedding(file.path.clone());
    //             let embedding = model.encode(&[&identifier])?;
    //             state_machine.blocking_send(StateMachineTask::set_tag("file_name_embedding".to_string(), bincode::serialize(&embedding)?, Some(file.metadata_id)))?;
    //         },
    //         StateMachineEvent::Exit => {
    //             info!("Exiting embedding task");
    //             break;
    //         }
    //     }
    // }
    // info!("Embedding task done");
    Ok(())
}

fn _prepare_path_for_embedding(path: String) -> String {
    let path_buf = PathBuf::from(path);
    let count = path_buf.iter().count();

    if count == 0 {
        return String::new();
    }

    let mut striped_path = if count > 2 {
        let mut path_parts = path_buf.iter().rev().take(2).map(|p| p.to_str().unwrap()).collect::<Vec<_>>();
        path_parts.reverse();
        path_parts.join("/")
    } else {
        path_buf.iter().last().map(|p| p.to_str().unwrap()).unwrap().to_string()
    };

    striped_path = striped_path.replace("_", " - ");
    striped_path = striped_path.replace("\t", " ");
    striped_path = striped_path.to_lowercase();
    striped_path = striped_path.split(" ").map(|s| s.trim()).collect::<Vec<_>>().join(" ");
    striped_path
}
