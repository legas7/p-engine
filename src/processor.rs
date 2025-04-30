use std::collections::HashMap;

use tokio::{
    sync::mpsc::{self, UnboundedReceiver},
    task::JoinHandle,
};

use crate::{
    account::Account,
    dispute_resolver::DisputesResolver,
    transactions::{ClientId, TransactionId, TransactionType, TxDTO},
};

type TransactionStatus = (TransactionId, ProcessingResult);

enum ProcessingResult {
    Success,
    Error(anyhow::Error),
}

struct ProcessorImplAlt<'a> {
    accounts: HashMap<ClientId, Account>,
    register: DisputesResolver<'a>,
}

impl<'a> ProcessorImplAlt<'a> {
    pub fn run() -> (UnboundedReceiver<TransactionStatus>, JoinHandle<()>) {
        let (sender, receiver) = mpsc::unbounded_channel::<TransactionStatus>();
        let handle = tokio::spawn(async move {
            let s = sender;
        });

        (receiver, handle)
    }

    fn process(&mut self, tx: &'a TxDTO) -> anyhow::Result<()> {
        if let Some(account) = self.accounts.get_mut(&tx.client_id) {
            match tx.detail {
                TransactionType::Deposit | TransactionType::Withdrawal => {
                    self.register.apply_adjustment(tx, account)
                }
                TransactionType::Dispute => self.register.open_dispute(tx, account),
                TransactionType::Resolve | TransactionType::Chargeback => {
                    self.register.close_dispute(tx, account)
                }
            }
        } else {
            self.accounts
                .insert(tx.client_id, Account::new(tx.client_id));
            Ok(())
        }
    }
}
