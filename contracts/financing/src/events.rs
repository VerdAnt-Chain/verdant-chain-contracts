use soroban_sdk::{Address, contractevent};

use crate::Financing;

#[contractevent]
pub struct Initialized {
    #[topic]
    admin: Address,
}

#[contractevent]
pub struct FinancingCreated {
    #[topic]
    financing_id: u64,
    #[topic]
    funder: Address,
    financing: Financing,
}

#[contractevent]
pub struct FinancingDeposited {
    #[topic]
    financing_id: u64,
    #[topic]
    from: Address,
    amount: i128,
    new_drawn_amount: i128,
}

#[contractevent]
pub struct FinancingReleased {
    #[topic]
    financing_id: u64,
    #[topic]
    releaser: Address,
    released_amount: i128,
}

#[contractevent]
pub struct FinancingRefunded {
    #[topic]
    financing_id: u64,
    #[topic]
    refundee: Address,
    refunded_amount: i128,
    defaulted: bool,
    defaulted_ledger: u32,
}

pub fn initialized(env: &soroban_sdk::Env, admin: &Address) {
    env.events().publish_event(&Initialized {
        admin: admin.clone(),
    });
}

pub fn financing_created(
    env: &soroban_sdk::Env,
    financing_id: u64,
    funder: &Address,
    financing: &Financing,
) {
    env.events().publish_event(&FinancingCreated {
        financing_id,
        funder: funder.clone(),
        financing: financing.clone(),
    });
}

pub fn financing_deposited(
    env: &soroban_sdk::Env,
    financing_id: u64,
    from: &Address,
    amount: i128,
    new_drawn_amount: i128,
) {
    env.events().publish_event(&FinancingDeposited {
        financing_id,
        from: from.clone(),
        amount,
        new_drawn_amount,
    });
}

pub fn financing_released(
    env: &soroban_sdk::Env,
    financing_id: u64,
    releaser: &Address,
    released_amount: i128,
) {
    env.events().publish_event(&FinancingReleased {
        financing_id,
        releaser: releaser.clone(),
        released_amount,
    });
}

pub fn financing_refunded(
    env: &soroban_sdk::Env,
    financing_id: u64,
    refundee: &Address,
    refunded_amount: i128,
    defaulted: bool,
    defaulted_ledger: u32,
) {
    env.events().publish_event(&FinancingRefunded {
        financing_id,
        refundee: refundee.clone(),
        refunded_amount,
        defaulted,
        defaulted_ledger,
    });
}
