use soroban_sdk::{Address, Bytes, contracttype};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseCondition {
    pub kind: u32,           // 0 = Manual, 1 = Milestone, 2 = Timeout
    pub releaser: Address, // Manual: authorized releaser; Milestone: proof verifier; unused for Timeout
    pub timeout_ledger: u32, // Timeout: ledger after which auto-refund allowed; unused otherwise
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Escrow {
    pub id: u64,
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub released_amount: i128,
    pub booking_ref: Bytes,
    pub condition: ReleaseCondition,
    pub created_ledger: u32,
    pub updated_ledger: u32,
}
