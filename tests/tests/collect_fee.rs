use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use escrow_program::{
    instructions::{collect_fee::CollectFeeArgs, collect_fee_ix},
    state::EscrowState,
};

use crate::utils::{
    add_escrow_state_account, add_test_fixture_from_file, add_token_account, prepare_program_test,
};

const ESCROW_FEE_BALANCE: u64 = 1337;

#[tokio::test]
async fn it_collects_fee() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Load token mints
    let token_mint_address = add_test_fixture_from_file(&mut program_test, "inf-mint.json");

    // Initialize the escrow state account
    let manager_keypair = Keypair::new();
    let (escrow_state, escrow_state_address) =
        EscrowState::new(&escrow_program::ID, manager_keypair.pubkey(), 0, 0);
    add_escrow_state_account(&mut program_test, escrow_state);

    // Initialize escrow fee account to be collected
    let escrow_fee_account =
        get_associated_token_address(&escrow_state_address, &token_mint_address);
    add_token_account(
        &mut program_test,
        escrow_fee_account,
        token_mint_address,
        escrow_state_address,
        ESCROW_FEE_BALANCE,
    );

    // Prepare and initialize manager's token account for fee destination
    let destination_token_account_address =
        get_associated_token_address(&manager_keypair.pubkey(), &token_mint_address);
    add_token_account(
        &mut program_test,
        destination_token_account_address,
        token_mint_address,
        manager_keypair.pubkey(),
        0,
    );

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call collect fee instruction
    let collect_fee_instruction = collect_fee_ix(
        &manager_keypair.pubkey(),
        &token_mint_address,
        &destination_token_account_address,
        CollectFeeArgs {
            should_close_fee_account: false,
        },
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[collect_fee_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &manager_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn it_collects_fee_and_closes_fee_account() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Load token mints
    let token_mint_address = add_test_fixture_from_file(&mut program_test, "inf-mint.json");

    // Initialize the escrow state account
    let manager_keypair = Keypair::new();
    let (escrow_state, escrow_state_address) =
        EscrowState::new(&escrow_program::ID, manager_keypair.pubkey(), 0, 0);
    add_escrow_state_account(&mut program_test, escrow_state);

    // Initialize escrow fee account to be collected
    let escrow_fee_account =
        get_associated_token_address(&escrow_state_address, &token_mint_address);
    add_token_account(
        &mut program_test,
        escrow_fee_account,
        token_mint_address,
        escrow_state_address,
        ESCROW_FEE_BALANCE,
    );

    // Prepare and initialize manager's token account for fee destination
    let destination_token_account_address =
        get_associated_token_address(&manager_keypair.pubkey(), &token_mint_address);
    add_token_account(
        &mut program_test,
        destination_token_account_address,
        token_mint_address,
        manager_keypair.pubkey(),
        0,
    );

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call collect fee instruction
    let collect_fee_instruction = collect_fee_ix(
        &manager_keypair.pubkey(),
        &token_mint_address,
        &destination_token_account_address,
        CollectFeeArgs {
            should_close_fee_account: true,
        },
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[collect_fee_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &manager_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();

    // Check the result
    let destination_token_account_balance_after_collect = banks_client
        .get_packed_account_data::<TokenAccount>(destination_token_account_address)
        .await
        .unwrap()
        .amount;

    assert_eq!(
        destination_token_account_balance_after_collect,
        ESCROW_FEE_BALANCE,
    );
}
