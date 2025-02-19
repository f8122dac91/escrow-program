use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
    spl_associated_token_account::get_associated_token_address,
};

use crate::{
    instructions::{
        collect_fee::CollectFeeArgs, initialize::InitializeArgs, make_offer::MakeOfferArgs,
        set_fees::SetFeesArgs,
    },
    state::{EscrowState, Offer},
};

pub mod cancel_offer;
pub mod collect_fee;
pub mod initialize;
pub mod make_offer;
pub mod set_fees;
pub mod set_manager;
pub mod take_offer;

/// Declares all available instructions of the escrow program.
///
/// Comments defines the expected accounts for each instruction type.
/// The expected type of argument are declared for instructions that requires argument.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowInstruction {
    // Manager-facing instructions

    // Initialize the program state (Escrow State)
    //
    // 0. `[writeable]` (PDA) Escrow state account to be initialized
    // 1. `[signer]` Initial manager (specified in consts.rs)
    // 3. `[writeable,signer]` Funding account
    // 2. `[]` System program
    Initialize(InitializeArgs),

    // Set programwide fees (maker fee and taker fee) in the program state
    //
    // 0. `[writeable]` (PDA) Escrow state account
    // 1. `[signer]` Manager
    SetFees(SetFeesArgs),

    // Set programwide manager in the program state
    //
    // 0. `[writeable]` (PDA) Escrow state account
    // 1. `[signer]` Manager
    // 2. `[]` New manager
    SetManager,

    // Collect tokens in escrow fee account
    //
    // 0. `[]` (PDA) Escrow state account
    // 1. `[signer(,writeable)]` Manager (writeable if `should_close_fee_account` is set)
    // 2. `[]` Mint account for escrow fee
    // 3. `[writeable]` Source escrow fee account (ATA of escrow state)
    // 4. `[writeable]` Destination token account
    // 5. `[]` SPL Token program
    CollectFee(CollectFeeArgs),

    // User-facing instructions

    // Make escrow offer
    //
    // 0. `[writeable]` (PDA) Escrow offer account to be initialized
    // 1. `[]` Token A (maker's token) mint account for the escrow offer
    // 2. `[]` Token B (taker's token) mint account for the escrow offer
    // 3. `[writeable]` Maker's token A account for the escrow offer
    // 4. `[writeable]` (PDA) Escrow offer's vault token account (Token A, ATA of Offer account)
    // 5. `[signer]` Maker's wallet address
    // 6. `[writeable,signer]` Funding account
    // 7. `[]` SPL Token program
    // 8. `[]` SPL Associated Token Account program
    // 9. `[]` System program
    MakeOffer(MakeOfferArgs),

    // Take escrow offer
    //
    // 0. `[]` (PDA) Escrow state account
    // 1. `[writeable]` (PDA) Escrow offer account to be taken
    // 2. `[]` Token A (maker's token) mint account for the escrow offer
    // 3. `[]` Token B (taker's token) mint account for the escrow offer
    // 4. `[writeable]` Maker's token B account to receive from taker (ATA)
    // 5. `[writeable]` Taker's token A account to receive from vault (ATA)
    // 6. `[writeable]` Taker's token B account to send to maker
    // 7. `[writeable]` Escrow state's Token A account for fee collection (ATA of Escrow state)
    // 8. `[writeable]` Escrow state's Token B account for fee collection (ATA of Escrow state)
    // 9. `[writeable]` (PDA) Escrow offer's vault token account (Token A, ATA of Offer account)
    // 10. `[]` Maker's wallet address
    // 11. `[signer]` Taker's wallet address
    // 12. `[writeable,signer]` Funding account
    // 13. `[]` SPL Token program
    // 14. `[]` SPL Associated Token Account program
    // 15. `[]` System program
    TakeOffer,

    // Cancel escrow offer
    //
    // 0. `[writeable]` (PDA) Escrow offer account to be canceled
    // 1. `[]` Token A (maker's token) mint account for the escrow offer
    // 2. `[writeable]` Maker's token A account to refund to (Token A, ATA)
    // 3. `[writeable]` (PDA) Escrow offer's vault token account (Token A, ATA of Offer account)
    // 4. `[signer]` Maker's wallet address
    // 5. `[writeable,signer]` Funding account
    // 6. `[]` SPL Token program
    // 7. `[]` SPL Associated Token Account program
    // 8. `[]` System program
    CancelOffer,
}

