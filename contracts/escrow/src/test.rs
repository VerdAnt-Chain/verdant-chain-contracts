use soroban_sdk::{Address, Bytes, Env, testutils::Address as _};

use crate::EscrowContract;

#[test]
fn test_create_escrow() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    let condition = crate::ReleaseCondition {
        kind: 0,
        releaser: depositor.clone(),
        timeout_ledger: 0,
    };
    let _booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
    let escrow_id = EscrowContract::create_escrow(
        env.clone(),
        depositor,
        beneficiary,
        amount,
        condition,
        booking_ref,
    );

    let record = EscrowContract::get_escrow(env, escrow_id);
    assert_eq!(record.depositor, depositor);
    assert_eq!(record.beneficiary, beneficiary);
    assert_eq!(record.amount, amount);
    assert_eq!(record.released_amount, 0);
    assert_eq!(record.token, token);
    assert_eq!(record.condition.kind, 0);
    assert_eq!(record.condition.releaser, depositor);
}

#[test]
fn test_release_manual_condition() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    let condition = crate::ReleaseCondition {
        kind: 0,
        releaser: depositor.clone(),
        timeout_ledger: 0,
    };
    let _escrow_id = {
        let _booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
        EscrowContract::create_escrow(
            env.clone(),
            depositor,
            beneficiary,
            amount,
            condition,
            Bytes::from_slice(&env, &[1, 0, 0, 0]),
        )
    };

    EscrowContract::release(
        env.clone(),
        1,
        depositor.clone(),
        Bytes::from_slice(&env, &[]),
    );

    let record = EscrowContract::get_escrow(env, 1);
    assert_eq!(record.released_amount, amount);
}

#[test]
fn test_release_timeout_condition() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    let condition = crate::ReleaseCondition {
        kind: 2,
        releaser: depositor.clone(),
        timeout_ledger: 100,
    };
    let _escrow_id = {
        let _booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
        EscrowContract::create_escrow(
            env.clone(),
            depositor,
            beneficiary,
            amount,
            condition,
            Bytes::from_slice(&env, &[1, 0, 0, 0]),
        )
    };

    let seq = env.ledger().sequence();
    env.ledger().set_sequence_number(seq + 100);

    EscrowContract::release(
        env.clone(),
        1,
        depositor.clone(),
        Bytes::from_slice(&env, &[]),
    );

    let record = EscrowContract::get_escrow(env, 1);
    assert_eq!(record.released_amount, amount);
}

#[test]
fn test_release_milestone_condition() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    let proof_verifier = Address::generate(&env);
    let amount = 1000i128;

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    let condition = crate::ReleaseCondition {
        kind: 1,
        releaser: proof_verifier.clone(),
        timeout_ledger: 0,
    };
    let _escrow_id = {
        let _booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
        EscrowContract::create_escrow(
            env.clone(),
            depositor,
            beneficiary,
            amount,
            condition,
            Bytes::from_slice(&env, &[1, 0, 0, 0]),
        )
    };

    EscrowContract::release(
        env.clone(),
        1,
        proof_verifier.clone(),
        Bytes::from_slice(&env, &[42; 32]),
    );

    let record = EscrowContract::get_escrow(env, 1);
    assert_eq!(record.released_amount, amount);
}

#[test]
fn test_refund_with_timeout() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let depositor = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    let condition = crate::ReleaseCondition {
        kind: 2,
        releaser: depositor.clone(),
        timeout_ledger: 100,
    };
    let _escrow_id = {
        let _booking_ref = Bytes::from_slice(&env, &[1, 2, 3, 4]);
        EscrowContract::create_escrow(
            env.clone(),
            depositor,
            beneficiary,
            amount,
            condition,
            Bytes::from_slice(&env, &[1, 0, 0, 0]),
        )
    };

    let seq = env.ledger().sequence();
    env.ledger().set_sequence_number(seq + 100);

    EscrowContract::refund(env.clone(), 1, depositor.clone());

    let record = EscrowContract::get_escrow(env, 1);
    assert_eq!(record.released_amount, amount);
}

#[test]
fn test_get_escrows_for_booking() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    let condition1 = crate::ReleaseCondition {
        kind: 0,
        releaser: Address::generate(&env),
        timeout_ledger: 0,
    };
    let escrow_id1 = {
        EscrowContract::create_escrow(
            env.clone(),
            Address::generate(&env),
            Address::generate(&env),
            1000,
            condition1,
            Bytes::from_slice(&env, &[1, 0, 0, 0]),
        )
    };

    let condition2 = crate::ReleaseCondition {
        kind: 0,
        releaser: Address::generate(&env),
        timeout_ledger: 0,
    };
    let escrow_id2 = {
        EscrowContract::create_escrow(
            env.clone(),
            Address::generate(&env),
            Address::generate(&env),
            2000,
            condition2,
            Bytes::from_slice(&env, &[2, 0, 0, 0]),
        )
    };

    let ids1 = EscrowContract::get_escrows_for_booking(
        env.clone(),
        Bytes::from_slice(&env, &[1, 0, 0, 0]),
    );
    assert_eq!(ids1.len(), 1);
    assert_eq!(ids1.get(0).unwrap(), escrow_id1);

    let ids2 = EscrowContract::get_escrows_for_booking(
        env.clone(),
        Bytes::from_slice(&env, &[2, 0, 0, 0]),
    );
    assert_eq!(ids2.len(), 1);
    assert_eq!(ids2.get(0).unwrap(), escrow_id2);
}

#[test]
#[should_panic]
fn test_get_escrow_not_found() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    EscrowContract::initialize(env.clone(), admin.clone(), token.clone());

    EscrowContract::get_escrow(env, 999);
}
