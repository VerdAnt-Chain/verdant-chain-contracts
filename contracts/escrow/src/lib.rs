#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, Env, token::TokenClient};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::EscrowError;
pub use events::{Initialized, EscrowCreated, EscrowDeposited, EscrowReleased, EscrowRefunded};
pub use types::{Escrow, ReleaseCondition};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
        storage::set_next_id(&env, 1);
        events::initialized(&env, &admin);
    }

    pub fn create_escrow(
        env: Env,
        depositor: Address,
        beneficiary: Address,
        amount: i128,
        condition: ReleaseCondition,
        booking_ref: Bytes,
    ) -> u64 {
        depositor.require_auth();
        storage::require_initialized(&env);

        if amount <= 0 || booking_ref.is_empty() {
            panic_with_error!(&env, EscrowError::InvalidInput);
        }

        let id = storage::get_next_id(&env);
        let token = storage::get_token(&env);
        let token_client = TokenClient::new(&env, &token);

        // Pull funds from depositor
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let record = Escrow {
            id,
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token,
            amount,
            released_amount: 0,
            booking_ref: booking_ref.clone(),
            condition,
            created_ledger: env.ledger().sequence(),
            updated_ledger: env.ledger().sequence(),
        };

        storage::set_escrow(&env, id, &record);
        storage::set_next_id(&env, id + 1);

        let mut booking_ids = storage::get_booking_escrows(&env, &booking_ref);
        booking_ids.push_back(id);
        storage::set_booking_escrows(&env, &booking_ref, &booking_ids);

        events::escrow_created(&env, id, &depositor, &record);
        id
    }

    pub fn deposit(env: Env, escrow_id: u64, from: Address, amount: i128) {
        from.require_auth();
        storage::require_initialized(&env);

        if amount <= 0 {
            panic_with_error!(&env, EscrowError::InvalidInput);
        }

        let mut record = storage::get_escrow(&env, escrow_id);
        let token = storage::get_token(&env);
        let token_client = TokenClient::new(&env, &token);

        // Pull additional funds
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        record.amount += amount;
        record.updated_ledger = env.ledger().sequence();
        storage::set_escrow(&env, escrow_id, &record);

        events::escrow_deposited(&env, escrow_id, &from, amount);
    }

    pub fn release(env: Env, escrow_id: u64, releaser: Address, proof_hash: Bytes) {
        releaser.require_auth();
        storage::require_initialized(&env);

        let mut record = storage::get_escrow(&env, escrow_id);
        let remaining = record.amount - record.released_amount;

        if remaining <= 0 {
            panic_with_error!(&env, EscrowError::AlreadyFullyReleased);
        }

        // Verify authorization based on release condition
        match record.condition.kind {
            0 => { // Manual
                if &releaser != &record.condition.releaser {
                    panic_with_error!(&env, EscrowError::ConditionNotMet);
                }
            }
            1 => { // Milestone
                if &releaser != &record.condition.releaser {
                    panic_with_error!(&env, EscrowError::ConditionNotMet);
                }
                if proof_hash.is_empty() {
                    panic_with_error!(&env, EscrowError::InvalidInput);
                }
            }
            2 => { // Timeout
                if env.ledger().sequence() < record.condition.timeout_ledger {
                    panic_with_error!(&env, EscrowError::TimeoutNotElapsed);
                }
            }
            _ => panic_with_error!(&env, EscrowError::InvalidInput),
        }

        let token = storage::get_token(&env);
        let token_client = TokenClient::new(&env, &token);

        // Transfer remaining amount to beneficiary
        token_client.transfer(&env.current_contract_address(), &record.beneficiary, &remaining);

        record.released_amount += remaining;
        record.updated_ledger = env.ledger().sequence();
        storage::set_escrow(&env, escrow_id, &record);

        events::escrow_released(&env, escrow_id, &releaser, remaining);
    }

    pub fn refund(env: Env, escrow_id: u64, refundee: Address) {
        refundee.require_auth();
        storage::require_initialized(&env);

        let mut record = storage::get_escrow(&env, escrow_id);

        if record.released_amount > 0 {
            // Only allow refund if nothing has been released yet, or if timeout condition met
            if record.condition.kind == 2 {
                if env.ledger().sequence() < record.condition.timeout_ledger {
                    panic_with_error!(&env, EscrowError::TimeoutNotElapsed);
                }
            } else {
                panic_with_error!(&env, EscrowError::ConditionNotMet);
            }
        }

        let remaining = record.amount - record.released_amount;
        if remaining <= 0 {
            panic_with_error!(&env, EscrowError::AlreadyFullyReleased);
        }

        if &refundee != &record.depositor {
            panic_with_error!(&env, EscrowError::Unauthorized);
        }

        let token = storage::get_token(&env);
        let token_client = TokenClient::new(&env, &token);

        // Return remaining to depositor
        token_client.transfer(&env.current_contract_address(), &refundee, &remaining);

        record.released_amount = record.amount; // Mark as fully refunded
        record.updated_ledger = env.ledger().sequence();
        storage::set_escrow(&env, escrow_id, &record);

        events::escrow_refunded(&env, escrow_id, &refundee, remaining);
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> crate::types::Escrow {
        storage::get_escrow(&env, escrow_id)
    }

    pub fn get_escrows_for_booking(env: Env, booking_ref: Bytes) -> soroban_sdk::Vec<u64> {
        storage::get_booking_escrows(&env, &booking_ref)
    }
}