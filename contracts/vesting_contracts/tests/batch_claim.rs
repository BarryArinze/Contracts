use soroban_sdk::testutils::Address as_;
use soroban_sdk::{vec, Address, Env};

use vesting_contracts::{VestingContract, VestingContractClient};

fn setup(env: &Env) -> (VestingContractClient<'static>, Address, Address) {
    env.mock_all_auths();

    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let user = Address::generate(env);
    client.initialize(&admin, &1_000_000i128);

    // Set token
    let token_id = Address::generate(env);
    client.set_token(&token_id);

    (client, admin, user)
}

#[test]
fn test_batch_claim_single_vault() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Create a vault for the user
    let vault_id = client.create_vault_full(
        &user,
        &1000i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    // Fast forward time to make tokens vestable
    env.ledger().set_timestamp(start + 500);

    // Check total claimable amount
    let total_claimable = client.get_total_claimable_amount(&user);
    assert_eq!(total_claimable, 500i128);

    // Batch claim should return the same amount
    let claimed = client.batch_claim();
    assert_eq!(claimed, 500i128);

    // After claiming, total claimable should be 0
    let total_claimable_after = client.get_total_claimable_amount(&user);
    assert_eq!(total_claimable_after, 0i128);
}

#[test]
fn test_batch_claim_multiple_vaults() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Create multiple vaults for the user (simulating Seed, Private, Advisory schedules)
    let vault1 = client.create_vault_full(
        &user,
        &500i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    let vault2 = client.create_vault_full(
        &user,
        &300i128,
        &start,
        &end + 500,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    let vault3 = client.create_vault_full(
        &user,
        &200i128,
        &start,
        &end + 1000,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    // Fast forward time
    env.ledger().set_timestamp(start + 500);

    // Calculate expected claimable amounts:
    // Vault1: 500 * (500/1000) = 250
    // Vault2: 300 * (500/1500) = 100
    // Vault3: 200 * (500/2000) = 50
    let expected_total = 250i128 + 100i128 + 50i128;

    // Check total claimable amount
    let total_claimable = client.get_total_claimable_amount(&user);
    assert_eq!(total_claimable, expected_total);

    // Batch claim should aggregate all amounts
    let claimed = client.batch_claim();
    assert_eq!(claimed, expected_total);

    // After claiming, total claimable should be 0
    let total_claimable_after = client.get_total_claimable_amount(&user);
    assert_eq!(total_claimable_after, 0i128);
}

#[test]
fn test_batch_claim_skips_frozen_vaults() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Create two vaults
    let vault1 = client.create_vault_full(
        &user,
        &500i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    let vault2 = client.create_vault_full(
        &user,
        &300i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    // Freeze one vault
    client.freeze_vault(&vault2, &true);

    // Fast forward time
    env.ledger().set_timestamp(start + 500);

    // Should only claim from the non-frozen vault
    let expected_claimable = 250i128; // Only from vault1
    let claimed = client.batch_claim();
    assert_eq!(claimed, expected_claimable);
}

#[test]
fn test_batch_claim_skips_paused_vaults() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Create two vaults
    let vault1 = client.create_vault_full(
        &user,
        &500i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    let vault2 = client.create_vault_full(
        &user,
        &300i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    // Set pause authority and pause one vault
    let pause_auth = Address::generate(&env);
    client.set_pause_authority(&pause_auth);
    env.mock_auths(&[
        (&pause_auth, &client.contract_id, &Symbol::new(&env, "pause_specific_schedule"), &())
    ]);
    client.pause_specific_schedule(&vault2, &String::from_str(&env, "Test pause"));

    // Fast forward time
    env.ledger().set_timestamp(start + 500);

    // Should only claim from the non-paused vault
    let expected_claimable = 250i128; // Only from vault1
    let claimed = client.batch_claim();
    assert_eq!(claimed, expected_claimable);
}

#[test]
fn test_batch_claim_no_claimable_tokens() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Create a vault but don't fast forward time
    let vault_id = client.create_vault_full(
        &user,
        &1000i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    // No tokens should be claimable yet
    let claimed = client.batch_claim();
    assert_eq!(claimed, 0i128);
}

#[test]
fn test_batch_claim_user_with_no_vaults() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    // User has no vaults
    let claimed = client.batch_claim();
    assert_eq!(claimed, 0i128);
}

#[test]
fn test_batch_claim_respects_locked_tokens() {
    let env = Env::default();
    let (client, admin, user) = setup(&env);

    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Create a vault
    let vault_id = client.create_vault_full(
        &user,
        &1000i128,
        &start,
        &end,
        &0i128,
        &true,
        &false,
        &0u64,
    );

    // Set collateral bridge and lock some tokens
    let bridge = Address::generate(&env);
    client.set_collateral_bridge(&bridge);
    
    env.mock_auths(&[
        (&bridge, &client.contract_id, &Symbol::new(&env, "lock_tokens"), &())
    ]);
    client.lock_tokens(&vault_id, &200i128);

    // Fast forward time
    env.ledger().set_timestamp(start + 500);

    // Should claim 500 (vested) - 200 (locked) = 300
    let claimed = client.batch_claim();
    assert_eq!(claimed, 300i128);
}
