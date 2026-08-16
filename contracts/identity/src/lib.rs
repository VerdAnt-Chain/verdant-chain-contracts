#![no_std]
use soroban_sdk::{Address, Bytes, Env, contract, contractimpl, panic_with_error};

mod errors;
mod events;
mod storage;
mod types;

pub use errors::IdentityError;
pub use events::{FarmerMetadataUpdated, FarmerRegistered, Initialized, VerificationMarkerSet};
pub use types::{Farmer, VerificationMarker};

#[cfg(test)]
mod test;

#[contract]
pub struct IdentityContract;

#[contractimpl]
impl IdentityContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        storage::set_admin(&env, &admin);
        events::initialized(&env, &admin);
    }

    pub fn register_farmer(env: Env, farmer: Address, metadata_hash: Bytes) -> Farmer {
        farmer.require_auth();
        storage::require_initialized(&env);
        if storage::has_farmer(&env, &farmer) {
            panic_with_error!(&env, IdentityError::AlreadyRegistered);
        }
        let record = Farmer {
            address: farmer.clone(),
            metadata_hash,
            verification_markers: soroban_sdk::Vec::new(&env),
            created_ledger: env.ledger().sequence(),
            updated_ledger: env.ledger().sequence(),
        };
        storage::set_farmer(&env, &farmer, &record);
        events::farmer_registered(&env, &farmer, &record);
        record
    }

    pub fn update_metadata(env: Env, farmer: Address, metadata_hash: Bytes) -> Farmer {
        farmer.require_auth();
        storage::require_initialized(&env);
        let mut record = storage::require_farmer(&env, &farmer);
        record.metadata_hash = metadata_hash;
        record.updated_ledger = env.ledger().sequence();
        storage::set_farmer(&env, &farmer, &record);
        events::metadata_updated(&env, &farmer, &record);
        record
    }

    pub fn set_verification_marker(
        env: Env,
        admin: Address,
        farmer: Address,
        marker: VerificationMarker,
    ) {
        admin.require_auth();
        storage::require_admin(&env, &admin);
        let mut record = storage::require_farmer(&env, &farmer);
        record.verification_markers.push_back(marker.clone());
        record.updated_ledger = env.ledger().sequence();
        storage::set_farmer(&env, &farmer, &record);
        events::verification_marker_set(&env, &farmer, &marker);
    }

    pub fn get_farmer(env: Env, farmer: Address) -> Farmer {
        storage::require_farmer(&env, &farmer)
    }

    pub fn is_registered(env: Env, farmer: Address) -> bool {
        storage::has_farmer(&env, &farmer)
    }
}
