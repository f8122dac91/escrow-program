//! Instruction to take an existing offer.
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

use crate::{
    errors::EscrowError,
    state::{EscrowState, Offer},
    utils::assert_is_associated_token_account,
};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct TakeOfferArgs {}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [
        // accounts in order (see `crate::instruction::EscrowInstruction` enum)
        escrow_state_info,
        offer_info,
        token_a_mint,
        token_b_mint,
        maker_token_b_account,
        taker_token_a_account,
        taker_token_b_account,
        escrow_fee_token_a_account,
        escrow_fee_token_b_account,
        vault,
        maker,
        taker,
        payer,
        token_program,
        associated_token_program,
        system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure the taker signs the instruction
    if !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the escrow state create program address
    let escrow_state = EscrowState::try_from_slice(&escrow_state_info.data.borrow()[..])?;
    let escrow_state_address = EscrowState::create_program_address(program_id, escrow_state.bump)?;

    // Ensure the provided escrow state address is correct
    if *escrow_state_info.key != escrow_state_address {
        return Err(EscrowError::EscrowStateKeyMismatch.into());
    };

    // Validate the escrow fee token accounts are owned by the escrow state (ATA)
    assert_is_associated_token_account(
        escrow_fee_token_a_account.key,
        escrow_state_info.key,
        token_a_mint.key,
    )?;
    assert_is_associated_token_account(
        escrow_fee_token_b_account.key,
        escrow_state_info.key,
        token_b_mint.key,
    )?;

    // Deserialize the offer
    let offer = Offer::try_from_slice(&offer_info.data.borrow()[..])?;

    // Validate the offer
    assert_eq!(&offer.maker, maker.key);
    assert_eq!(&offer.token_a_mint, token_a_mint.key);
    assert_eq!(&offer.token_b_mint, token_b_mint.key);

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

    // Validate the receiving token B accout is owned by the maker (ATA)
    assert_is_associated_token_account(maker_token_b_account.key, maker.key, token_b_mint.key)?;

    // Validate the receiving token A accout is owned by the taker (ATA)
    assert_is_associated_token_account(taker_token_a_account.key, taker.key, token_a_mint.key)?;

    // Create taker token A account if needed, before receiveing tokens
    invoke(
        &associated_token_account_instruction::create_associated_token_account_idempotent(
            payer.key,
            taker.key,
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
            taker_token_a_account.clone(),
            taker.clone(),
            token_a_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Create maker token B account if needed, before receiveing tokens
    invoke(
        &associated_token_account_instruction::create_associated_token_account_idempotent(
            payer.key,
            maker.key,
            token_b_mint.key,
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
            maker_token_b_account.clone(),
            maker.clone(),
            token_b_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(), // not required
        ],
    )?;

    // Read token amount in the offer's vault account
    let vault_amount_a = TokenAccount::unpack(&vault.data.borrow())?.amount;
    // let taker_amount_a_before_transfer =
    //     TokenAccount::unpack(&taker_token_a_account.data.borrow())?.amount;

    // Calculate token B fee amount
    let token_b_fee_amount = escrow_state.get_token_b_fee(offer.token_b_wanted_amount)?;

    // Create escrow fee token B account (escrow state ATA) if needed, before receiveing tokens for fee
    invoke(
        &associated_token_account_instruction::create_associated_token_account_idempotent(
            payer.key,
            &escrow_state_address,
            token_b_mint.key,
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
            escrow_fee_token_b_account.clone(),
            escrow_state_info.clone(),
            token_b_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Transfer fees for token B from taker to escrow fee account for token B
    invoke(
        &token_instruction::transfer(
            token_program.key,
            taker_token_b_account.key,
            escrow_fee_token_b_account.key,
            taker.key,
            &[taker.key],
            token_b_fee_amount,
        )?,
        //   0. `[writable]` The source account.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The source account's owner/delegate.
        &[
            taker_token_b_account.clone(),
            escrow_fee_token_b_account.clone(),
            taker.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer token B from taker (TA) to maker (ATA)
    let token_b_to_transfer_after_fee = offer
        .token_b_wanted_amount
        .checked_sub(token_b_fee_amount)
        .ok_or(EscrowError::MathError)?;
    invoke(
        &token_instruction::transfer(
            token_program.key,
            taker_token_b_account.key,
            maker_token_b_account.key,
            taker.key,
            &[taker.key],
            token_b_to_transfer_after_fee,
        )?,
        //   0. `[writable]` The source account.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The source account's owner/delegate.
        &[
            taker_token_b_account.clone(),
            maker_token_b_account.clone(),
            taker.clone(),
            token_program.clone(),
        ],
    )?;

    // Calculate token A fee amount
    let token_a_fee_amount = escrow_state.get_token_a_fee(vault_amount_a)?;

    // Create escrow fee token A account (escrow state ATA) if needed, before receiveing tokens for fee
    invoke(
        &associated_token_account_instruction::create_associated_token_account_idempotent(
            payer.key,
            &escrow_state_address,
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
            escrow_fee_token_a_account.clone(),
            escrow_state_info.clone(),
            token_a_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Transfer fees for token A from vault to escrow fee account for token A
    invoke_signed(
        &token_instruction::transfer(
            token_program.key,
            vault.key,
            escrow_fee_token_a_account.key,
            offer_info.key,
            &[],
            token_a_fee_amount,
        )?,
        //   0. `[writable]` The source account.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The source account's owner/delegate.
        &[
            vault.clone(),
            escrow_fee_token_a_account.clone(),
            offer_info.clone(),
            token_program.clone(),
        ],
        &[offer_signer_seed],
    )?;

    // Transfer token A from vault (Offer ATA) to taker (ATA)
    let token_a_to_transfer_after_fee = vault_amount_a
        .checked_sub(token_a_fee_amount)
        .ok_or(EscrowError::MathError)?;
    invoke_signed(
        &token_instruction::transfer(
            token_program.key,
            vault.key,
            taker_token_a_account.key,
            offer_info.key,
            &[],
            token_a_to_transfer_after_fee,
        )?,
        //   0. `[writable]` The source account.
        //   1. `[writable]` The destination account.
        //   2. `[signer]` The source account's owner/delegate.
        &[
            vault.clone(),
            taker_token_a_account.clone(),
            offer_info.clone(),
            token_program.clone(), // not required
        ],
        &[offer_signer_seed],
    )?;

    // let taker_amount_a = TokenAccount::unpack(&taker_token_a_account.data.borrow())?.amount;
    // let maker_amount_b = TokenAccount::unpack(&maker_token_b_account.data.borrow())?.amount;
    // let escrow_fee_amount_a =
    //     TokenAccount::unpack(&escrow_fee_token_a_account.data.borrow())?.amount;
    // let escrow_fee_amount_b =
    //     TokenAccount::unpack(&escrow_fee_token_b_account.data.borrow())?.amount;

    // assert_eq!(
    //     taker_amount_a,
    //     taker_amount_a_before_transfer + vault_amount_a - escrow_fee_amount_a
    // );
    // assert_eq!(
    //     maker_amount_b,
    //     taker_amount_a_before_transfer + offer.token_b_wanted_amount - escrow_fee_amount_b
    // );

    // Close the vault account
    invoke_signed(
        &token_instruction::close_account(
            token_program.key,
            vault.key,
            payer.key,
            offer_info.key,
            &[],
        )?,
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
