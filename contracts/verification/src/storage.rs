use soroban_sdk::{Address, Bytes, Env, contracttype, panic_with_error};

use crate::errors::VerificationError;
use crate::types::Verification;

#[contracttype]
enum DataKey {
    Admin,
    VerificationAuthority,
    NextVerificationId,
    Verification(u64),
    BatchVerifications(Bytes),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

#[allow(dead_code)]
pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .expect("admin not set")
}

pub fn set_verification_authority(env: &Env, authority: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::VerificationAuthority, authority);
}

pub fn get_verification_authority(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::VerificationAuthority)
        .expect("verification authority not set")
}

pub fn require_initialized(env: &Env) {
    if !env.storage().persistent().has(&DataKey::Admin) {
        panic_with_error!(env, VerificationError::NotInitialized);
    }
}

pub fn require_authority(env: &Env, caller: &Address) {
    let authority = get_verification_authority(env);
    if caller != &authority {
        panic_with_error!(env, VerificationError::Unauthorized);
    }
}

pub fn get_next_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextVerificationId)
        .unwrap_or(1)
}

pub fn set_next_id(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::NextVerificationId, &id);
}

pub fn set_verification(env: &Env, id: u64, record: &Verification) {
    env.storage()
        .persistent()
        .set(&DataKey::Verification(id), record);
}

pub fn get_verification(env: &Env, id: u64) -> Verification {
    env.storage()
        .persistent()
        .get(&DataKey::Verification(id))
        .unwrap_or_else(|| panic_with_error!(env, VerificationError::VerificationNotFound))
}

pub fn set_batch_verifications(env: &Env, batch: &Bytes, ids: &soroban_sdk::Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::BatchVerifications(batch.clone()), ids);
}

pub fn get_batch_verifications(env: &Env, batch: &Bytes) -> soroban_sdk::Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::BatchVerifications(batch.clone()))
        .unwrap_or_else(|| soroban_sdk::Vec::new(env))
}
