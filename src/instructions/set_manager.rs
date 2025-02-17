//! Instruction for manager to transfer the authority.
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use crate::{errors::EscrowError, state::EscrowState};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SetManagerArgs {}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [
        // accounts in order
        escrow_state_info,
        manager,
        new_manager,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the manager signs the instruction
    if !manager.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the escrow state create program address
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

    // Check for noop condition
    if *new_manager.key == escrow_state.manager {
        return Err(EscrowError::ManagerKeyAlreadySet.into());
    }

    escrow_state.manager = *new_manager.key;

    // Write data into escrow state account
    escrow_state.serialize(&mut *escrow_state_info.data.borrow_mut())?;

    solana_program::msg!("Set manager in the escrow state: {:?}", escrow_state);

    Ok(())
}
