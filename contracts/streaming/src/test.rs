use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env,
};

struct Ctx<'a> {
    env: Env,
    client: StreamingContractClient<'a>,
    token: TokenClient<'a>,
    payer: Address,
    alice: Address,
    bob: Address,
}

fn setup<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let payer = Address::generate(&env);
    StellarAssetClient::new(&env, &sac.address()).mint(&payer, &10_000);
    let id = env.register_contract(None, StreamingContract);
    let client = StreamingContractClient::new(&env, &id);
    let token = TokenClient::new(&env, &sac.address());
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    Ctx { env, client, token, payer, alice, bob }
}

fn create(c: &Ctx) -> u64 {
    c.client.create_stream(
        &c.payer,
        &vec![&c.env, (c.alice.clone(), 60u32), (c.bob.clone(), 40u32)],
        &c.token.address,
        &1000,
        &1000,
        &2000,
    )
}

#[test]
fn vesting_boundaries() {
    assert_eq!(vested_at(1000, 100, 200, 50), 0);
    assert_eq!(vested_at(1000, 100, 200, 100), 0);
    assert_eq!(vested_at(1000, 100, 200, 150), 500);
    assert_eq!(vested_at(1000, 100, 200, 200), 1000);
    assert_eq!(vested_at(1000, 100, 200, 999), 1000);
    assert_eq!(vested_at(10, 0, 3, 1), 3); // floor(10/3)
}

#[test]
fn create_escrows_funds() {
    let c = setup();
    let id = create(&c);
    assert_eq!(id, 1);
    assert_eq!(c.token.balance(&c.payer), 9_000);
    assert_eq!(c.token.balance(&c.client.address), 1_000);
}

#[test]
fn create_rejects_invalid_inputs() {
    let c = setup();
    let t = &c.token.address;
    let one = vec![&c.env, (c.alice.clone(), 100u32)];
    let bad_sum = vec![&c.env, (c.alice.clone(), 60u32), (c.bob.clone(), 30u32)];
    let empty: Vec<(Address, u32)> = vec![&c.env];
    assert_eq!(c.client.try_create_stream(&c.payer, &one, t, &0, &1000, &2000), Err(Ok(Error::InvalidAmount)));
    assert_eq!(c.client.try_create_stream(&c.payer, &one, t, &-5, &1000, &2000), Err(Ok(Error::InvalidAmount)));
    assert_eq!(c.client.try_create_stream(&c.payer, &one, t, &10, &2000, &1000), Err(Ok(Error::InvalidTimes)));
    assert_eq!(c.client.try_create_stream(&c.payer, &one, t, &10, &500, &900), Err(Ok(Error::InvalidTimes)));
    assert_eq!(c.client.try_create_stream(&c.payer, &bad_sum, t, &10, &1000, &2000), Err(Ok(Error::InvalidRecipients)));
    assert_eq!(c.client.try_create_stream(&c.payer, &empty, t, &10, &1000, &2000), Err(Ok(Error::InvalidRecipients)));
}

#[test]
fn create_requires_payer_auth() {
    let env = Env::default(); // no mock_all_auths
    let id = env.register_contract(None, StreamingContract);
    let client = StreamingContractClient::new(&env, &id);
    let p = Address::generate(&env);
    let t = Address::generate(&env);
    let r = vec![&env, (Address::generate(&env), 100u32)];
    assert!(client.try_create_stream(&p, &r, &t, &10, &1000, &2000).is_err());
}

#[test]
fn withdraw_over_time_and_no_double_spend() {
    let c = setup();
    let id = create(&c);
    c.env.ledger().set_timestamp(1500); // 50% vested => alice 300, bob 200
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 300);
    assert_eq!(c.client.get_withdrawable(&id, &c.bob), 200);
    c.client.withdraw(&id, &c.alice, &300);
    assert_eq!(c.token.balance(&c.alice), 300);
    assert_eq!(c.client.try_withdraw(&id, &c.alice, &1), Err(Ok(Error::NothingToWithdraw)));
    assert_eq!(c.client.try_withdraw(&id, &c.bob, &201), Err(Ok(Error::AmountExceedsWithdrawable)));
    c.env.ledger().set_timestamp(2500);
    c.client.withdraw(&id, &c.alice, &300);
    c.client.withdraw(&id, &c.bob, &400);
    assert_eq!(c.token.balance(&c.alice), 600);
    assert_eq!(c.token.balance(&c.bob), 400);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn withdraw_before_start_and_non_recipient() {
    let c = setup();
    let id = create(&c);
    assert_eq!(c.client.try_withdraw(&id, &c.alice, &1), Err(Ok(Error::NothingToWithdraw)));
    let stranger = Address::generate(&c.env);
    assert_eq!(c.client.try_withdraw(&id, &stranger, &1), Err(Ok(Error::NotRecipient)));
    assert_eq!(c.client.try_withdraw(&id, &c.alice, &0), Err(Ok(Error::InvalidAmount)));
    assert_eq!(c.client.try_withdraw(&99, &c.alice, &1), Err(Ok(Error::StreamNotFound)));
}

