use soroban_sdk::{Address, Bytes, Vec, contracttype};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub index: u32,           // 1-based index among all milestones for this financing
    pub deadline_ledger: u32, // the ledger sequence by which the milestone must be hit
    pub proof_hash: Bytes,    // sha256 of milestone evidence (deliverables, inspections, etc.)
    pub proof_amount: i128,   // amount of capital released against this milestone (i128);
                              // positive means release TO beneficiary, negative means refund FROM beneficiary
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Financing {
    pub id: u64,                       // contract-issued counter (AD-009)
    pub funder: Address,               // the funder/lender providing the capital
    pub beneficiary: Address,          // the farmer/recipient of the capital
    pub total_amount: i128,            // total committed financing amount
    pub drawn_amount: i128,            // cumulative amount drawn down so far
    pub milestone_count: u32,          // total number of milestones defined
    pub milestones: Vec<Milestone>,    // defined milestones (ordered by index)
    pub drawn_ledger: u32,             // ledger sequence at last drawdown
    pub repaid_amount: i128,           // cumulative amount repaid so far
    pub defaulted: bool,               // immutable flag set on default
    pub defaulted_ledger: Option<u32>, // ledger sequence at default
}
