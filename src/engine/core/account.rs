use std::ops::Deref;

use crate::engine::{
    EngineError,
    objects::{
        Adjustment, AdjustmentKind, ClientId, DisputeClaim, ResolutionKind, TransactionDTO,
        TransactionId,
    },
};


pub struct Account {
    pub client_id: ClientId,
    pub available: f32, // for trading
    pub held: f32,      // for disputes
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }

    pub fn to_csv(&self) -> String {
        let total = self.available + self.held;
        format!(
            "{},{:.4},{:.4},{:.4},{}",
            *self.client_id, self.available, self.held, total, self.locked
        )
    }

    pub fn apply_adjustment(&mut self, tx: TransactionDTO) -> Result<Adjustment, EngineError> {
        self.check_account_lock()?;
        let adjustment: Adjustment = tx.try_into()?;
        let amount = adjustment.amount.deref();
        match adjustment.category {
            AdjustmentKind::Deposit => {
                self.available += amount;
            }
            AdjustmentKind::Withdrawal => {
                let new_balance = self.available - amount;
                if new_balance >= 0.0f32 {
                    self.available = new_balance;
                } else {
                    return Err(EngineError::Account_NotEnoughFunds);
                }
            }
        }
        Ok(adjustment)
    }

    pub fn open_dispute(
        &mut self,
        disputed_adjustment: &Adjustment,
    ) -> Result<DisputeClaim, EngineError> {
        self.check_account_lock()?;
        if self.client_id != disputed_adjustment.details.client_id {
            return Err(EngineError::Account_DisputeReferencesDifferentClient_OnCreation);
        }

        let amount = disputed_adjustment.amount.deref();
        match disputed_adjustment.category {
            AdjustmentKind::Deposit => {
                self.available -= amount;
                self.held += amount;
            }
            AdjustmentKind::Withdrawal => {
                // not giving money in advance - no provisional refund here ;)
            }
        }

        Ok(DisputeClaim {
            client_id: disputed_adjustment.details.client_id,
            kind: disputed_adjustment.category,
            amount: disputed_adjustment.amount,
        })
    }

    pub fn resolve_dispute(
        &mut self,
        claim: &DisputeClaim,
        tx_id: &TransactionId,
        tx_client_id: &ClientId,
        resolution_category: &ResolutionKind,
    ) -> Result<TransactionId, EngineError> {
        self.check_account_lock()?;
        let amount = claim.amount;

        if &claim.client_id != tx_client_id {
            return Err(EngineError::Account_DisputeReferencesDifferentClient_OnResolution);
        }

        match (claim.kind, resolution_category) {
            (AdjustmentKind::Deposit, ResolutionKind::Resolve) => {
                self.available += *amount;
                self.held -= *amount;
            }
            (AdjustmentKind::Deposit, ResolutionKind::Chargeback) => {
                let total = self.held + self.available;
                if *amount > total {
                    return Err(EngineError::Account_NotEnoughFunds);
                }
                self.held -= *amount;
                self.locked = true;
            }
            (AdjustmentKind::Withdrawal, ResolutionKind::Chargeback) => {
                self.available += *amount;
            }
            (AdjustmentKind::Withdrawal, ResolutionKind::Resolve) => {
                // no provisional refunds were made when opening a dispute, so it's a no-op
            }
        }

        Ok(*tx_id)
    }

    fn check_account_lock(&self) -> Result<(), EngineError> {
        if self.locked {
            Err(EngineError::Account_AccountLocked)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::objects::{
        AdjustmentKind, ClientId, DisputeClaim, TransactionDTO, TransactionId, TxAmount, TxKind,
    };

    use super::Account;

    #[test]
    fn adjustment_fails_on_locked_account() {
        let mut account = Account {
            locked: true,
            ..Account::new(ClientId(1))
        };
        let tx = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(100.0),
        };

        let res = account.apply_adjustment(tx);

        assert!(res.is_err())
    }

    #[test]
    fn withdrawal_fails_when_insufficient_funds() {
        let mut account = Account {
            available: 100.0,
            ..Account::new(ClientId(1))
        };
        let tx = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Withdrawal,
            amount: Some(110.0),
        };

        let res = account.apply_adjustment(tx);

        assert!(res.is_err());
        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 0.0);
    }

    #[test]
    fn dispute_on_deposit_blocks_funds() {
        let mut account = Account {
            available: 100.0,
            ..Account::new(ClientId(1))
        };
        let tx = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(50.0),
        };

        let adjustment = account.apply_adjustment(tx).unwrap();

        assert_eq!(account.available, 150.0);
        assert_eq!(account.held, 0.0);

        let res = account.open_dispute(&adjustment);

        assert!(res.is_ok());
        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 50.0);
    }

    #[test]
    fn resolution_on_disputed_deposit_unblocks_funds() {
        let mut account = Account {
            available: 100.0,
            held: 100.0,
            ..Account::new(ClientId(1))
        };
        let claim = DisputeClaim {
            client_id: ClientId(1),
            kind: AdjustmentKind::Deposit,
            amount: TxAmount(50.0),
        };
        let tx = TransactionDTO {
            id: TransactionId(0),
            client_id: ClientId(1),
            kind: TxKind::Resolve,
            amount: None,
        };

        let res =
            account.resolve_dispute(&claim, &tx.id, &tx.client_id, &tx.kind.try_into().unwrap());

        assert!(res.is_ok());
        assert_eq!(account.available, 150.0);
        assert_eq!(account.held, 50.0);
    }

    #[test]
    fn chargeback_on_disputed_deposit_decreases_funds() {
        let mut account = Account {
            available: 100.0,
            held: 100.0,
            ..Account::new(ClientId(1))
        };
        let claim = DisputeClaim {
            client_id: ClientId(1),
            kind: AdjustmentKind::Deposit,
            amount: TxAmount(50.0),
        };

        let tx = TransactionDTO {
            id: TransactionId(0),
            client_id: ClientId(1),
            kind: TxKind::Chargeback,
            amount: None,
        };

        let res =
            account.resolve_dispute(&claim, &tx.id, &tx.client_id, &tx.kind.try_into().unwrap());

        assert!(res.is_ok());
        assert!(account.locked);
        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 50.0);
    }

    #[test]
    fn deposit_and_withdraw_are_processed_succesfully() {
        let mut account = Account {
            available: 100.0,
            ..Account::new(ClientId(1))
        };
        let tx0 = TransactionDTO {
            id: TransactionId(1),
            client_id: ClientId(1),
            kind: TxKind::Deposit,
            amount: Some(50.0),
        };
        let tx1 = TransactionDTO {
            id: TransactionId(2),
            client_id: ClientId(1),
            kind: TxKind::Withdrawal,
            amount: Some(50.0),
        };

        let _adjustment = account.apply_adjustment(tx0).unwrap();

        assert_eq!(account.available, 150.0);
        assert_eq!(account.held, 0.0);

        let _adjustment = account.apply_adjustment(tx1).unwrap();

        assert_eq!(account.available, 100.0);
        assert_eq!(account.held, 0.0);
    }
}
