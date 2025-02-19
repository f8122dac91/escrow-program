use {
    borsh::BorshDeserialize,
    solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey},
};

use crate::instructions::*;

/// Entrypoint function that processes instructions
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::try_from_slice(instruction_data)?;

    msg!("Instruction: {:?}", instruction);

    match instruction {
        EscrowInstruction::Initialize(args) => initialize::process(program_id, accounts, args),
        EscrowInstruction::SetFees(args) => set_fees::process(program_id, accounts, args),
        EscrowInstruction::SetManager => set_manager::process(program_id, accounts),
        EscrowInstruction::CollectFee(args) => collect_fee::process(program_id, accounts, args),
        EscrowInstruction::MakeOffer(args) => make_offer::process(program_id, accounts, args),
        EscrowInstruction::TakeOffer => take_offer::process(program_id, accounts),
        EscrowInstruction::CancelOffer => cancel_offer::process(program_id, accounts),
    }
}
