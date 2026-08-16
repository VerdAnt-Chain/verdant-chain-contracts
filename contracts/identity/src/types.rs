use soroban_sdk::{Address, Bytes, Vec, contracttype};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationMarker {
    pub kind: Bytes,
    pub issuer: Address,
    pub issued_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Farmer {
    pub address: Address,
    pub metadata_hash: Bytes,
    pub verification_markers: Vec<VerificationMarker>,
    pub created_ledger: u32,
    pub updated_ledger: u32,
}
