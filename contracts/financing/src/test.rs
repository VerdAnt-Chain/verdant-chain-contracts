use soroban_sdk::{Address, Bytes, Env, Vec, testutils::Address as _, testutils::Ledger};

use crate::{FinancingContract, FinancingContractClient, Milestone};

fn setup() -> (
    Env,
    FinancingContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let authority = Address::generate(&env);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    let contract_id = env.register(FinancingContract, ());
    let client = FinancingContractClient::new(&env, &contract_id);
    client.mock_all_auths().initialize(&admin, &authority);

    (env, client, admin, funder, beneficiary)
}

fn create_financing(
    env: &Env,
    client: &FinancingContractClient<'static>,
    funder: &Address,
    beneficiary: &Address,
    deadline: u32,
) -> u64 {
    let milestones = Vec::from_slice(
        env,
        &[Milestone {
            index: 1,
            deadline_ledger: deadline,
            proof_hash: Bytes::from_slice(env, &[1; 32]),
            proof_amount: 600,
        }],
    );
    client
        .mock_all_auths()
        .create_financing(funder, beneficiary, &1000i128, &1u32, &milestones)
}

#[test]
fn test_create_financing() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();

    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    let record = client.get_financing(&id);
    assert_eq!(record.id, 1);
    assert_eq!(record.funder, funder);
    assert_eq!(record.beneficiary, beneficiary);
    assert_eq!(record.total_amount, 1000);
    assert_eq!(record.drawn_amount, 0);
    assert_eq!(record.milestone_count, 1);
    assert_eq!(record.milestones.len(), 1);
    assert!(!record.defaulted);
    assert_eq!(record.defaulted_ledger, None);
}

#[test]
fn test_create_financing_rejects_zero_total() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let milestones = Vec::from_slice(
        &env,
        &[Milestone {
            index: 1,
            deadline_ledger: seq + 1000,
            proof_hash: Bytes::from_slice(&env, &[1; 32]),
            proof_amount: 0,
        }],
    );

    let result = client.mock_all_auths().try_create_financing(
        &funder,
        &beneficiary,
        &0i128,
        &1u32,
        &milestones,
    );
    assert!(result.is_err());
}

#[test]
fn test_create_financing_rejects_empty_milestones() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let milestones = Vec::new(&env);

    let result = client.mock_all_auths().try_create_financing(
        &funder,
        &beneficiary,
        &1000i128,
        &0u32,
        &milestones,
    );
    assert!(result.is_err());
}

#[test]
fn test_deposit_increases_drawn_amount() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    client.mock_all_auths().deposit(&id, &funder, &400);

    let record = client.get_financing(&id);
    assert_eq!(record.drawn_amount, 400);
}

#[test]
fn test_deposit_rejects_exceeding_total() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    let result = client.mock_all_auths().try_deposit(&id, &funder, &1001);
    assert!(result.is_err());
}

#[test]
fn test_deposit_rejects_nonpositive() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    let result = client.mock_all_auths().try_deposit(&id, &funder, &0);
    assert!(result.is_err());
}

#[test]
fn test_release_on_milestone_releases_positive_amount() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    client
        .mock_all_auths()
        .release_on_milestone(&id, &Bytes::from_slice(&env, &[1; 32]), &0u32);

    let record = client.get_financing(&id);
    assert_eq!(record.drawn_amount, 600);
}

#[test]
fn test_release_on_milestone_refunds_negative_amount() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let milestones = Vec::from_slice(
        &env,
        &[Milestone {
            index: 1,
            deadline_ledger: seq + 1000,
            proof_hash: Bytes::from_slice(&env, &[1; 32]),
            proof_amount: -600,
        }],
    );
    let id = client.mock_all_auths().create_financing(
        &funder,
        &beneficiary,
        &1000i128,
        &1u32,
        &milestones,
    );

    client
        .mock_all_auths()
        .release_on_milestone(&id, &Bytes::from_slice(&env, &[1; 32]), &0u32);

    let record = client.get_financing(&id);
    assert_eq!(record.drawn_amount, 600);
}

#[test]
fn test_release_on_milestone_rejects_deadline_exceeded() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 10);

    env.ledger().set_sequence_number(seq + 100);

    let result = client.mock_all_auths().try_release_on_milestone(
        &id,
        &Bytes::from_slice(&env, &[1; 32]),
        &0u32,
    );
    assert!(result.is_err());
}

#[test]
fn test_release_on_milestone_rejects_empty_proof() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    let result =
        client
            .mock_all_auths()
            .try_release_on_milestone(&id, &Bytes::from_slice(&env, &[]), &0u32);
    assert!(result.is_err());
}

#[test]
fn test_release_on_milestone_rejects_bad_index() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    let result = client.mock_all_auths().try_release_on_milestone(
        &id,
        &Bytes::from_slice(&env, &[1; 32]),
        &5u32,
    );
    assert!(result.is_err());
}

#[test]
fn test_refund_refunds_undrawn_amount() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    client.mock_all_auths().refund(&id, &funder);

    let record = client.get_financing(&id);
    assert_eq!(record.repaid_amount, 1000);
}

#[test]
fn test_refund_marks_default_on_deadline_missed() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    let id = create_financing(&env, &client, &funder, &beneficiary, seq + 10);

    env.ledger().set_sequence_number(seq + 100);

    client.mock_all_auths().refund(&id, &funder);

    let record = client.get_financing(&id);
    assert!(record.defaulted);
    assert_eq!(record.defaulted_ledger, Some(env.ledger().sequence()));
}

#[test]
fn test_get_financings_by_funder() {
    let (env, client, _admin, funder, beneficiary) = setup();
    let seq = env.ledger().sequence();
    create_financing(&env, &client, &funder, &beneficiary, seq + 1000);
    create_financing(&env, &client, &funder, &beneficiary, seq + 1000);

    let financings = client.get_financings_by_funder(&funder);
    assert_eq!(financings.len(), 2);
    assert_eq!(financings.get(0).unwrap(), 1);
    assert_eq!(financings.get(1).unwrap(), 2);
}

#[test]
#[should_panic]
fn test_get_financing_not_found() {
    let (_env, client, _admin, _funder, _beneficiary) = setup();
    client.get_financing(&999);
}
