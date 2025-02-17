//! Instruction to make an offer.
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
        sysvar::Sysvar,
    },
    spl_associated_token_account::instruction as associated_token_account_instruction,
    spl_token::{instruction as token_instruction, state::Account as TokenAccount},
};

use crate::{
    errors::EscrowError,
    state::Offer,
    utils::{assert_is_associated_token_account, assert_token_account_mint_and_owner},
};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct MakeOfferArgs {
    pub id: u64,
    pub token_a_offered_amount: u64,
    pub token_b_wanted_amount: u64,
}

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: MakeOfferArgs,
) -> ProgramResult {
    let [
        // accounts in order (see `crate::instruction::EscrowInstruction` enum)
        offer_info,
        token_a_mint,
        token_b_mint,
        maker_token_a_account,
        vault,
        maker,
        // maker @ { is_signer: true, .. }, // NOTE: syntax for checking conditions while pattern matching
        payer,
        token_program,
        associated_token_program,
        system_program,
        // res @ .. // NOTE: syntax for accepting dynamic length of accounts
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the maker signs the instruction
    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (offer, offer_address) = Offer::new(
        program_id,
        args.id,
        *maker.key,
        *token_a_mint.key,
        *token_b_mint.key,
        args.token_b_wanted_amount,
    );

    // Ensure the provided offer address is correct
    if *offer_info.key != offer_address {
        return Err(EscrowError::OfferKeyMismatch.into());
    };

    let offer_signer_seed = &[
        Offer::SEED_PREFIX,
        maker.key.as_ref(),
        &args.id.to_le_bytes(),
        &[offer.bump],
    ];

    // Validate the owner and mint of the sending token A account
    assert_token_account_mint_and_owner(maker_token_a_account, maker.key, token_a_mint.key)?;

    // Validate the vault is owned by the offer account (ATA)
    assert_is_associated_token_account(vault.key, offer_info.key, token_a_mint.key)?;

    // Create offer account
    let size = borsh::to_vec::<Offer>(&offer)?.len();
    let lamports_required = (Rent::get()?).minimum_balance(size);
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            offer_info.key,
            lamports_required,
            size as u64,
            program_id,
        ),
        //   0. `[WRITE, SIGNER]` Funding account
        //   1. `[WRITE, SIGNER]` New account
        &[
            payer.clone(),
            offer_info.clone(),
            system_program.clone(), //
        ],
        &[offer_signer_seed],
    )?;

    // Create the vault token account
    invoke(
        &associated_token_account_instruction::create_associated_token_account(
            payer.key,
            offer_info.key,
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
            vault.clone(),
            offer_info.clone(),
            token_a_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Transfer token A to vault
    invoke(
        &token_instruction::transfer(
            token_program.key,
            maker_token_a_account.key,
            vault.key,
            maker.key,
            &[maker.key],
            args.token_a_offered_amount,
        )?,
        //   0. `[writable]` The source account.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The source account's owner/delegate.
        &[
            maker_token_a_account.clone(),
            vault.clone(),
            maker.clone(),
            token_program.clone(), // not required
        ],
    )?;

    let vault_token_amount = TokenAccount::unpack(&vault.data.borrow())?.amount;

    assert_eq!(vault_token_amount, args.token_a_offered_amount);

    // Write data into offer account
    offer.serialize(&mut *offer_info.data.borrow_mut())?;

    Ok(())
}
