use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FinancingError {
    NotInitialized = 1,
    Unauthorized = 2,
    FinancingNotFound = 3,
    InvalidInput = 4,
    MilestoneDeadlineExceeded = 5,
}
