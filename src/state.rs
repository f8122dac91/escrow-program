use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::{Pubkey, PubkeyError};

use crate::{consts::MAX_BPS_VALUE, errors::EscrowError};

/// Singleton program state that describes the manager authority and escrow fees.
///
/// Also used to hold escrow fee accounts (ATA).
///
/// PDA seed format: [b"state"]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct EscrowState {
    pub manager: Pubkey,
    pub maker_fee_bps: u16,
    pub taker_fee_bps: u16,
    pub bump: u8,
}

impl EscrowState {
    pub const SEED: &'static [u8] = b"state";

    pub fn new(
        program_id: &Pubkey,
        manager: Pubkey,
        maker_fee_bps: u16,
        taker_fee_bps: u16,
    ) -> (Self, Pubkey) {
        let (address, bump) = Self::find_program_address(program_id);
        (
            Self {
                manager,
                maker_fee_bps,
                taker_fee_bps,
                bump,
            },
            address,
        )
    }

    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8) {
        let escrow_state_seed = &[Self::SEED];

        Pubkey::find_program_address(escrow_state_seed, program_id)
    }

    pub fn create_program_address(program_id: &Pubkey, bump: u8) -> Result<Pubkey, PubkeyError> {
        let escrow_state_signer_seed = &[Self::SEED, &[bump]];

        Pubkey::create_program_address(escrow_state_signer_seed, program_id)
    }

    /// Calculate token A (offer token) fee amount.
    ///
    /// The fee is to be levied **from the amount transferred from vault to taker**.
    pub fn get_token_a_fee(&self, amount: u64) -> Result<u64, EscrowError> {
        u128::from(amount)
            .checked_mul(u128::from(self.taker_fee_bps))
            .and_then(|v| v.checked_div(u128::from(MAX_BPS_VALUE)))
            .and_then(|v| u64::try_from(v).ok())
            .ok_or(EscrowError::MathError)
    }

    /// Calculate token B (ask token) fee amount.
    ///
    /// The fee is to be levied **from the amount transferred from taker to maker**.
    pub fn get_token_b_fee(&self, amount: u64) -> Result<u64, EscrowError> {
        u128::from(amount)
            .checked_mul(u128::from(self.maker_fee_bps))
            .and_then(|v| v.checked_div(u128::from(MAX_BPS_VALUE)))
            .and_then(|v| u64::try_from(v).ok())
            .ok_or(EscrowError::MathError)
    }
}

/// Describes each offer made by maker.
///
/// Also used to hold vault (ATA) until escrow offer is taken.
///
///
/// PDA seed format: ["offer", maker_pubkey, offer_id]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Offer {
    pub id: u64,
    pub maker: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_b_wanted_amount: u64,
    pub bump: u8,
}

impl Offer {
    pub const SEED_PREFIX: &'static [u8] = b"offer";

    pub fn new(
        program_id: &Pubkey,
        offer_id: u64,
        maker_pubkey: Pubkey,
        token_a_mint_pubkey: Pubkey,
        token_b_mint_pubkey: Pubkey,
        token_b_wanted_amount: u64,
    ) -> (Self, Pubkey) {
        let (address, bump) = Self::find_program_address(program_id, &maker_pubkey, offer_id);
        (
            Self {
                id: offer_id,
                maker: maker_pubkey,
                token_a_mint: token_a_mint_pubkey,
                token_b_mint: token_b_mint_pubkey,
                token_b_wanted_amount,
                bump,
            },
            address,
        )
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        maker_pubkey: &Pubkey,
        offer_id: u64,
    ) -> (Pubkey, u8) {
        let offer_seed = &[
            Self::SEED_PREFIX,
            maker_pubkey.as_ref(),
            &offer_id.to_le_bytes(),
        ];

        Pubkey::find_program_address(offer_seed, program_id)
    }

    pub fn create_program_address(
        program_id: &Pubkey,
        maker_pubkey: &Pubkey,
        offer_id: u64,
        bump: u8,
    ) -> Result<Pubkey, PubkeyError> {
        let offer_signer_seed = &[
            Self::SEED_PREFIX,
            maker_pubkey.as_ref(),
            &offer_id.to_le_bytes(),
            &[bump],
        ];

        Pubkey::create_program_address(offer_signer_seed, program_id)
    }
}
