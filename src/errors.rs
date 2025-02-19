//! Defines custom errors that the program returns as ProgramError::Custom(u32)
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EscrowError {
    #[error("Escrow state key provided does not match expected")]
    EscrowStateKeyMismatch,

    #[error("Initial manager key provided does not match expected")]
    InitialManagerKeyMismatch,

    #[error("Basis point value provided exceeded allowed range")]
    MaxBpsValueExceeded,

    #[error("Manager key provided is not authorized")]
    ManagerKeyUnauthorized,

    #[error("Manager key provided is the current manager")]
    ManagerKeyAlreadySet,

    #[error("Offer key provided does not match expected")]
    OfferKeyMismatch,

    #[error("Token account provided does not match expected")]
    TokenAccountMismatch,

    #[error("Argument provided resulted in overflow")]
    MathError,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
