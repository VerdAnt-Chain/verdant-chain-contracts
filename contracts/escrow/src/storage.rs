use soroban_sdk::{contracttype, panic_with_error, Address, Bytes, Env};

use crate::errors::EscrowError;
use crate::types::Escrow;

#[contracttype]
enum DataKey {
    Admin,
    Token,
    NextEscrowId,
    Escrow(u64),
    BookingEscrows(Bytes),
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

pub fn set_token(env: &Env, token: &Address) {
    env.storage().persistent().set(&DataKey::Token, token);
}

pub fn get_token(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Token)
        .expect("token not set")
}

pub fn require_initialized(env: &Env) {
    if !env.storage().persistent().has(&DataKey::Admin) {
        panic_with_error!(env, EscrowError::NotInitialized);
    }
}

pub fn get_next_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextEscrowId)
        .unwrap_or(1)
}

pub fn set_next_id(env: &Env, id: u64) {
    env.storage().persistent().set(&DataKey::NextEscrowId, &id);
}

pub fn set_escrow(env: &Env, id: u64, record: &Escrow) {
    env.storage()
        .persistent()
        .set(&DataKey::Escrow(id), record);
}

pub fn get_escrow(env: &Env, id: u64) -> crate::types::Escrow {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(id))
        .unwrap_or_else(|| panic_with_error!(env, crate::errors::EscrowError::EscrowNotFound))
}

pub fn set_booking_escrows(env: &Env, booking_ref: &Bytes, ids: &soroban_sdk::Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::BookingEscrows(booking_ref.clone()), ids);
}

pub fn get_booking_escrows(env: &Env, booking_ref: &Bytes) -> soroban_sdk::Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::BookingEscrows(booking_ref.clone()))
        .unwrap_or_else(|| soroban_sdk::Vec::new(env))
}