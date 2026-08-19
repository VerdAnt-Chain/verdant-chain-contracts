#![no_std]
use soroban_sdk::{Address, Bytes, Env, Vec, contract, contractimpl, panic_with_error};

mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

pub use errors::FinancingError;
pub use events::{
    FinancingCreated, FinancingDeposited, FinancingRefunded, FinancingReleased, initialized,
};
pub use types::{Financing, Milestone};

#[contract]
pub struct FinancingContract;

#[contractimpl]
impl FinancingContract {
    pub fn initialize(env: Env, admin: Address, financing_authority: Address) {
        admin.require_auth();
        financing_authority.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_financing_authority(&env, &financing_authority);
        storage::set_next_id(&env, 1);
        events::initialized(&env, &admin);
    }

    pub fn create_financing(
        env: Env,
        funder: Address,
        beneficiary: Address,
        total_amount: i128,
        milestone_count: u32,
        milestones: Vec<Milestone>,
    ) -> u64 {
        funder.require_auth();
        let auth = storage::get_financing_authority(&env);
        auth.require_auth();

        if total_amount <= 0 || milestone_count == 0 || milestones.is_empty() {
            panic_with_error!(&env, FinancingError::InvalidInput);
        }

        let id = storage::next_id(&env);
        let financing = Financing {
            id,
            funder: funder.clone(),
            beneficiary: beneficiary.clone(),
            total_amount,
            drawn_amount: 0,
            milestone_count,
            milestones,
            drawn_ledger: env.ledger().sequence(),
            repaid_amount: 0,
            defaulted: false,
            defaulted_ledger: None,
        };

        storage::set_financing(&env, id, &financing);

        let mut funder_financings = storage::get_funder_financings(&env, &funder);
        funder_financings.push_back(id);
        storage::set_funder_financings(&env, &funder, &funder_financings);

        storage::set_next_id(&env, id + 1);

        events::financing_created(&env, id, &funder, &financing);
        id
    }

    pub fn deposit(env: Env, financing_id: u64, from: Address, amount: i128) {
        from.require_auth();
        let auth = storage::get_financing_authority(&env);
        auth.require_auth();

        let mut financing = storage::get_financing(&env, financing_id);

        if amount <= 0 || financing.drawn_amount + amount > financing.total_amount {
            panic_with_error!(&env, FinancingError::InvalidInput);
        }

        financing.drawn_amount += amount;
        financing.drawn_ledger = env.ledger().sequence();
        storage::set_financing(&env, financing_id, &financing);

        events::financing_deposited(&env, financing_id, &from, amount, financing.drawn_amount);
    }

    pub fn release_on_milestone(
        env: Env,
        financing_id: u64,
        proof: Bytes,
        milestone_index: u32,
    ) -> i128 {
        let auth = storage::get_financing_authority(&env);
        auth.require_auth();

        let mut financing = storage::get_financing(&env, financing_id);

        let Some(milestone) = financing.milestones.get(milestone_index) else {
            panic_with_error!(&env, FinancingError::InvalidInput);
        };

        if env.ledger().sequence() > milestone.deadline_ledger {
            panic_with_error!(&env, FinancingError::MilestoneDeadlineExceeded);
        }

        if proof.is_empty() {
            panic_with_error!(&env, FinancingError::InvalidInput);
        }

        let release_amount = milestone.proof_amount;
        let remaining = financing.total_amount - financing.drawn_amount;
        let actual_release = if release_amount > 0 && release_amount <= remaining {
            release_amount
        } else if release_amount < 0 && -release_amount <= remaining {
            -release_amount
        } else {
            0
        };

        financing.drawn_amount += actual_release;
        financing.drawn_ledger = env.ledger().sequence();
        storage::set_financing(&env, financing_id, &financing);

        events::financing_released(&env, financing_id, &auth, actual_release);
        actual_release
    }

    pub fn refund(env: Env, financing_id: u64, refundee: Address) {
        let auth = storage::get_financing_authority(&env);
        auth.require_auth();

        let mut financing = storage::get_financing(&env, financing_id);

        let refundable = financing.total_amount - financing.drawn_amount;
        if refundable <= 0 {
            panic_with_error!(&env, FinancingError::InvalidInput);
        }

        let is_default = financing
            .milestones
            .iter()
            .any(|m| env.ledger().sequence() > m.deadline_ledger);

        if is_default {
            financing.defaulted = true;
            financing.defaulted_ledger = Some(env.ledger().sequence());
        }

        financing.repaid_amount += refundable;
        storage::set_financing(&env, financing_id, &financing);

        events::financing_refunded(
            &env,
            financing_id,
            &refundee,
            refundable,
            financing.defaulted,
            financing.defaulted_ledger.unwrap_or(0),
        );
    }

    pub fn get_financing(env: Env, financing_id: u64) -> Financing {
        storage::get_financing(&env, financing_id)
    }

    pub fn get_financings_by_funder(env: Env, funder: Address) -> Vec<u64> {
        storage::get_funder_financings(&env, &funder)
    }
}
