#![no_std]
use soroban_sdk::{Address, Bytes, Env, contract, contractimpl, panic_with_error};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::VerificationError;
pub use events::{Initialized, VerificationCreated, VerificationRevoked};
pub use types::{Verification, VerificationIdList};

#[cfg(test)]
mod test;

#[contract]
pub struct VerificationContract;

#[contractimpl]
impl VerificationContract {
    pub fn initialize(env: Env, admin: Address, verification_authority: Address) {
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_verification_authority(&env, &verification_authority);
        storage::set_next_id(&env, 1);
        events::initialized(&env, &admin);
    }

    pub fn create_verification(
        env: Env,
        batch: Bytes,
        subject: Address,
        proof_hash: Bytes,
        issuer: Address,
    ) -> u64 {
        storage::require_initialized(&env);
        storage::require_authority(&env, &issuer);

        if batch.is_empty() || proof_hash.is_empty() {
            panic_with_error!(&env, VerificationError::InvalidInput);
        }

        let id = storage::get_next_id(&env);
        let record = Verification {
            id,
            batch: batch.clone(),
            subject: subject.clone(),
            proof_hash,
            issuer: issuer.clone(),
            issued_ledger: env.ledger().sequence(),
            revoked: false,
            revoked_ledger: None,
        };

        storage::set_verification(&env, id, &record);
        storage::set_next_id(&env, id + 1);

        let mut batch_ids = storage::get_batch_verifications(&env, &batch);
        batch_ids.push_back(id);
        storage::set_batch_verifications(&env, &batch, &batch_ids);

        events::verification_created(&env, id, &record);
        id
    }

    pub fn revoke_verification(
        env: Env,
        authority: Address,
        verification_id: u64,
        reason_hash: Bytes,
    ) {
        authority.require_auth();
        storage::require_initialized(&env);
        storage::require_authority(&env, &authority);

        let mut record = storage::get_verification(&env, verification_id);
        if record.revoked {
            // Idempotent: already revoked, no-op
            return;
        }
        if reason_hash.is_empty() {
            panic_with_error!(&env, VerificationError::InvalidInput);
        }

        record.revoked = true;
        record.revoked_ledger = Some(env.ledger().sequence());
        storage::set_verification(&env, verification_id, &record);
        events::verification_revoked(&env, verification_id, &record);
    }

    pub fn get_verification(env: Env, verification_id: u64) -> Verification {
        storage::get_verification(&env, verification_id)
    }

    pub fn get_batch_verifications(env: Env, batch: Bytes) -> soroban_sdk::Vec<u64> {
        storage::get_batch_verifications(&env, &batch)
    }
}
