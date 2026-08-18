use soroban_sdk::{Address, Env, Vec, contracttype, panic_with_error};

use crate::errors::FinancingError;
use crate::types::Financing;

#[contracttype]
enum DataKey {
    Admin,
    FinancingAuthority,
    NextFinancingId,
    Financing(u64),
    FunderFinancings(Address),
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

pub fn set_financing_authority(env: &Env, authority: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::FinancingAuthority, authority);
}

pub fn get_financing_authority(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::FinancingAuthority)
        .expect("financing authority not set")
}

pub fn set_next_id(env: &Env, id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::NextFinancingId, &id);
}

pub fn next_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextFinancingId)
        .unwrap_or(1)
}

pub fn set_financing(env: &Env, id: u64, financing: &Financing) {
    env.storage()
        .persistent()
        .set(&DataKey::Financing(id), financing);
}

pub fn get_financing(env: &Env, id: u64) -> Financing {
    env.storage()
        .persistent()
        .get(&DataKey::Financing(id))
        .unwrap_or_else(|| panic_with_error!(env, FinancingError::FinancingNotFound))
}

pub fn set_funder_financings(env: &Env, funder: &Address, ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::FunderFinancings(funder.clone()), ids);
}

pub fn get_funder_financings(env: &Env, funder: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::FunderFinancings(funder.clone()))
        .unwrap_or_else(|| Vec::new(env))
}
