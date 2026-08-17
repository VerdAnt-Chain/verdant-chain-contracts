use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    NotInitialized = 1,
    Unauthorized = 2,
    EscrowNotFound = 3,
    InsufficientBalance = 4,
    ConditionNotMet = 5,
    AlreadyFullyReleased = 6,
    TimeoutNotElapsed = 7,
    InvalidInput = 8,
}