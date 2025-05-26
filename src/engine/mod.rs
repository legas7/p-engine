mod core;
pub mod objects;
pub mod processor;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum EngineError {
    Resolver_TransactionNotFound,
    Resolver_TransactionNotUnderDispute,
    Resolver_TransactionAlreadyUnderDispute,

    Account_DisputeReferencesDifferentClient_OnCreation,
    Account_DisputeReferencesDifferentClient_OnResolution,
    Account_AccountLocked,
    Account_NotEnoughFunds,

    Parsing_MissingAmountFieldConstructingAdjustment,
    Parsing_TryingToConstructAdjustmentFromIncompatibileTransaction,
    Parsing_TryingToConstructDisputeFromIncompatibileTransaction,
}
