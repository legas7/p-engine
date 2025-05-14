use std::collections::HashMap;

use tokio::{
    sync::mpsc::{self, UnboundedReceiver},
    task::JoinHandle,
};

use crate::engine::objects::{ClientId, TransactionDTO, TransactionId, TxKind};

use super::core::{account::Account, tx_resolver::TxResolver};

type TransactionStatus = (TransactionId, ProcessingResult);

#[allow(dead_code)]
pub enum ProcessingResult {
    Success,
    Error(anyhow::Error),
}

pub struct ProcessorImpl {
    accounts: HashMap<ClientId, Account>,
    resolver: TxResolver,
}

impl ProcessorImpl {
    pub fn run(
        mut rx: UnboundedReceiver<TransactionDTO>,
    ) -> (UnboundedReceiver<TransactionStatus>, JoinHandle<()>) {
        let (sender, receiver) = mpsc::unbounded_channel::<TransactionStatus>();
        let handle = tokio::spawn(async move {
            let mut processor = Self {
                accounts: Default::default(),
                resolver: TxResolver::new(),
            };

            while let Some(transaction) = rx.recv().await {
                let tx_id = transaction.id;
                // TODO: nicer result handling
                match processor.process(transaction) {
                    Ok(()) => {
                        _ = sender.send((tx_id, ProcessingResult::Success));
                    }
                    Err(e) => {
                        _ = sender.send((tx_id, ProcessingResult::Error(e)));
                    }
                }
            }
            processor.print_account_balances_to_stdout();
        });

        (receiver, handle)
    }

    fn print_account_balances_to_stdout(&self) {
        for account in self.accounts.values() {
            println!("{}", account.to_csv());
        }
    }

    fn process(&mut self, tx: TransactionDTO) -> anyhow::Result<()> {
        let account = self
            .accounts
            .entry(tx.client_id)
            .or_insert(Account::new(tx.client_id));

        match tx.kind {
            TxKind::Deposit | TxKind::Withdrawal => self.resolver.apply_adjustment(tx, account),
            TxKind::Dispute => self.resolver.open_dispute(&tx.id, account),
            TxKind::Resolve | TxKind::Chargeback => self.resolver.close_dispute(tx, account),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::discriminant;

    use anyhow::anyhow;
    use tokio::sync::mpsc;

    use crate::engine::{
        objects::{ClientId, TransactionDTO, TransactionId, TxKind},
        processor::{ProcessingResult, ProcessorImpl, TransactionStatus},
    };

    #[tokio::test]
    async fn processor_returns_results_and_errors() {
        let (sender, receiver) = mpsc::unbounded_channel::<TransactionDTO>();
        let client_id = 1;

        let transactions: Vec<(TransactionDTO, TransactionStatus)> = [
            (
                TransactionDTO {
                    id: TransactionId(1),
                    client_id: ClientId(client_id),
                    kind: TxKind::Deposit,
                    amount: Some(100.0),
                },
                (TransactionId(1), ProcessingResult::Success),
            ),
            (
                TransactionDTO {
                    id: TransactionId(2),
                    client_id: ClientId(client_id),
                    kind: TxKind::Withdrawal,
                    amount: Some(50.0),
                },
                (TransactionId(2), ProcessingResult::Success),
            ),
            (
                TransactionDTO {
                    id: TransactionId(2),
                    client_id: ClientId(client_id),
                    kind: TxKind::Dispute,
                    amount: None,
                },
                (TransactionId(2), ProcessingResult::Success),
            ),
            (
                TransactionDTO {
                    id: TransactionId(2),
                    client_id: ClientId(client_id),
                    kind: TxKind::Chargeback,
                    amount: None,
                },
                (TransactionId(2), ProcessingResult::Success),
            ),
            (
                TransactionDTO {
                    id: TransactionId(100),
                    client_id: ClientId(client_id),
                    kind: TxKind::Chargeback,
                    amount: None,
                },
                (
                    TransactionId(100),
                    ProcessingResult::Error(anyhow!("Transaction not under dispute")),
                ),
            ),
            (
                TransactionDTO {
                    id: TransactionId(3),
                    client_id: ClientId(client_id),
                    kind: TxKind::Withdrawal,
                    amount: Some(70.0),
                },
                (TransactionId(3), ProcessingResult::Success),
            ),
            (
                TransactionDTO {
                    id: TransactionId(4),
                    client_id: ClientId(client_id),
                    kind: TxKind::Withdrawal,
                    amount: Some(40.0),
                },
                (
                    TransactionId(4),
                    ProcessingResult::Error(anyhow!("Not enough funds")),
                ),
            ),
            (
                TransactionDTO {
                    id: TransactionId(500),
                    client_id: ClientId(client_id),
                    kind: TxKind::Dispute,
                    amount: Some(40.0),
                },
                (
                    TransactionId(500),
                    ProcessingResult::Error(anyhow!("Transaction not found")),
                ),
            ),
        ]
        .into_iter()
        .collect();

        let (mut results, _handle) = ProcessorImpl::run(receiver);
        for transaction in transactions {
            let expect_res = transaction.1.1;
            let expect_id = transaction.1.0;

            sender.send(transaction.0).unwrap();
            let (id, res) = results.recv().await.unwrap();

            assert_eq!(id, expect_id);
            assert_eq!(discriminant(&res), discriminant(&expect_res));
        }
    }
}
