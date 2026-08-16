use soroban_sdk::{Address, Env, contracttype, panic_with_error};

use crate::errors::IdentityError;
use crate::types::Farmer;

#[contracttype]
enum DataKey {
    Admin,
    Farmer(Address),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .expect("admin not set")
}

pub fn require_initialized(env: &Env) {
    if !env.storage().persistent().has(&DataKey::Admin) {
        panic_with_error!(env, IdentityError::NotInitialized);
    }
}

pub fn require_admin(env: &Env, caller: &Address) {
    let admin = get_admin(env);
    if caller != &admin {
        panic_with_error!(env, IdentityError::Unauthorized);
    }
}

pub fn has_farmer(env: &Env, farmer: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Farmer(farmer.clone()))
}

pub fn set_farmer(env: &Env, farmer: &Address, record: &Farmer) {
    env.storage()
        .persistent()
        .set(&DataKey::Farmer(farmer.clone()), record);
}

pub fn require_farmer(env: &Env, farmer: &Address) -> Farmer {
    env.storage()
        .persistent()
        .get(&DataKey::Farmer(farmer.clone()))
        .unwrap_or_else(|| panic_with_error!(env, IdentityError::FarmerNotFound))
}
