//! Instruction for maker to cancel an existing offer.
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
    },
    spl_associated_token_account::instruction as associated_token_account_instruction,
    spl_token::{instruction as token_instruction, state::Account as TokenAccount},
};

use crate::{errors::EscrowError, state::Offer, utils::assert_is_associated_token_account};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct CancelOfferArgs {}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [
        // accounts in order (see `crate::instruction::EscrowInstruction` enum)
        offer_info,
        token_a_mint,
        maker_token_a_account,
        vault,
        maker,
        payer,
        token_program,
        associated_token_program,
        system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the maker signs the instruction
    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the offer
    let offer = Offer::try_from_slice(&offer_info.data.borrow()[..])?;

    // Validate the offer
    assert_eq!(&offer.maker, maker.key);
    assert_eq!(&offer.token_a_mint, token_a_mint.key);

    // Create program address of the offer
    let offer_address = Offer::create_program_address(program_id, maker.key, offer.id, offer.bump)?;

    // Ensure the provided offer address is correct
    if *offer_info.key != offer_address {
        return Err(EscrowError::OfferKeyMismatch.into());
    };

    let offer_signer_seed = &[
        Offer::SEED_PREFIX,
        maker.key.as_ref(),
        &offer.id.to_le_bytes(),
        &[offer.bump],
    ];

    // Validate the receiving token A accout is owned by the maker (ATA)
    assert_is_associated_token_account(maker_token_a_account.key, maker.key, token_a_mint.key)?;

    // Validate vault is owned by the offer account (ATA)
    assert_is_associated_token_account(vault.key, offer_info.key, token_a_mint.key)?;

    // Create maker token A account (ATA) if needed, before receiveing tokens
    invoke(
        &associated_token_account_instruction::create_associated_token_account_idempotent(
            payer.key,
            maker.key,
            token_a_mint.key,
            token_program.key,
        ),
        //   0. `[writeable,signer]` Funding account (must be a system account)
        //   1. `[writeable]` Associated token account address to be created
        //   2. `[]` Wallet address for the new associated token account
        //   3. `[]` The token mint for the new associated token account
        //   4. `[]` System program
        //   5. `[]` SPL Token program
        &[
            payer.clone(),
            maker_token_a_account.clone(),
            maker.clone(),
            token_a_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Read token amount in the offer's vault account
    let vault_amount = TokenAccount::unpack(&vault.data.borrow())?.amount;
    let maker_amount_before_transfer =
        TokenAccount::unpack(&maker_token_a_account.data.borrow())?.amount;

    // Transfer (refund) token A in vault to maker
    invoke_signed(
        &token_instruction::transfer(
            token_program.key,
            vault.key,
            maker_token_a_account.key,
            offer_info.key,
            &[],
            vault_amount,
        )?,
        //   0. `[writable]` The source account.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The source account's owner/delegate.
        &[
            vault.clone(),
            maker_token_a_account.clone(),
            offer_info.clone(),
            token_program.clone(),
        ],
        &[offer_signer_seed],
    )?;

    let maker_amount_after_transfer =
        TokenAccount::unpack(&maker_token_a_account.data.borrow())?.amount;
    assert_eq!(
        maker_amount_after_transfer,
        maker_amount_before_transfer + vault_amount
    );

    // Close the vault account
    invoke_signed(
        &token_instruction::close_account(
            token_program.key,
            vault.key,
            payer.key,
            offer_info.key,
            &[],
        )?,
        //   0. `[writable]` The account to close.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The account's owner.
        &[
            vault.clone(),
            payer.clone(),
            offer_info.clone(),
            token_program.clone(),
        ],
        &[offer_signer_seed],
    )?;

    // Send the rent back to the payer
    let lamports = offer_info.lamports();
    **offer_info.lamports.borrow_mut() -= lamports;
    **payer.lamports.borrow_mut() += lamports;

    // Realloc the account to zero
    offer_info.realloc(0, true)?;

    // Assign the account to the System Program
    offer_info.assign(system_program.key);

    Ok(())
}
