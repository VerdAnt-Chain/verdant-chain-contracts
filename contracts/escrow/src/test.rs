use soroban_sdk::{
    Address, Bytes, Env, testutils::Address as _, testutils::Ledger, token::StellarAssetClient,
};

use crate::{EscrowContract, EscrowContractClient};

fn setup(
    amount: i128,
) -> (
    Env,
    EscrowContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = StellarAssetClient::new(&env, &token.address());
    token_client.mock_all_auths().mint(&depositor, &amount);

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);
    client.mock_all_auths().initialize(&admin, &token.address());

    (env, client, depositor, beneficiary, token.address())
}

fn condition(kind: u32, releaser: Address, timeout_ledger: u32) -> crate::ReleaseCondition {
    crate::ReleaseCondition {
        kind,
        releaser,
        timeout_ledger,
    }
}

#[test]
fn test_create_escrow() {
    let (env, client, depositor, beneficiary, token) = setup(1000);
    let amount = 1000i128;
    let condition = condition(0, depositor.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);

    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &amount,
        &condition,
        &booking_ref,
    );

    let record = client.get_escrow(&escrow_id);
    assert_eq!(record.depositor, depositor);
    assert_eq!(record.beneficiary, beneficiary);
    assert_eq!(record.amount, amount);
    assert_eq!(record.released_amount, 0);
    assert_eq!(record.token, token);
    assert_eq!(record.condition.kind, 0);
    assert_eq!(record.condition.releaser, depositor);
}

#[test]
fn test_create_escrow_rejects_empty_booking_ref() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let condition = condition(0, depositor.clone(), 0);
    let empty = Bytes::from_slice(&env, &[]);

    let result = client.mock_all_auths().try_create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &empty,
    );
    assert!(result.is_err());
}

#[test]
fn test_create_escrow_rejects_nonpositive_amount() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let condition = condition(0, depositor.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);

    let result = client.mock_all_auths().try_create_escrow(
        &depositor,
        &beneficiary,
        &0i128,
        &condition,
        &booking_ref,
    );
    assert!(result.is_err());
}

#[test]
fn test_deposit_adds_funds() {
    let (env, client, depositor, beneficiary, token) = setup(1000);
    let condition = condition(0, depositor.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let token_client = StellarAssetClient::new(&env, &token);
    token_client.mock_all_auths().mint(&depositor, &500);

    client
        .mock_all_auths()
        .deposit(&escrow_id, &depositor, &500);

    let record = client.get_escrow(&escrow_id);
    assert_eq!(record.amount, 1500);
}

#[test]
fn test_release_manual_condition() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let condition = condition(0, depositor.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 0, 0, 0]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    client
        .mock_all_auths()
        .release(&escrow_id, &depositor, &Bytes::from_slice(&env, &[]));

    let record = client.get_escrow(&escrow_id);
    assert_eq!(record.released_amount, 1000);
}

#[test]
fn test_release_manual_condition_rejects_unauthorized_releaser() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let stranger = Address::generate(&env);
    let condition = condition(0, depositor.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 0, 0, 0]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let result =
        client
            .mock_all_auths()
            .try_release(&escrow_id, &stranger, &Bytes::from_slice(&env, &[]));
    assert!(result.is_err());
}

#[test]
fn test_release_timeout_condition() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let condition = condition(2, depositor.clone(), 100);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let seq = env.ledger().sequence();
    env.ledger().set_sequence_number(seq + 100);

    client
        .mock_all_auths()
        .release(&escrow_id, &depositor, &Bytes::from_slice(&env, &[]));

    let record = client.get_escrow(&escrow_id);
    assert_eq!(record.released_amount, 1000);
}

#[test]
fn test_release_timeout_not_elapsed() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let condition = condition(2, depositor.clone(), 100);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let result =
        client
            .mock_all_auths()
            .try_release(&escrow_id, &depositor, &Bytes::from_slice(&env, &[]));
    assert!(result.is_err());
}

#[test]
fn test_release_milestone_condition() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let proof_verifier = Address::generate(&env);
    let condition = condition(1, proof_verifier.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    client.mock_all_auths().release(
        &escrow_id,
        &proof_verifier,
        &Bytes::from_slice(&env, &[42; 32]),
    );

    let record = client.get_escrow(&escrow_id);
    assert_eq!(record.released_amount, 1000);
}

#[test]
fn test_release_milestone_rejects_empty_proof() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let proof_verifier = Address::generate(&env);
    let condition = condition(1, proof_verifier.clone(), 0);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let result = client.mock_all_auths().try_release(
        &escrow_id,
        &proof_verifier,
        &Bytes::from_slice(&env, &[]),
    );
    assert!(result.is_err());
}

#[test]
fn test_refund_with_timeout() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let condition = condition(2, depositor.clone(), 100);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let seq = env.ledger().sequence();
    env.ledger().set_sequence_number(seq + 100);

    client.mock_all_auths().refund(&escrow_id, &depositor);

    let record = client.get_escrow(&escrow_id);
    assert_eq!(record.released_amount, 1000);
}

#[test]
fn test_refund_rejects_non_depositor() {
    let (env, client, depositor, beneficiary, _token) = setup(1000);
    let stranger = Address::generate(&env);
    let condition = condition(2, depositor.clone(), 100);
    let booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition,
        &booking_ref,
    );

    let seq = env.ledger().sequence();
    env.ledger().set_sequence_number(seq + 100);

    let result = client.mock_all_auths().try_refund(&escrow_id, &stranger);
    assert!(result.is_err());
}

#[test]
fn test_get_escrows_for_booking() {
    let (env, client, depositor, beneficiary, _token) = setup(3000);
    let condition1 = condition(0, depositor.clone(), 0);
    let condition2 = condition(0, depositor.clone(), 0);

    let escrow_id1 = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &1000i128,
        &condition1,
        &Bytes::from_slice(&env, &[1, 0, 0, 0]),
    );
    let escrow_id2 = client.mock_all_auths().create_escrow(
        &depositor,
        &beneficiary,
        &2000i128,
        &condition2,
        &Bytes::from_slice(&env, &[2, 0, 0, 0]),
    );

    let ids1 = client.get_escrows_for_booking(&Bytes::from_slice(&env, &[1, 0, 0, 0]));
    assert_eq!(ids1.len(), 1);
    assert_eq!(ids1.get(0).unwrap(), escrow_id1);

    let ids2 = client.get_escrows_for_booking(&Bytes::from_slice(&env, &[2, 0, 0, 0]));
    assert_eq!(ids2.len(), 1);
    assert_eq!(ids2.get(0).unwrap(), escrow_id2);
}

#[test]
#[should_panic]
fn test_get_escrow_not_found() {
    let (_env, client, _depositor, _beneficiary, _token) = setup(1000);
    client.get_escrow(&999);
}
