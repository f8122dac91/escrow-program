use solana_program::entrypoint;

use crate::processor::process_instruction;

// Defines the function to be invoked when program is called on-chain
entrypoint!(process_instruction);
