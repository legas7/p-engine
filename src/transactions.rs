use std::ops::Deref;

use anyhow::anyhow;

pub struct TxDTO {
    pub id: TransactionId,
    pub client_id: ClientId,
    pub detail: TransactionType,
    pub amount: f32,
}

#[derive(Clone, Copy)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct TransactionId(u32);
impl Deref for TransactionId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ClientId(u16);
impl Deref for ClientId {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Adjustment<'a> {
    pub category: AdjustmentType,
    pub details: TxDetails<'a>,
    pub amount: TxAmount<'a>,
}
pub enum AdjustmentType {
    Deposit,
    Withdrawal,
}

pub struct TxAmount<'a>(&'a f32);

pub struct TxDetails<'a> {
    pub id: &'a TransactionId,
    pub client_id: &'a ClientId,
}

pub struct DisputeClaim {
    pub client_id: ClientId,
    pub amount: f32,
}

pub struct DisputeResolution<'a> {
    pub category: ResolutionType,
    pub details: TxDetails<'a>,
}

pub enum ResolutionType {
    Resolve,
    Chargeback,
}

impl<'a> TryFrom<&'a TxDTO> for Adjustment<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a TxDTO) -> Result<Self, Self::Error> {
        Ok(Adjustment {
            category: value.detail.try_into()?,
            details: TxDetails {
                id: &value.id,
                client_id: &value.client_id,
            },
            amount: TxAmount(&value.amount),
        })
    }
}

impl TryFrom<TransactionType> for AdjustmentType {
    type Error = anyhow::Error;

    fn try_from(value: TransactionType) -> Result<Self, Self::Error> {
        match value {
            TransactionType::Deposit => Ok(AdjustmentType::Deposit),
            TransactionType::Withdrawal => Ok(AdjustmentType::Withdrawal),
            _ => Err(anyhow!(
                "Tried to construct AdjustmentType from incompatibile TransactionType"
            )),
        }
    }
}

impl<'a> TryFrom<&'a TxDTO> for DisputeResolution<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a TxDTO) -> Result<Self, Self::Error> {
        Ok(DisputeResolution {
            category: value.detail.try_into()?,
            details: TxDetails {
                id: &value.id,
                client_id: &value.client_id,
            },
        })
    }
}

impl TryFrom<TransactionType> for ResolutionType {
    type Error = anyhow::Error;

    fn try_from(value: TransactionType) -> Result<Self, Self::Error> {
        match value {
            TransactionType::Chargeback => Ok(ResolutionType::Chargeback),
            TransactionType::Resolve => Ok(ResolutionType::Resolve),
            _ => Err(anyhow!(
                "Tried to construct DisputeType from incompatibile TransactionType"
            )),
        }
    }
}

impl<'a> Deref for TxAmount<'a> {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
