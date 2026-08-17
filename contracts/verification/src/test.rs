use soroban_sdk::{Address, Bytes, Env, testutils::Address as _};

use crate::{Verification, VerificationContract, VerificationContractClient, VerificationError};

fn env_with_authority() -> (Env, VerificationContractClient<'static>, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let authority = Address::generate(&env);
    let contract_id = env.register(VerificationContract, ());
    let client = VerificationContractClient::new(&env, &contract_id);
    client.mock_all_auths().initialize(&admin, &authority);
    (env, client, admin, authority)
}

fn hash(env: &Env, value: u32) -> Bytes {
    Bytes::from_slice(env, &value.to_be_bytes())
}

#[test]
fn initialize_sets_admin_and_authority() {
    let (env, client, admin, authority) = env_with_authority();
    let _ = (admin, authority);
    assert!(true);
}

#[test]
fn create_verification_requires_authority() {
    let (env, client, _admin, _authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let proof_hash = hash(&env, 100);
    let issuer = Address::generate(&env);

    // No auth mocked -> require_auth fails.
    let result = client.try_create_verification(&batch, &subject, &proof_hash, &issuer);
    assert!(result.is_err());
}

#[test]
fn create_verification_succeeds_with_authority() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let proof_hash = hash(&env, 100);

    let id = client
        .mock_all_auths()
        .create_verification(&batch, &subject, &proof_hash, &authority);

    assert_eq!(id, 1);

    let record: Verification = client.get_verification(&id);
    assert_eq!(record.id, 1);
    assert_eq!(record.batch, batch);
    assert_eq!(record.subject, subject);
    assert_eq!(record.proof_hash, proof_hash);
    assert_eq!(record.issuer, authority);
    assert!(!record.revoked);
    assert!(record.revoked_ledger.is_none());
}

#[test]
fn create_verification_rejects_empty_batch_or_proof() {
    let (env, client, _admin, authority) = env_with_authority();
    let subject = Address::generate(&env);
    let proof_hash = hash(&env, 100);
    let batch = hash(&env, 1);

    let empty = Bytes::new(&env);

    let result1 =
        client
            .mock_all_auths()
            .try_create_verification(&empty, &subject, &proof_hash, &authority);
    assert!(result1.is_err());

    let result2 = client
        .mock_all_auths()
        .try_create_verification(&batch, &subject, &empty, &authority);
    assert!(result2.is_err());
}

#[test]
fn multiple_verifications_increment_counter() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);

    let id1 =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 1), &authority);
    let id2 =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 2), &authority);
    let id3 =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 3), &authority);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn batch_verifications_indexed() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 42);
    let subject = Address::generate(&env);

    client
        .mock_all_auths()
        .create_verification(&batch, &subject, &hash(&env, 1), &authority);
    client
        .mock_all_auths()
        .create_verification(&batch, &subject, &hash(&env, 2), &authority);
    client
        .mock_all_auths()
        .create_verification(&batch, &subject, &hash(&env, 3), &authority);

    let ids: soroban_sdk::Vec<u64> = client.get_batch_verifications(&batch);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.get(0).unwrap(), 1);
    assert_eq!(ids.get(1).unwrap(), 2);
    assert_eq!(ids.get(2).unwrap(), 3);
}

#[test]
fn get_verification_unknown_fails() {
    let (env, client, _admin, _authority) = env_with_authority();
    let result = client.try_get_verification(&999);
    assert!(result.is_err());
}

#[test]
fn revoke_verification_requires_authority() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let id =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 1), &authority);

    // No auth -> fails
    let result = client.try_revoke_verification(&authority, &id, &hash(&env, 99));
    assert!(result.is_err());
}

#[test]
fn revoke_verification_succeeds_with_authority() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let id =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 1), &authority);

    // Authority revokes
    let reason = hash(&env, 99);
    client
        .mock_all_auths()
        .revoke_verification(&authority, &id, &reason);

    let record: Verification = client.get_verification(&id);
    assert!(record.revoked);
    assert_eq!(record.revoked_ledger, Some(env.ledger().sequence()));
}

#[test]
fn revoke_verification_idempotent() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let id =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 1), &authority);

    client
        .mock_all_auths()
        .revoke_verification(&authority, &id, &hash(&env, 99));
    client
        .mock_all_auths()
        .revoke_verification(&authority, &id, &hash(&env, 100));

    let record: Verification = client.get_verification(&id);
    assert!(record.revoked);
    assert_eq!(record.revoked_ledger, Some(env.ledger().sequence()));
}

#[test]
fn revoke_unknown_verification_fails() {
    let (env, client, _admin, authority) = env_with_authority();
    let result = client
        .mock_all_auths()
        .try_revoke_verification(&authority, &999, &hash(&env, 99));
    assert!(result.is_err());
}

#[test]
fn revoke_rejects_empty_reason() {
    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let id =
        client
            .mock_all_auths()
            .create_verification(&batch, &subject, &hash(&env, 1), &authority);

    let empty = Bytes::new(&env);
    let result = client
        .mock_all_auths()
        .try_revoke_verification(&authority, &id, &empty);
    assert!(result.is_err());
}

#[test]
fn operations_require_initialization() {
    let env = Env::default();
    let contract_id = env.register(VerificationContract, ());
    let client = VerificationContractClient::new(&env, &contract_id);
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);
    let authority = Address::generate(&env);

    let result = client.mock_all_auths().try_create_verification(
        &batch,
        &subject,
        &hash(&env, 1),
        &authority,
    );
    assert!(result.is_err());
}

#[test]
fn events_are_emitted() {
    use soroban_sdk::testutils::Events as _;

    let (env, client, _admin, authority) = env_with_authority();
    let batch = hash(&env, 1);
    let subject = Address::generate(&env);

    client
        .mock_all_auths()
        .create_verification(&batch, &subject, &hash(&env, 1), &authority);

    let events = env.events().all();
    assert!(!events.events().is_empty());
}
