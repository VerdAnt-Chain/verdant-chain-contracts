use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IdentityError {
    NotInitialized = 1,
    AlreadyRegistered = 2,
    FarmerNotFound = 3,
    Unauthorized = 4,
}