#[test]
fn cancel_refunds_unvested_and_recipients_keep_vested() {
    let c = setup();
    let id = create(&c);
    c.env.ledger().set_timestamp(1250); // 25% vested = 250
    assert_eq!(c.client.cancel_stream(&id), 750);
    assert_eq!(c.token.balance(&c.payer), 9_750);
    assert_eq!(c.client.try_cancel_stream(&id), Err(Ok(Error::AlreadyCancelled)));
    c.env.ledger().set_timestamp(9999); // vesting stays frozen
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 150);
    c.client.withdraw(&id, &c.alice, &150);
    c.client.withdraw(&id, &c.bob, &100);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

// Issue #3: additional regression tests. Keep this test-only; no contract changes.

#[test]
fn create_rejects_more_than_ten_recipients() {
    let c = setup();
    let mut recipients = Vec::new(&c.env);
    for _ in 0..11 {
        recipients.push_back((c.alice.clone(), 10u32));
    }
    assert_eq!(
        c.client.try_create_stream(
            &c.payer, &recipients, &c.token.address, &1000, &1000, &2000,
        ),
        Err(Ok(Error::InvalidRecipients)),
    );
    assert_eq!(c.token.balance(&c.payer), 10_000);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn create_rejects_zero_recipient_share() {
    let c = setup();
    let recipients = vec![&c.env, (c.alice.clone(), 0u32), (c.bob.clone(), 100u32)];
    assert_eq!(
        c.client.try_create_stream(
            &c.payer, &recipients, &c.token.address, &1000, &1000, &2000,
        ),
        Err(Ok(Error::InvalidRecipients)),
    );
    assert_eq!(c.token.balance(&c.payer), 10_000);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn create_rejects_share_above_hundred() {
    let c = setup();
    let recipients = vec![&c.env, (c.alice.clone(), 101u32)];
    assert_eq!(
        c.client.try_create_stream(
            &c.payer, &recipients, &c.token.address, &1000, &1000, &2000,
        ),
        Err(Ok(Error::InvalidRecipients)),
    );
    assert_eq!(c.token.balance(&c.payer), 10_000);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn cancel_before_start_refunds_everything() {
    let c = setup();
    // Current ledger timestamp is 1000; this stream will not begin until 1500.
    let id = c.client.create_stream(
        &c.payer,
        &vec![&c.env, (c.alice.clone(), 60u32), (c.bob.clone(), 40u32)],
        &c.token.address,
        &1000,
        &1500,
        &2500,
    );
    assert_eq!(c.token.balance(&c.payer), 9_000);
    assert_eq!(c.token.balance(&c.client.address), 1_000);

    assert_eq!(c.client.cancel_stream(&id), 1_000);
    assert_eq!(c.client.get_stream(&id).cancelled_at, Some(1000));
    assert_eq!(c.token.balance(&c.payer), 10_000);
    assert_eq!(c.token.balance(&c.client.address), 0);
    assert_eq!(c.token.balance(&c.alice), 0);
    assert_eq!(c.token.balance(&c.bob), 0);
    c.env.ledger().set_timestamp(3000);
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 0);
    assert_eq!(c.client.get_withdrawable(&id, &c.bob), 0);
}

#[test]
fn cancel_after_end_refunds_zero_without_token_transfer() {
    let c = setup();
    let id = create(&c);
    c.env.ledger().set_timestamp(2500);
    assert_eq!(c.client.cancel_stream(&id), 0);
    assert_eq!(c.client.get_stream(&id).cancelled_at, Some(2500));
    assert_eq!(c.token.balance(&c.payer), 9_000);
    assert_eq!(c.token.balance(&c.client.address), 1_000);
    assert_eq!(c.token.balance(&c.alice), 0);
    assert_eq!(c.token.balance(&c.bob), 0);
    c.client.withdraw(&id, &c.alice, &600);
    c.client.withdraw(&id, &c.bob, &400);
    assert_eq!(c.token.balance(&c.client.address), 0);
    assert_eq!(c.token.balance(&c.alice), 600);
    assert_eq!(c.token.balance(&c.bob), 400);
}

#[test]
fn cancel_requires_payer_authorization() {
    let c = setup();
    let id = create(&c);
    c.env.ledger().set_timestamp(1250);
    // No caller argument exists on cancel_stream; it authorizes the stored payer.
    // Disable the blanket authorization mock after creating the funded stream.
    c.env.set_auths(&[]);
    assert!(c.client.try_cancel_stream(&id).is_err());
    assert_eq!(c.client.get_stream(&id).cancelled_at, None);
    assert_eq!(c.token.balance(&c.payer), 9_000);
    assert_eq!(c.token.balance(&c.client.address), 1_000);
    assert_eq!(c.token.balance(&c.alice), 0);
    assert_eq!(c.token.balance(&c.bob), 0);
}

#[test]
fn recipients_can_withdraw_after_cancellation() {
    let c = setup();
    let id = create(&c);
    c.env.ledger().set_timestamp(1250);
    assert_eq!(c.client.cancel_stream(&id), 750);
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 150);
    assert_eq!(c.client.get_withdrawable(&id, &c.bob), 100);
    c.env.ledger().set_timestamp(5000);
    c.client.withdraw(&id, &c.alice, &150);
    c.client.withdraw(&id, &c.bob, &100);
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 0);
    assert_eq!(c.client.get_withdrawable(&id, &c.bob), 0);
    assert_eq!(c.token.balance(&c.payer), 9_750);
    assert_eq!(c.token.balance(&c.alice), 150);
    assert_eq!(c.token.balance(&c.bob), 100);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn cancelled_withdrawable_stays_frozen_as_ledger_advances() {
    let c = setup();
    let id = create(&c);
    c.env.ledger().set_timestamp(1250);
    c.client.withdraw(&id, &c.alice, &50);
    assert_eq!(c.client.cancel_stream(&id), 750);
    for now in [1250u64, 1500, 2000, 10_000] {
        c.env.ledger().set_timestamp(now);
        assert_eq!(c.client.get_withdrawable(&id, &c.alice), 100);
        assert_eq!(c.client.get_withdrawable(&id, &c.bob), 100);
    }
    assert_eq!(c.token.balance(&c.payer), 9_750);
    assert_eq!(c.token.balance(&c.alice), 50);
    assert_eq!(c.token.balance(&c.bob), 0);
    assert_eq!(c.token.balance(&c.client.address), 200);
    c.client.withdraw(&id, &c.alice, &100);
    c.client.withdraw(&id, &c.bob, &100);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn get_stream_reports_current_state_and_withdrawn_shares() {
    let c = setup();
    let id = create(&c);
    let initial = c.client.get_stream(&id);
    assert_eq!(initial.payer, c.payer);
    assert_eq!(initial.token, c.token.address);
    assert_eq!(initial.amount, 1_000);
    assert_eq!(initial.start_time, 1_000);
    assert_eq!(initial.end_time, 2_000);
    assert_eq!(initial.cancelled_at, None);
    assert_eq!(initial.recipients.len(), 2);
    let first = initial.recipients.get(0).unwrap();
    let second = initial.recipients.get(1).unwrap();
    assert_eq!((first.address, first.shares, first.withdrawn), (c.alice.clone(), 60, 0));
    assert_eq!((second.address, second.shares, second.withdrawn), (c.bob.clone(), 40, 0));

    c.env.ledger().set_timestamp(1500);
    c.client.withdraw(&id, &c.alice, &100);
    assert_eq!(c.client.get_stream(&id).recipients.get(0).unwrap().withdrawn, 100);
    assert_eq!(c.token.balance(&c.payer), 9_000);
    assert_eq!(c.token.balance(&c.alice), 100);
    assert_eq!(c.token.balance(&c.client.address), 900);
}

#[test]
fn get_vested_amount_uses_supplied_timestamp_and_ignores_cancellation() {
    let c = setup();
    let id = create(&c);
    for (at, expected) in [(900u64, 0), (1000, 0), (1250, 250), (2000, 1000), (3000, 1000)] {
        assert_eq!(c.client.get_vested_amount(&id, &at), expected);
    }
    c.env.ledger().set_timestamp(1250);
    assert_eq!(c.client.cancel_stream(&id), 750);
    // This getter reports hypothetical vesting at a supplied time, not a frozen claim.
    assert_eq!(c.client.get_vested_amount(&id, &3000), 1_000);
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 150);
    assert_eq!(c.token.balance(&c.payer), 9_750);
    assert_eq!(c.token.balance(&c.client.address), 250);
}

#[test]
fn missing_stream_queries_return_stream_not_found() {
    let c = setup();
    assert_eq!(c.client.try_get_stream(&999), Err(Ok(Error::StreamNotFound)));
    assert_eq!(
        c.client.try_get_vested_amount(&999, &1500),
        Err(Ok(Error::StreamNotFound)),
    );
    assert_eq!(
        c.client.try_get_withdrawable(&999, &c.alice),
        Err(Ok(Error::StreamNotFound)),
    );
    assert_eq!(c.token.balance(&c.payer), 10_000);
    assert_eq!(c.token.balance(&c.client.address), 0);
}

#[test]
fn cancelling_then_draining_recipients_leaves_one_unit_of_rounding_dust() {
    let c = setup();
    let id = create(&c); // 1000 units, 60% Alice, 40% Bob, 1000..2000.
    c.env.ledger().set_timestamp(1101); // vested = floor(1000 * 101 / 1000) = 101
    assert_eq!(c.client.get_vested_amount(&id, &1101), 101);
    assert_eq!(c.client.cancel_stream(&id), 899);
    // floor(101 * 60 / 100) = 60; floor(101 * 40 / 100) = 40.
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 60);
    assert_eq!(c.client.get_withdrawable(&id, &c.bob), 40);
    assert_eq!(c.token.balance(&c.client.address), 101);
    c.client.withdraw(&id, &c.alice, &60);
    c.client.withdraw(&id, &c.bob, &40);
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 0);
    assert_eq!(c.client.get_withdrawable(&id, &c.bob), 0);
    assert_eq!(c.token.balance(&c.payer), 9_899);
    assert_eq!(c.token.balance(&c.alice), 60);
    assert_eq!(c.token.balance(&c.bob), 40);
    assert_eq!(c.token.balance(&c.client.address), 1); // documented rounding dust
    assert_eq!(
        c.token.balance(&c.payer)
            + c.token.balance(&c.alice)
            + c.token.balance(&c.bob)
            + c.token.balance(&c.client.address),
        10_000,
    );
}

