//! Instruction for manager to update escrow fees.
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use crate::{errors::EscrowError, state::EscrowState, utils::assert_is_bps_in_range};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SetFeesArgs {
    pub maker_fee_bps: u16,
    pub taker_fee_bps: u16,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], args: SetFeesArgs) -> ProgramResult {
    // Check the range of bps values in args
    assert_is_bps_in_range(args.maker_fee_bps)?;
    assert_is_bps_in_range(args.taker_fee_bps)?;

    let [
        // accounts in order (see `crate::instruction::EscrowInstruction` enum)
        escrow_state_info,
        manager,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the manager signs the instruction
    if !manager.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the escrow state and create program address
    let mut escrow_state = EscrowState::try_from_slice(&escrow_state_info.data.borrow()[..])?;
    let escrow_state_address = EscrowState::create_program_address(program_id, escrow_state.bump)?;

    // Ensure the provided escrow state address is correct
    if *escrow_state_info.key != escrow_state_address {
        return Err(EscrowError::EscrowStateKeyMismatch.into());
    };

    // Ensure the provided manager is authorized
    if *manager.key != escrow_state.manager {
        return Err(EscrowError::ManagerKeyUnauthorized.into());
    }

    escrow_state.maker_fee_bps = args.maker_fee_bps;
    escrow_state.taker_fee_bps = args.taker_fee_bps;

    // Write data into escrow state account
    escrow_state.serialize(&mut *escrow_state_info.data.borrow_mut())?;

    solana_program::msg!("Set fees in the escrow state: {:?}", escrow_state);

    Ok(())
}
