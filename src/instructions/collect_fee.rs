//! Instruction for manager to collect accumulated fee from an escrow fee account.
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
        program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    },
    spl_token::{instruction as token_instruction, state::Account as TokenAccount},
};

use crate::{errors::EscrowError, state::EscrowState, utils::assert_is_associated_token_account};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct CollectFeeArgs {
    pub should_close_fee_account: bool,
}

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CollectFeeArgs,
) -> ProgramResult {
    let [
        // accounts in order (see `crate::instruction::EscrowInstruction` enum)
        escrow_state_info,
        manager,
        escrow_fee_mint,
        escrow_fee_account,
        destination_token_account,
        token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the manager signs the instruction
    if !manager.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the escrow state and create program address
    let escrow_state = EscrowState::try_from_slice(&escrow_state_info.data.borrow()[..])?;
    let escrow_state_address = EscrowState::create_program_address(program_id, escrow_state.bump)?;

    // Ensure escrow state address is correct
    if *escrow_state_info.key != escrow_state_address {
        return Err(EscrowError::EscrowStateKeyMismatch.into());
    };

    // Ensure the provided manager is authorized
    if *manager.key != escrow_state.manager {
        return Err(EscrowError::ManagerKeyUnauthorized.into());
    }

    // Validate the escrow fee account
    assert_is_associated_token_account(
        escrow_fee_account.key,
        escrow_state_info.key,
        escrow_fee_mint.key,
    )?;

    let fee_amount = TokenAccount::unpack(&escrow_fee_account.data.borrow())?.amount;

    let escrow_state_signer_seed = &[EscrowState::SEED, &[escrow_state.bump]];
    if fee_amount != 0 {
        invoke_signed(
            &token_instruction::transfer(
                token_program.key,
                escrow_fee_account.key,
                destination_token_account.key,
                escrow_state_info.key,
                &[],
                fee_amount,
            )?,
            // 0. `[writable]` The source account.
            // 1. `[writable]` The destination account.
            // 2. `[signer]` The source account's owner/delegate.
            &[
                escrow_fee_account.clone(),
                destination_token_account.clone(),
                escrow_state_info.clone(),
                token_program.clone(),
            ],
            &[escrow_state_signer_seed],
        )?;

        solana_program::msg!("Collected fee: {}", fee_amount);
    }
    if args.should_close_fee_account {
        invoke_signed(
            &token_instruction::close_account(
                token_program.key,
                escrow_fee_account.key,
                manager.key,
                escrow_state_info.key,
                &[],
            )?,
            // 0. `[writable]` The account to close.
            // 1. `[writable]` The destination account.
            // 2. `[signer]` The account's owner.
            &[
                escrow_fee_account.clone(),
                manager.clone(),
                escrow_state_info.clone(),
                token_program.clone(),
            ],
            &[escrow_state_signer_seed],
        )?;
    }

    Ok(())
}
