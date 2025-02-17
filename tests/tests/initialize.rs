use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

use escrow_program::{
    consts::INITIAL_MANAGER_KEYPAIR,
    instructions::{initialize::InitializeArgs, initialize_ix},
    state::EscrowState,
};

use crate::utils::prepare_program_test;

#[tokio::test]
async fn it_initializes() {
    // [Setup Test]
    let program_test = prepare_program_test();

    // Load initial manager key pair (NOTE: requires "testing" feature to be enabled)
    let initial_manager_keypair = Keypair::from_bytes(&INITIAL_MANAGER_KEYPAIR).unwrap();

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call initialize instruction
    let initialize_instruction = initialize_ix(
        &initial_manager_keypair.pubkey(),
        &payer_keypair.pubkey(),
        InitializeArgs {
            maker_fee_bps: 69,
            taker_fee_bps: 420,
        },
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &initial_manager_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();

    // Check the result
    let (escrow_state_address, _bump) = EscrowState::find_program_address(&escrow_program::ID);

    let escrow_state_after_init = banks_client
        .get_account_data_with_borsh::<EscrowState>(escrow_state_address)
        .await
        .unwrap();

    assert_eq!(
        escrow_state_after_init.manager,
        initial_manager_keypair.pubkey()
    );
}
