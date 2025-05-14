use std::{env, str::FromStr};

use engine::{
    objects::{ClientId, TransactionDTO, TransactionId, TxKind},
    processor::ProcessorImpl,
};
use tokio::io::AsyncBufReadExt;

mod engine;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();
    let file = tokio::fs::File::open(&args[1]).await.unwrap();

    let mut reader = tokio::io::BufReader::new(file).lines();
    let (t_sender, t_receiver) = tokio::sync::mpsc::unbounded_channel::<TransactionDTO>();

    let (_processing_results, handle) = ProcessorImpl::run(t_receiver);

    while let Ok(Some(line)) = reader.next_line().await {
        if let Ok(res) = parse_input_line(line) {
            _ = t_sender.send(res)
        }
    }
    drop(t_sender);

    handle.await.unwrap();
}

fn parse_input_line(line: String) -> anyhow::Result<TransactionDTO> {
    let linesplit: Vec<&str> = line.split(',').collect();
    Ok(TransactionDTO {
        id: {
            let id_str = linesplit[2].trim();
            TransactionId(id_str.parse()?)
        },
        client_id: {
            let client_id_str = linesplit[1].trim();
            ClientId(client_id_str.parse()?)
        },
        kind: {
            let kind_str = linesplit[0].trim();
            TxKind::from_str(kind_str)?
        },
        amount: Some({
            let amount_str = linesplit[3].trim();
            amount_str.parse()?
        }),
    })
}
