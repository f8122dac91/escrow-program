use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use escrow_program::{
    instructions::{make_offer::MakeOfferArgs, make_offer_ix},
    state::{EscrowState, Offer},
};

use crate::utils::{
    add_escrow_state_account, add_test_fixture_from_file, add_token_account, prepare_program_test,
};

const OFFER_ID: u64 = 0;
const MAKER_TOKEN_A_BALANCE: u64 = 1337;
const TOKEN_A_OFFERED: u64 = 69; // NB: should be lower that MAKER_TOKEN_A_BALANCE
const TOKEN_B_WANTED: u64 = 420;
const MAKER_FEE_BPS: u16 = 100;
const TAKER_FEE_BPS: u16 = 500;

#[tokio::test]
async fn it_makes_offer() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Load token mints
    let token_a_mint_address = add_test_fixture_from_file(&mut program_test, "inf-mint.json");
    let token_b_mint_address = add_test_fixture_from_file(&mut program_test, "usdc-mint.json");

    // Initialize the escrow state account
    add_escrow_state_account(
        &mut program_test,
        EscrowState::new(
            &escrow_program::ID,
            Pubkey::new_unique(),
            MAKER_FEE_BPS,
            TAKER_FEE_BPS,
        )
        .0,
    );

    // Create maker keypair, and token A token account address (ATA)
    let maker_keypair = Keypair::new();
    let maker_token_a_account_pubkey =
        get_associated_token_address(&maker_keypair.pubkey(), &token_a_mint_address);

    // Initialize maker's token A ATA
    add_token_account(
        &mut program_test,
        maker_token_a_account_pubkey,
        token_a_mint_address,
        maker_keypair.pubkey(),
        MAKER_TOKEN_A_BALANCE,
    );

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call make offer instruction
    let make_offer_instruction = make_offer_ix(
        &maker_keypair.pubkey(),
        &maker_token_a_account_pubkey,
        &token_a_mint_address,
        &token_b_mint_address,
        &payer_keypair.pubkey(),
        MakeOfferArgs {
            id: OFFER_ID,
            token_a_offered_amount: TOKEN_A_OFFERED,
            token_b_wanted_amount: TOKEN_B_WANTED,
        },
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[make_offer_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &maker_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();

    // Check the result
    let (offer_address, _bump) =
        Offer::find_program_address(&escrow_program::ID, &maker_keypair.pubkey(), OFFER_ID);
    let offer_after_make = banks_client
        .get_account_data_with_borsh::<Offer>(offer_address)
        .await
        .unwrap();
    assert_eq!(offer_after_make.id, OFFER_ID);
    assert_eq!(offer_after_make.maker, maker_keypair.pubkey());
    assert_eq!(offer_after_make.token_a_mint, token_a_mint_address);
    assert_eq!(offer_after_make.token_b_mint, token_b_mint_address);
    assert_eq!(offer_after_make.token_b_wanted_amount, TOKEN_B_WANTED);

    let vault_address = get_associated_token_address(&offer_address, &token_a_mint_address);
    let vault_balance_after_make = banks_client
        .get_packed_account_data::<TokenAccount>(vault_address)
        .await
        .unwrap()
        .amount;
    assert_eq!(vault_balance_after_make, TOKEN_A_OFFERED);
}
