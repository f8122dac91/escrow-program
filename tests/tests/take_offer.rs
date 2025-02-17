use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use escrow_program::{
    instructions::take_offer_ix,
    state::{EscrowState, Offer},
};

use crate::utils::{
    add_escrow_state_account, add_offer_account, add_test_fixture_from_file, add_token_account,
    prepare_program_test,
};

const OFFER_ID: u64 = 0;
const TAKER_TOKEN_B_BALANCE: u64 = 1337;
const TOKEN_A_OFFERED: u64 = 69;
const TOKEN_B_WANTED: u64 = 420;
const MAKER_FEE_BPS: u16 = 100;
const TAKER_FEE_BPS: u16 = 500;

#[tokio::test]
async fn it_takes_offer() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Load token mints
    let token_a_mint_address = add_test_fixture_from_file(&mut program_test, "inf-mint.json");
    let token_b_mint_address = add_test_fixture_from_file(&mut program_test, "usdc-mint.json");

    // Initialize the escrow state account
    let (escrow_state, escrow_state_address) = EscrowState::new(
        &escrow_program::ID,
        Pubkey::new_unique(),
        MAKER_FEE_BPS,
        TAKER_FEE_BPS,
    );
    add_escrow_state_account(&mut program_test, escrow_state);

    // Create maker pubkey //, and token A token account address (ATA)
    let maker_pubkey = Pubkey::new_unique();

    // Initialize an offer (and its vault account) to be taken
    let (offer, offer_address) = Offer::new(
        &escrow_program::ID,
        OFFER_ID,
        maker_pubkey,
        token_a_mint_address,
        token_b_mint_address,
        TOKEN_B_WANTED,
    );
    add_offer_account(&mut program_test, offer);
    let vault_address = get_associated_token_address(&offer_address, &token_a_mint_address);
    add_token_account(
        &mut program_test,
        vault_address,
        token_a_mint_address,
        offer_address,
        TOKEN_A_OFFERED,
    );

    // Create taker keypair, and Token B token account (ATA)
    let taker_keypair = Keypair::new();
    let taker_token_b_account_pubkey =
        get_associated_token_address(&taker_keypair.pubkey(), &token_b_mint_address);
    add_token_account(
        &mut program_test,
        taker_token_b_account_pubkey,
        token_b_mint_address,
        taker_keypair.pubkey(),
        TAKER_TOKEN_B_BALANCE,
    );

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call take offer instruction
    let take_offer_instruction = take_offer_ix(
        &offer_address,
        &token_a_mint_address,
        &token_b_mint_address,
        &taker_token_b_account_pubkey,
        &maker_pubkey,
        &taker_keypair.pubkey(),
        &payer_keypair.pubkey(),
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[take_offer_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &taker_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();

    // Check the result
    let escrow_state = banks_client
        .get_account_data_with_borsh::<EscrowState>(escrow_state_address)
        .await
        .unwrap();
    let token_a_fee_amount = escrow_state.get_token_a_fee(TOKEN_A_OFFERED).unwrap();
    let token_b_fee_amount = escrow_state.get_token_b_fee(TOKEN_B_WANTED).unwrap();

    let token_a_escrow_fee_address =
        get_associated_token_address(&escrow_state_address, &token_a_mint_address);
    let token_b_escrow_fee_address =
        get_associated_token_address(&escrow_state_address, &token_b_mint_address);
    let token_a_escrow_fee_balance_after_take = banks_client
        .get_packed_account_data::<TokenAccount>(token_a_escrow_fee_address)
        .await
        .unwrap()
        .amount;
    let token_b_escrow_fee_balance_after_take = banks_client
        .get_packed_account_data::<TokenAccount>(token_b_escrow_fee_address)
        .await
        .unwrap()
        .amount;
    assert_eq!(token_a_escrow_fee_balance_after_take, token_a_fee_amount);
    assert_eq!(token_b_escrow_fee_balance_after_take, token_b_fee_amount);

    let taker_token_a_account_pubkey =
        get_associated_token_address(&taker_keypair.pubkey(), &token_a_mint_address);
    let taker_token_a_balance_after_take = banks_client
        .get_packed_account_data::<TokenAccount>(taker_token_a_account_pubkey)
        .await
        .unwrap()
        .amount;
    assert_eq!(
        taker_token_a_balance_after_take,
        TOKEN_A_OFFERED - token_a_fee_amount
    );

    let taker_token_b_balance_after_take = banks_client
        .get_packed_account_data::<TokenAccount>(taker_token_b_account_pubkey)
        .await
        .unwrap()
        .amount;
    assert_eq!(
        taker_token_b_balance_after_take,
        TAKER_TOKEN_B_BALANCE - TOKEN_B_WANTED
    );

    let maker_token_b_account_pubkey =
        get_associated_token_address(&maker_pubkey, &token_b_mint_address);
    let maker_token_b_balance_after_take = banks_client
        .get_packed_account_data::<TokenAccount>(maker_token_b_account_pubkey)
        .await
        .unwrap()
        .amount;
    assert_eq!(
        maker_token_b_balance_after_take,
        TOKEN_B_WANTED - token_b_fee_amount
    );

    // let escrow
}
