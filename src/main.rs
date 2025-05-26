use std::{env, error::Error, ops::Deref, str::FromStr};

use engine::{
    objects::{ClientId, TransactionDTO, TransactionId, TxKind},
    processor::ProcessorImpl,
};
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::{
    io::AsyncBufReadExt,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

mod engine;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();
    let file = tokio::fs::File::open(&args[1]).await.unwrap();

    let mut reader = tokio::io::BufReader::new(file).lines();
    let (t_sender, t_receiver) = tokio::sync::mpsc::unbounded_channel::<TransactionDTO>();

    while let Ok(Some(line)) = reader.next_line().await {
        if let Ok(res) = parse_input_line(line) {
            _ = t_sender.send(res)
        }
    }
    // In streaming input scenario (not file) this should be dropped when service receives shutdown signal
    // It would provide graceful shutdown to all processing units
    drop(t_sender);

    run_scaled(2, t_receiver).await;
}

async fn run_scaled(instance_count: u16, mut rx: UnboundedReceiver<TransactionDTO>) {
    let mut senders: Vec<UnboundedSender<TransactionDTO>> = Vec::new();
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for i in 0..instance_count {
        let (t_sender, t_receiver) = tokio::sync::mpsc::unbounded_channel::<TransactionDTO>();
        let proc_handle = ProcessorImpl::run(t_receiver, i);
        senders.push(t_sender);
        handles.push(proc_handle.1);
    }

    while let Some(transaction) = rx.recv().await {
        let bucket = transaction.client_id.deref() % instance_count;
        _ = senders[bucket as usize].send(transaction);
    }
    // notify instances that all inputs are processed by closing channels' tx end
    senders.clear();

    // wait till instances finsh work
    handles
        .into_iter()
        .map(async |jh| jh.await)
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;
}

fn parse_input_line(line: String) -> Result<TransactionDTO, Box<dyn Error>> {
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
        amount: {
            let amount_str = linesplit.get(3);
            amount_str.and_then(|e| e.trim().parse::<f32>().ok())
        },
    })
}
