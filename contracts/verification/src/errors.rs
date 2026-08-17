use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VerificationError {
    NotInitialized = 1,
    Unauthorized = 2,
    VerificationNotFound = 3,
    InvalidInput = 4,
}
