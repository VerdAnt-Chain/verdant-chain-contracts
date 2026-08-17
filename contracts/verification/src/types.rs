use soroban_sdk::{Address, Bytes, Vec, contracttype};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verification {
    pub id: u64,
    pub batch: Bytes,
    pub subject: Address,
    pub proof_hash: Bytes,
    pub issuer: Address,
    pub issued_ledger: u32,
    pub revoked: bool,
    pub revoked_ledger: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationIdList(pub Vec<u64>);