#[test]
fn duplicate_recipient_address_is_accepted_but_has_inconsistent_withdrawal_lookup() {
    let c = setup();
    // Contract does not reject duplicates. The view selects the first entry,
    // whereas withdraw currently updates the last matching entry.
    let duplicates = vec![&c.env, (c.alice.clone(), 50u32), (c.alice.clone(), 50u32)];
    let id = c.client.create_stream(
        &c.payer, &duplicates, &c.token.address, &1000, &1000, &2000,
    );
    assert_eq!(c.client.get_stream(&id).recipients.len(), 2);
    c.env.ledger().set_timestamp(2500);
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 500);
    c.client.withdraw(&id, &c.alice, &500);
    // This documents existing behavior, not the desired business rule.
    assert_eq!(c.client.get_withdrawable(&id, &c.alice), 500);
    assert_eq!(c.client.try_withdraw(&id, &c.alice, &500), Err(Ok(Error::NothingToWithdraw)));
    let stream = c.client.get_stream(&id);
    assert_eq!(stream.recipients.get(0).unwrap().withdrawn, 0);
    assert_eq!(stream.recipients.get(1).unwrap().withdrawn, 500);
    assert_eq!(c.token.balance(&c.payer), 9_000);
    assert_eq!(c.token.balance(&c.alice), 500);
    assert_eq!(c.token.balance(&c.bob), 0);
    assert_eq!(c.token.balance(&c.client.address), 500);
}
