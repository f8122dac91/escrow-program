use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use escrow_program::{
    instructions::cancel_offer_ix,
    state::{EscrowState, Offer},
};

use crate::utils::{
    add_escrow_state_account, add_offer_account, add_test_fixture_from_file, add_token_account,
    prepare_program_test,
};

const OFFER_ID: u64 = 0;
const TOKEN_A_OFFERED: u64 = 69;
const TOKEN_B_WANTED: u64 = 420;

#[tokio::test]
async fn it_cancels_offer() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Load token mints
    let token_a_mint_address = add_test_fixture_from_file(&mut program_test, "inf-mint.json");
    let token_b_mint_address = add_test_fixture_from_file(&mut program_test, "usdc-mint.json");

    // Initialize the escrow state account
    add_escrow_state_account(
        &mut program_test,
        EscrowState::new(&escrow_program::ID, Pubkey::new_unique(), 0, 0).0,
    );

    // Create maker keypair, and token A token account address (ATA)
    let maker_keypair = Keypair::new();

    // Initialize an offer (and its vault account) to be canceled
    let (offer, offer_address) = Offer::new(
        &escrow_program::ID,
        OFFER_ID,
        maker_keypair.pubkey(),
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

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call cancel offer instruction
    let cancel_offer_instruction = cancel_offer_ix(
        &maker_keypair.pubkey(),
        &token_a_mint_address,
        &payer_keypair.pubkey(),
        OFFER_ID,
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[cancel_offer_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &maker_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();

    // Check the result
    let maker_token_a_account_pubkey =
        get_associated_token_address(&maker_keypair.pubkey(), &token_a_mint_address);

    let maker_token_a_account_balance_after_cancel = banks_client
        .get_packed_account_data::<TokenAccount>(maker_token_a_account_pubkey)
        .await
        .unwrap()
        .amount;

    assert_eq!(maker_token_a_account_balance_after_cancel, TOKEN_A_OFFERED);
}