//
// Helper functions for creating instructions
//
pub fn initialize_ix(
    initial_manager_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
    initialize_args: InitializeArgs,
) -> Instruction {
    let (escrow_state_address, _) = EscrowState::find_program_address(&crate::ID);

    let accounts = vec![
        AccountMeta::new(escrow_state_address, false),
        AccountMeta::new_readonly(*initial_manager_pubkey, true),
        AccountMeta::new(*payer_pubkey, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let instruction_data = EscrowInstruction::Initialize(initialize_args);

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}

pub fn set_fees_ix(manager_pubkey: &Pubkey, set_fees_args: SetFeesArgs) -> Instruction {
    let (escrow_state_address, _) = EscrowState::find_program_address(&crate::ID);

    let accounts = vec![
        AccountMeta::new(escrow_state_address, false),
        AccountMeta::new_readonly(*manager_pubkey, true),
    ];

    let instruction_data = EscrowInstruction::SetFees(set_fees_args);

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}

pub fn set_manager_ix(manager_pubkey: &Pubkey, new_manager_pubkey: &Pubkey) -> Instruction {
    let (escrow_state_address, _) = EscrowState::find_program_address(&crate::ID);

    let accounts = vec![
        AccountMeta::new(escrow_state_address, false),
        AccountMeta::new_readonly(*manager_pubkey, true),
        AccountMeta::new_readonly(*new_manager_pubkey, false),
    ];

    let instruction_data = EscrowInstruction::SetManager {};

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}

pub fn collect_fee_ix(
    manager_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_token_account_pubkey: &Pubkey,
    collect_fee_args: CollectFeeArgs,
) -> Instruction {
    let (escrow_state_address, _) = EscrowState::find_program_address(&crate::ID);

    let escrow_fee_account = get_associated_token_address(&escrow_state_address, mint_pubkey);

    let manager_account_meta = if collect_fee_args.should_close_fee_account {
        // Set to writeable (rent destination)
        AccountMeta::new(*manager_pubkey, true)
    } else {
        AccountMeta::new_readonly(*manager_pubkey, true)
    };
    let accounts = vec![
        AccountMeta::new_readonly(escrow_state_address, false),
        manager_account_meta,
        AccountMeta::new_readonly(*mint_pubkey, false),
        AccountMeta::new(escrow_fee_account, false),
        AccountMeta::new(*destination_token_account_pubkey, false),
        AccountMeta::new_readonly(spl_token::ID, false),
    ];

    let instruction_data = EscrowInstruction::CollectFee(collect_fee_args);

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}

pub fn make_offer_ix(
    maker_pubkey: &Pubkey,
    maker_token_a_account_pubkey: &Pubkey,
    token_a_mint_pubkey: &Pubkey,
    token_b_mint_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
    make_offer_args: MakeOfferArgs,
) -> Instruction {
    let (offer_address, _) =
        Offer::find_program_address(&crate::ID, maker_pubkey, make_offer_args.id);

    let vault_pubkey = get_associated_token_address(&offer_address, token_a_mint_pubkey);

    let accounts = vec![
        AccountMeta::new(offer_address, false),
        AccountMeta::new_readonly(*token_a_mint_pubkey, false),
        AccountMeta::new_readonly(*token_b_mint_pubkey, false),
        AccountMeta::new(*maker_token_a_account_pubkey, false),
        AccountMeta::new(vault_pubkey, false),
        AccountMeta::new_readonly(*maker_pubkey, true),
        AccountMeta::new(*payer_pubkey, true),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let instruction_data = EscrowInstruction::MakeOffer(make_offer_args);

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}

pub fn take_offer_ix(
    offer_pubkey: &Pubkey,
    token_a_mint_pubkey: &Pubkey,
    token_b_mint_pubkey: &Pubkey,
    // maker_token_b_account_pubkey: &Pubkey, // use ATA
    // taker_token_a_account_pubkey: &Pubkey, // use ATA
    taker_token_b_account_pubkey: &Pubkey,
    maker_pubkey: &Pubkey,
    taker_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
) -> Instruction {
    let (escrow_state_address, _) = EscrowState::find_program_address(&crate::ID);

    let vault_pubkey = get_associated_token_address(offer_pubkey, token_a_mint_pubkey);
    let maker_token_b_account_pubkey =
        get_associated_token_address(maker_pubkey, token_b_mint_pubkey);
    let taker_token_a_account_pubkey =
        get_associated_token_address(taker_pubkey, token_a_mint_pubkey);
    let escrow_fee_token_a_account_pubkey =
        get_associated_token_address(&escrow_state_address, token_a_mint_pubkey);
    let escrow_fee_token_b_account_pubkey =
        get_associated_token_address(&escrow_state_address, token_b_mint_pubkey);

    let accounts = vec![
        AccountMeta::new_readonly(escrow_state_address, false),
        AccountMeta::new(*offer_pubkey, false),
        AccountMeta::new_readonly(*token_a_mint_pubkey, false),
        AccountMeta::new_readonly(*token_b_mint_pubkey, false),
        AccountMeta::new(maker_token_b_account_pubkey, false),
        AccountMeta::new(taker_token_a_account_pubkey, false),
        AccountMeta::new(*taker_token_b_account_pubkey, false),
        AccountMeta::new(escrow_fee_token_a_account_pubkey, false),
        AccountMeta::new(escrow_fee_token_b_account_pubkey, false),
        AccountMeta::new(vault_pubkey, false),
        AccountMeta::new_readonly(*maker_pubkey, false),
        AccountMeta::new_readonly(*taker_pubkey, true),
        AccountMeta::new(*payer_pubkey, true),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let instruction_data = EscrowInstruction::TakeOffer {};

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}

pub fn cancel_offer_ix(
    maker_pubkey: &Pubkey,
    // maker_token_a_account_pubkey: &Pubkey, // use ATA
    token_a_mint_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
    offer_id: u64,
) -> Instruction {
    let (offer_address, _) = Offer::find_program_address(&crate::ID, maker_pubkey, offer_id);

    let maker_token_a_account_pubkey =
        get_associated_token_address(maker_pubkey, token_a_mint_pubkey);
    let vault_pubkey = get_associated_token_address(&offer_address, token_a_mint_pubkey);

    let accounts = vec![
        AccountMeta::new(offer_address, false),
        AccountMeta::new_readonly(*token_a_mint_pubkey, false),
        AccountMeta::new(maker_token_a_account_pubkey, false),
        AccountMeta::new(vault_pubkey, false),
        AccountMeta::new_readonly(*maker_pubkey, true),
        AccountMeta::new(*payer_pubkey, true),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    let instruction_data = EscrowInstruction::CancelOffer {};

    Instruction::new_with_borsh(crate::ID, &instruction_data, accounts)
}
