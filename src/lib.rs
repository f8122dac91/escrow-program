use solana_program::declare_id;

declare_id!("DNkSkUvP9fYkRJi9HV4B584xdNxwPBrmzubThtkU6yA3");

pub mod consts;
pub mod errors;
pub mod instructions;
pub mod processor;
pub mod state;
mod utils;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
