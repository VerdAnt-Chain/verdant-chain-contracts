use soroban_sdk::{Address, contractevent};

use crate::types::Verification;

#[contractevent]
pub struct Initialized {
    #[topic]
    admin: Address,
}

#[contractevent]
pub struct VerificationCreated {
    #[topic]
    verification_id: u64,
    record: Verification,
}

#[contractevent]
pub struct VerificationRevoked {
    #[topic]
    verification_id: u64,
    record: Verification,
}

pub fn initialized(env: &soroban_sdk::Env, admin: &Address) {
    env.events().publish_event(&Initialized {
        admin: admin.clone(),
    });
}

pub fn verification_created(env: &soroban_sdk::Env, id: u64, record: &Verification) {
    env.events().publish_event(&VerificationCreated {
        verification_id: id,
        record: record.clone(),
    });
}

pub fn verification_revoked(env: &soroban_sdk::Env, id: u64, record: &Verification) {
    env.events().publish_event(&VerificationRevoked {
        verification_id: id,
        record: record.clone(),
    });
}
