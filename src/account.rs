use std::ops::Deref;

use crate::transactions::{
    Adjustment, AdjustmentType, ClientId, DisputeClaim, DisputeResolution, ResolutionType, TxDTO,
};

use anyhow::anyhow;

pub struct Account {
    pub client_id: ClientId,
    pub available: f32, // for trading
    pub held: f32,      // for disputes
    pub total: f32,     // available + held
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn apply_adjustment(&mut self, tx: &TxDTO) -> anyhow::Result<()> {
        self.check_account_lock()?;
        let adjustment: Adjustment = tx.try_into()?;
        let amount = adjustment.amount.deref();
        match adjustment.category {
            AdjustmentType::Deposit => {
                self.available += amount;
                self.total += amount;
                Ok(())
            }
            AdjustmentType::Withdrawal => {
                let new_balance = self.available - amount;
                if new_balance >= 0.0f32 {
                    self.available = new_balance;
                    self.total -= amount;
                    Ok(())
                } else {
                    Err(anyhow!("Not enough funds"))
                }
            }
        }
    }

    pub fn open_dispute(&mut self, disputed_adjustment: &Adjustment) -> anyhow::Result<()> {
        self.check_account_lock()?;
        if &self.client_id != disputed_adjustment.details.client_id {
            return Err(anyhow!(
                "Dispute references different client_id than adjustment transaction"
            ));
        }

        let amount = disputed_adjustment.amount.deref();
        self.available -= amount;
        self.held += amount;

        Ok(())
    }

    pub fn resolve_dispute<'a>(
        &mut self,
        claim: &DisputeClaim,
        resolution: DisputeResolution,
    ) -> anyhow::Result<()> {
        self.check_account_lock()?;
        let amount = claim.amount;
        match resolution.category {
            ResolutionType::Resolve => {
                self.available += amount;
                self.held -= amount;
                Ok(())
            }
            ResolutionType::Chargeback => {
                self.total -= amount;
                self.held -= amount;
                self.locked = true;
                Ok(())
            }
        }
    }

    fn check_account_lock(&self) -> anyhow::Result<()> {
        if self.locked {
            return Err(anyhow!("Account locked"));
        } else {
            Ok(())
        }
    }
}
