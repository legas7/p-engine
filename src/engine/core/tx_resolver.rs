use std::collections::HashMap;

use anyhow::anyhow;

use crate::engine::objects::{Adjustment, DisputeClaim, TransactionDTO, TransactionId};

use super::account::Account;

pub struct TxResolver {
    transaction_log: HashMap<TransactionId, Adjustment>,
    active_disputes: HashMap<TransactionId, DisputeClaim>,
}

impl TxResolver {
    pub fn new() -> Self {
        Self {
            transaction_log: Default::default(),
            active_disputes: Default::default(),
        }
    }

    pub fn apply_adjustment(
        &mut self,
        tx: TransactionDTO,
        account: &mut Account,
    ) -> anyhow::Result<()> {
        account.apply_adjustment(tx).map(|applied_adjustment| {
            let adjustment_id = applied_adjustment.details.id;
            self.transaction_log
                .insert(adjustment_id, applied_adjustment);
        })
    }

    pub fn open_dispute(
        &mut self,
        tx_id: &TransactionId,
        account: &mut Account,
    ) -> anyhow::Result<()> {
        match self.transaction_log.get(tx_id) {
            Some(disputed_tx) => {
                if self.active_disputes.contains_key(&disputed_tx.details.id) {
                    return Err(anyhow!("Transaction currently under dispute"));
                }

                account.open_dispute(disputed_tx).map(|claim| {
                    self.active_disputes.insert(disputed_tx.details.id, claim);
                })
            }
            None => Err(anyhow!("Transaction not found")),
        }
    }

    pub fn close_dispute(
        &mut self,
        tx: TransactionDTO,
        account: &mut Account,
    ) -> anyhow::Result<()> {
        match self.active_disputes.get(&tx.id) {
            Some(disputed_tx) => account
                .resolve_dispute(disputed_tx, &tx.id, &tx.client_id, &tx.kind.try_into()?)
                .map(|resolved_tx_id| {
                    self.active_disputes.remove(&resolved_tx_id);
                }),
            None => Err(anyhow!("Transaction not under dispute")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::{
        core::account::Account,
        objects::{ClientId, TransactionDTO, TransactionId, TxKind},
    };

    use super::TxResolver;

    #[test]
    fn opening_dispute_for_missing_transaction_fails() {
        let mut account = Account {
            available: 100.0,
            total: 100.0,
            ..Account::new(ClientId(1))
        };
        let tx = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Dispute,
            amount: None,
        };
        let mut resolver = TxResolver::new();
        let res = resolver.open_dispute(&tx.id, &mut account);
        assert!(res.is_err())
    }

    #[test]
    fn opening_new_dispute_for_already_disputed_transaction_fails() {
        let mut account = Account {
            available: 100.0,
            total: 100.0,
            ..Account::new(ClientId(1))
        };
        let tx0 = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(100.0),
        };
        let tx1 = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Dispute,
            amount: None,
        };
        let mut resolver = TxResolver::new();

        assert!(resolver.apply_adjustment(tx0, &mut account).is_ok());
        assert!(resolver.open_dispute(&tx1.id, &mut account).is_ok());
        assert!(resolver.open_dispute(&tx1.id, &mut account).is_err())
    }

    #[test]
    fn closing_not_disputed_transaction_fails() {
        let mut account = Account {
            available: 100.0,
            total: 100.0,
            ..Account::new(ClientId(1))
        };
        let tx0 = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(100.0),
        };
        let tx1 = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Dispute,
            amount: None,
        };
        let mut resolver = TxResolver::new();

        let res = resolver.apply_adjustment(tx0, &mut account);
        assert!(res.is_ok());

        let res = resolver.close_dispute(tx1, &mut account);
        assert!(res.is_err())
    }

    #[test]
    fn open_and_close_dispute_succesfully() {
        let mut account = Account::new(ClientId(1));
        let mut resolver = TxResolver::new();
        let tx0 = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(100.0),
        };
        let tx1 = TransactionDTO {
            id: TransactionId(2),
            client_id: ClientId(1),
            kind: TxKind::Withdrawal,
            amount: Some(30.0),
        };
        let tx2 = TransactionDTO {
            id: TransactionId(2),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(60.0),
        };
        let tx0_chargeback = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Chargeback,
            amount: None,
        };

        assert!(resolver.apply_adjustment(tx0.clone(), &mut account).is_ok());
        assert!(resolver.apply_adjustment(tx1, &mut account).is_ok());

        // opening dispute regardless of account balance
        assert!(resolver.open_dispute(&tx0.id, &mut account).is_ok());
        assert_eq!(account.available, -30.0);
        assert_eq!(account.total, 70.0);
        assert_eq!(account.held, 100.0);

        // can't chargeback dispute due to insufficient funds
        assert!(
            resolver
                .close_dispute(tx0_chargeback.clone(), &mut account)
                .is_err()
        );
        assert!(!account.locked);

        assert!(resolver.apply_adjustment(tx2, &mut account).is_ok());

        // dispute can be charged back when funds are available
        assert!(resolver.close_dispute(tx0_chargeback, &mut account).is_ok());
        assert_eq!(account.available, 30.0);
        assert_eq!(account.total, 30.0);
        assert_eq!(account.held, 0.0);
    }
}
