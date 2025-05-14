use std::ops::{AddAssign, Deref, SubAssign};

use anyhow::{Context, anyhow};

#[derive(Clone)]
pub struct TransactionDTO {
    pub id: TransactionId,
    pub client_id: ClientId,
    pub kind: TxKind,
    pub amount: Option<f32>,
}

#[derive(Clone, Copy)]
pub enum TxKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone, Copy)]
pub enum AdjustmentKind {
    Deposit,
    Withdrawal,
}

#[derive(Clone, Copy)]
pub enum ResolutionKind {
    Resolve,
    Chargeback,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TransactionId(pub u32);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ClientId(pub u16);

pub struct Adjustment {
    pub category: AdjustmentKind,
    pub details: TxDetails,
    pub amount: TxAmount,
}

#[derive(Clone, Copy)]
pub struct TxAmount(pub f32);

pub struct TxDetails {
    pub id: TransactionId,
    pub client_id: ClientId,
}

pub struct DisputeClaim {
    pub client_id: ClientId,
    pub kind: AdjustmentKind,
    pub amount: TxAmount,
}

impl Deref for ClientId {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for TransactionId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign for TxAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl SubAssign for TxAmount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Deref for TxAmount {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<TransactionDTO> for Adjustment {
    type Error = anyhow::Error;

    fn try_from(value: TransactionDTO) -> Result<Self, Self::Error> {
        Ok(Adjustment {
            category: value.kind.try_into()?,
            details: TxDetails {
                id: value.id,
                client_id: value.client_id,
            },
            amount: TxAmount(value.amount.context(
                "Missing 'amount' field in TransactionDTO. Cannot construct Adjustment",
            )?),
        })
    }
}

impl TryFrom<TxKind> for AdjustmentKind {
    type Error = anyhow::Error;

    fn try_from(value: TxKind) -> Result<Self, Self::Error> {
        match value {
            TxKind::Deposit => Ok(AdjustmentKind::Deposit),
            TxKind::Withdrawal => Ok(AdjustmentKind::Withdrawal),
            _ => Err(anyhow!(
                "Tried to construct AdjustmentType from incompatibile TransactionType"
            )),
        }
    }
}

impl TryFrom<TxKind> for ResolutionKind {
    type Error = anyhow::Error;

    fn try_from(value: TxKind) -> Result<Self, Self::Error> {
        match value {
            TxKind::Chargeback => Ok(ResolutionKind::Chargeback),
            TxKind::Resolve => Ok(ResolutionKind::Resolve),
            _ => Err(anyhow!(
                "Tried to construct DisputeType from incompatibile TransactionType"
            )),
        }
    }
}
