//! Instruction to initialize the escrow state singleton.
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
        program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction,
        sysvar::Sysvar,
    },
};

use crate::{
    consts::INITIAL_MANAGER, errors::EscrowError, state::EscrowState, utils::assert_is_bps_in_range,
};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct InitializeArgs {
    pub maker_fee_bps: u16,
    pub taker_fee_bps: u16,
}

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitializeArgs,
) -> ProgramResult {
    // Check the range of bps values in args
    assert_is_bps_in_range(args.maker_fee_bps)?;
    assert_is_bps_in_range(args.taker_fee_bps)?;

    let [
        // accounts in order
        escrow_state_info,
        initial_manager,
        payer,
        system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the initial manager signs the instruction
    if !initial_manager.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the initial manager pubkey is correct
    if *initial_manager.key != INITIAL_MANAGER {
        return Err(EscrowError::InitialManagerKeyMismatch.into());
    }

    // Prepare a new escrow state and its address
    let (escrow_state, escrow_state_address) = EscrowState::new(
        program_id,
        *initial_manager.key,
        args.maker_fee_bps,
        args.taker_fee_bps,
    );

    // Ensure the provided escrow state address is correct
    if *escrow_state_info.key != escrow_state_address {
        return Err(EscrowError::EscrowStateKeyMismatch.into());
    };

    // Create escrow state account
    let size = borsh::to_vec::<EscrowState>(&escrow_state)?.len();
    let lamports_required = Rent::get()?.minimum_balance(size);

    let escrow_state_signer_seed = &[EscrowState::SEED, &[escrow_state.bump]];
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            escrow_state_info.key,
            lamports_required,
            size as u64,
            program_id,
        ),
        //   0. `[WRITE, SIGNER]` Funding account
        //   1. `[WRITE, SIGNER]` New account
        &[
            payer.clone(),
            escrow_state_info.clone(),
            system_program.clone(),
        ],
        &[escrow_state_signer_seed],
    )?;

    // Write data into escrow state account
    escrow_state.serialize(&mut *escrow_state_info.data.borrow_mut())?;

    msg!("Initialized escrow state: {:?}", escrow_state);

    Ok(())
}
