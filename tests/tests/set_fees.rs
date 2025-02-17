use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

use escrow_program::{
    instructions::{set_fees::SetFeesArgs, set_fees_ix},
    state::EscrowState,
};

use crate::utils::{add_escrow_state_account, prepare_program_test};

#[tokio::test]
async fn it_sets_fees() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Initialize the escrow state account
    let manager_keypair = Keypair::new();
    add_escrow_state_account(
        &mut program_test,
        EscrowState::new(&escrow_program::ID, manager_keypair.pubkey(), 0, 0).0,
    );

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call set fees instruction
    let set_fees_instruction = set_fees_ix(
        &manager_keypair.pubkey(),
        SetFeesArgs {
            maker_fee_bps: 1337,
            taker_fee_bps: 420,
        },
    );
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[set_fees_instruction],
            Some(&payer_keypair.pubkey()),
            &[&payer_keypair, &manager_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();
}
