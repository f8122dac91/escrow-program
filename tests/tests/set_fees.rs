use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

use escrow_program::{
    instructions::{set_fees::SetFeesArgs, set_fees_ix},
    state::EscrowState,
};

use crate::utils::{add_escrow_state_account, prepare_program_test};

const ORIG_MAKER_FEE_BPS: u16 = 0;
const ORIG_TAKER_FEE_BPS: u16 = 0;
const MAKER_FEE_BPS: u16 = 1337;
const TAKER_FEE_BPS: u16 = 420;

#[tokio::test]
async fn it_sets_fees() {
    // [Setup Test]
    let mut program_test = prepare_program_test();

    // Initialize the escrow state account
    let manager_keypair = Keypair::new();
    let (escrow_state, escrow_state_address) = EscrowState::new(
        &escrow_program::ID,
        manager_keypair.pubkey(),
        ORIG_MAKER_FEE_BPS,
        ORIG_TAKER_FEE_BPS,
    );
    add_escrow_state_account(&mut program_test, escrow_state);

    // [Start Test]
    let (banks_client, payer_keypair, last_blockhash) = program_test.start().await;

    // Call set fees instruction
    let set_fees_instruction = set_fees_ix(
        &manager_keypair.pubkey(),
        SetFeesArgs {
            maker_fee_bps: MAKER_FEE_BPS,
            taker_fee_bps: TAKER_FEE_BPS,
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

    // Check the result
    let (escrow_state_maker_fee_after_set, escrow_state_taker_fee_after_set) = {
        let escrow_state = banks_client
            .get_account_data_with_borsh::<EscrowState>(escrow_state_address)
            .await
            .unwrap();

        (escrow_state.maker_fee_bps, escrow_state.taker_fee_bps)
    };

    assert_eq!(escrow_state_maker_fee_after_set, MAKER_FEE_BPS);
    assert_eq!(escrow_state_taker_fee_after_set, TAKER_FEE_BPS);
}
