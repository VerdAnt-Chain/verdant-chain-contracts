use soroban_sdk::{Address, contractevent};

use crate::types::{Farmer, VerificationMarker};

#[contractevent]
pub struct Initialized {
    #[topic]
    admin: Address,
}

#[contractevent]
pub struct FarmerRegistered {
    #[topic]
    farmer: Address,
    record: Farmer,
}

#[contractevent]
pub struct FarmerMetadataUpdated {
    #[topic]
    farmer: Address,
    record: Farmer,
}

#[contractevent]
pub struct VerificationMarkerSet {
    #[topic]
    farmer: Address,
    marker: VerificationMarker,
}

pub fn initialized(env: &soroban_sdk::Env, admin: &Address) {
    env.events().publish_event(&Initialized {
        admin: admin.clone(),
    });
}

pub fn farmer_registered(env: &soroban_sdk::Env, farmer: &Address, record: &Farmer) {
    env.events().publish_event(&FarmerRegistered {
        farmer: farmer.clone(),
        record: record.clone(),
    });
}

pub fn metadata_updated(env: &soroban_sdk::Env, farmer: &Address, record: &Farmer) {
    env.events().publish_event(&FarmerMetadataUpdated {
        farmer: farmer.clone(),
        record: record.clone(),
    });
}

pub fn verification_marker_set(
    env: &soroban_sdk::Env,
    farmer: &Address,
    marker: &VerificationMarker,
) {
    env.events().publish_event(&VerificationMarkerSet {
        farmer: farmer.clone(),
        marker: marker.clone(),
    });
}
