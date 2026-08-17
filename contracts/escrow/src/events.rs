use soroban_sdk::{contractevent, Address};

use crate::types::Escrow;

#[contractevent]
pub struct Initialized {
    #[topic]
    admin: Address,
}

#[contractevent]
pub struct EscrowCreated {
    #[topic]
    escrow_id: u64,
    #[topic]
    depositor: Address,
    record: Escrow,
}

#[contractevent]
pub struct EscrowDeposited {
    #[topic]
    escrow_id: u64,
    #[topic]
    from: Address,
    amount: i128,
}

#[contractevent]
pub struct EscrowReleased {
    #[topic]
    escrow_id: u64,
    #[topic]
    releaser: Address,
    amount: i128,
}

#[contractevent]
pub struct EscrowRefunded {
    #[topic]
    escrow_id: u64,
    #[topic]
    refundee: Address,
    amount: i128,
}

pub fn initialized(env: &soroban_sdk::Env, admin: &Address) {
    env.events().publish_event(&Initialized {
        admin: admin.clone(),
    });
}

pub fn escrow_created(env: &soroban_sdk::Env, id: u64, depositor: &Address, record: &Escrow) {
    env.events().publish_event(&EscrowCreated {
        escrow_id: id,
        depositor: depositor.clone(),
        record: record.clone(),
    });
}

pub fn escrow_deposited(env: &soroban_sdk::Env, id: u64, from: &Address, amount: i128) {
    env.events().publish_event(&EscrowDeposited {
        escrow_id: id,
        from: from.clone(),
        amount,
    });
}

pub fn escrow_released(env: &soroban_sdk::Env, id: u64, releaser: &Address, amount: i128) {
    env.events().publish_event(&EscrowReleased {
        escrow_id: id,
        releaser: releaser.clone(),
        amount,
    });
}

pub fn escrow_refunded(env: &soroban_sdk::Env, id: u64, refundee: &Address, amount: i128) {
    env.events().publish_event(&EscrowRefunded {
        escrow_id: id,
        refundee: refundee.clone(),
        amount,
    });
}