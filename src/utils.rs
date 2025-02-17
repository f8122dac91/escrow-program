use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

use crate::{consts::MAX_BPS_VALUE, errors::EscrowError};

pub fn assert_is_associated_token_account(
    token_account_address: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<(), ProgramError> {
    let associated_token_account_address =
        &spl_associated_token_account::get_associated_token_address(owner, mint);

    if token_account_address != associated_token_account_address {
        return Err(EscrowError::TokenAccountMismatch.into());
    }

    Ok(())
}

pub fn assert_token_account_mint_and_owner(
    token_account_info: &AccountInfo,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<(), ProgramError> {
    let token_account = TokenAccount::unpack(&token_account_info.data.borrow())?;
    if token_account.mint != *mint || token_account.owner != *owner {
        return Err(EscrowError::TokenAccountMismatch.into());
    }

    Ok(())
}

pub fn assert_is_bps_in_range(bps: u16) -> Result<(), ProgramError> {
    if bps > MAX_BPS_VALUE {
        return Err(EscrowError::MaxBpsValueExceeded.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_checks_bps() {
        assert!(assert_is_bps_in_range(0).is_ok());
        assert!(assert_is_bps_in_range(30).is_ok());
        assert!(assert_is_bps_in_range(10_000).is_ok());
        assert!(assert_is_bps_in_range(10_001).is_err());
        assert!(assert_is_bps_in_range(20_000).is_err());
    }
}
