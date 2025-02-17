use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

use escrow_program::{instructions::set_manager_ix, state::EscrowState};

use crate::utils::{add_escrow_state_account, prepare_program_test};

#[tokio::test]
async fn it_sets_manager() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Initialize the escrow state account
    let manager_keypair = Keypair::new();
    add_escrow_state_account(
        &mut program_test,
        EscrowState::new(&escrow_program::ID, manager_keypair.pubkey(), 0, 0).0,
    );

    // Create new manager pubkey
    let new_manager_pubkey = Pubkey::new_unique();

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call set fees instruction
    let set_manager_instruction = set_manager_ix(&manager_keypair.pubkey(), &new_manager_pubkey);
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[set_manager_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &manager_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();
}
