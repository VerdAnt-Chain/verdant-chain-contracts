use soroban_sdk::{Address, Bytes, Env, Error, testutils::Address as _};

use crate::{Farmer, IdentityContract, IdentityContractClient, IdentityError, VerificationMarker};

fn env_with_admin() -> (Env, IdentityContractClient<'static>, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);
    client.mock_all_auths().initialize(&admin);
    (env, client, admin)
}

fn hash(env: &Env, value: u32) -> Bytes {
    Bytes::from_slice(env, &value.to_be_bytes())
}

fn marker(env: &Env, kind: &[u8], issuer: &Address) -> VerificationMarker {
    VerificationMarker {
        kind: Bytes::from_slice(env, kind),
        issuer: issuer.clone(),
        issued_ledger: env.ledger().sequence(),
    }
}

#[test]
fn register_farmer_creates_record() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    let metadata_hash = hash(&env, 1);

    let record = client
        .mock_all_auths()
        .register_farmer(&farmer, &metadata_hash);

    assert_eq!(record.address, farmer);
    assert_eq!(record.metadata_hash, metadata_hash);
    assert_eq!(record.verification_markers.len(), 0);
    assert!(client.is_registered(&farmer));
}

#[test]
fn register_farmer_requires_auth() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    let metadata_hash = hash(&env, 1);

    // No auth mocked -> require_auth fails.
    let result = client.try_register_farmer(&farmer, &metadata_hash);
    assert!(result.is_err());
}

#[test]
fn cannot_register_same_farmer_twice() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    let metadata_hash = hash(&env, 1);

    client
        .mock_all_auths()
        .register_farmer(&farmer, &metadata_hash);
    let result = client
        .mock_all_auths()
        .try_register_farmer(&farmer, &metadata_hash);

    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            IdentityError::AlreadyRegistered as u32
        )))
    );
}

#[test]
fn update_metadata_allowed_for_farmer() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    let first = hash(&env, 1);
    let second = hash(&env, 2);

    client.mock_all_auths().register_farmer(&farmer, &first);

    let updated = client.mock_all_auths().update_metadata(&farmer, &second);
    assert_eq!(updated.metadata_hash, second);
    let record = client.get_farmer(&farmer);
    assert_eq!(record.metadata_hash, second);
}

#[test]
fn update_metadata_rejects_other_caller() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    client
        .mock_all_auths()
        .register_farmer(&farmer, &hash(&env, 1));

    let stranger = Address::generate(&env);
    // mock_all_auths lets every require_auth pass, so the SDK surfaces a panic
    // because the caller is not the farmer's own address.
    let result = client
        .mock_all_auths()
        .try_update_metadata(&stranger, &hash(&env, 2));
    assert!(result.is_err());
}

#[test]
fn set_verification_marker_is_admin_only() {
    let (env, client, admin) = env_with_admin();
    let farmer = Address::generate(&env);
    client
        .mock_all_auths()
        .register_farmer(&farmer, &hash(&env, 1));

    let m = marker(&env, b"certified", &admin);

    // Admin can set a marker.
    client
        .mock_all_auths()
        .set_verification_marker(&admin, &farmer, &m);
    let record = client.get_farmer(&farmer);
    assert_eq!(record.verification_markers.len(), 1);
    assert_eq!(record.verification_markers.get(0).unwrap().kind, m.kind);

    // A non-admin caller is rejected.
    let non_admin = Address::generate(&env);
    let m2 = marker(&env, b"fake", &non_admin);
    let result = client
        .mock_all_auths()
        .try_set_verification_marker(&non_admin, &farmer, &m2);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            IdentityError::Unauthorized as u32
        )))
    );
}

#[test]
fn get_farmer_returns_record() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    let metadata_hash = hash(&env, 7);
    client
        .mock_all_auths()
        .register_farmer(&farmer, &metadata_hash);

    let record: Farmer = client.get_farmer(&farmer);
    assert_eq!(record.address, farmer);
    assert_eq!(record.metadata_hash, metadata_hash);
}

#[test]
fn get_farmer_unknown_panics() {
    let (env, client, _admin) = env_with_admin();
    let stranger = Address::generate(&env);
    let result = client.try_get_farmer(&stranger);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            IdentityError::FarmerNotFound as u32
        )))
    );
}

#[test]
fn is_registered_false_before_registration() {
    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    assert!(!client.is_registered(&farmer));
}

#[test]
fn operations_require_initialization() {
    let env = Env::default();
    let contract_id = env.register(IdentityContract, ());
    let client = IdentityContractClient::new(&env, &contract_id);
    let farmer = Address::generate(&env);

    let result = client
        .mock_all_auths()
        .try_register_farmer(&farmer, &hash(&env, 1));
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            IdentityError::NotInitialized as u32
        )))
    );
}

#[test]
fn events_are_emitted() {
    use soroban_sdk::testutils::Events as _;

    let (env, client, _admin) = env_with_admin();
    let farmer = Address::generate(&env);
    client
        .mock_all_auths()
        .register_farmer(&farmer, &hash(&env, 1));

    let events = env.events().all();
    assert!(!events.events().is_empty());
}
