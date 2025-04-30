use std::collections::HashMap;

use anyhow::anyhow;

use crate::{
    account::Account,
    transactions::{Adjustment, DisputeClaim, TransactionId, TxDTO},
};

pub struct DisputesResolver<'a> {
    transaction_log: HashMap<TransactionId, Adjustment<'a>>,
    active_disputes: HashMap<TransactionId, DisputeClaim>,
}

impl<'a> DisputesResolver<'a> {
    pub fn apply_adjustment(&mut self, tx: &'a TxDTO, account: &mut Account) -> anyhow::Result<()> {
        account.apply_adjustment(tx).and_then(|_| {
            self.transaction_log.insert(tx.id, tx.try_into()?);
            Ok(())
        })
    }

    pub fn open_dispute(&mut self, tx: &'a TxDTO, account: &mut Account) -> anyhow::Result<()> {
        match self.transaction_log.get(&tx.id) {
            Some(disputed_tx) => {
                if self.active_disputes.contains_key(disputed_tx.details.id) {
                    return Err(anyhow!("Transaction currently under dispute"));
                }

                account.open_dispute(disputed_tx).and_then(|_| {
                    self.active_disputes.insert(
                        *disputed_tx.details.id,
                        DisputeClaim {
                            client_id: *disputed_tx.details.client_id,
                            amount: *disputed_tx.amount,
                        },
                    );
                    Ok(())
                })
            }
            None => Err(anyhow!("Transaction not found")),
        }
    }

    pub fn close_dispute(&mut self, tx: &TxDTO, account: &mut Account) -> anyhow::Result<()> {
        match self.active_disputes.get(&tx.id) {
            Some(disputed_tx) => account
                .resolve_dispute(disputed_tx, tx.try_into()?)
                .and_then(|_| {
                    self.active_disputes.remove(&tx.id);
                    Ok(())
                }),
            None => Err(anyhow!("Transaction not under dispute")),
        }
    }
}
