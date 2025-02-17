use std::path::Path;

use escrow_program::state::{EscrowState, Offer};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    account::Account, program_option::COption, program_pack::Pack, pubkey::Pubkey, rent::Rent,
};
use spl_token::state::{Account as TokenAccount, AccountState as TokenAccountState};

use super::TestFixture;

/// Load escrow program into ProgramTest.
pub fn prepare_program_test() -> ProgramTest {
    ProgramTest::new(
        "escrow_program",
        escrow_program::ID,
        processor!(escrow_program::processor::process_instruction),
    )
}

/// Load given EscrowState into ProgramTest.
pub fn add_escrow_state_account(program_test: &mut ProgramTest, escrow_state: EscrowState) {
    let address =
        EscrowState::create_program_address(&escrow_program::ID, escrow_state.bump).unwrap();
    let data = borsh::to_vec::<EscrowState>(&escrow_state).unwrap();
    let lamports = Rent::default().minimum_balance(data.len());
    let account = Account {
        lamports,
        data,
        owner: escrow_program::ID,
        executable: false,
        rent_epoch: u64::MAX,
    };

    program_test.add_account(address, account);
}

/// Load given Offer into ProgramTest.
pub fn add_offer_account(program_test: &mut ProgramTest, offer: Offer) {
    let address =
        Offer::create_program_address(&escrow_program::ID, &offer.maker, offer.id, offer.bump)
            .unwrap();
    let data = borsh::to_vec::<Offer>(&offer).unwrap();
    let lamports = Rent::default().minimum_balance(data.len());
    let account = Account {
        lamports,
        data,
        owner: escrow_program::ID,
        executable: false,
        rent_epoch: u64::MAX,
    };

    program_test.add_account(address, account);
}

pub fn add_token_account(
    program_test: &mut ProgramTest,
    address: Pubkey,
    mint_pubkey: Pubkey,
    owner_pubkey: Pubkey,
    balance: u64,
) {
    let token_account = TokenAccount {
        mint: mint_pubkey,
        owner: owner_pubkey,
        amount: balance,
        delegate: COption::None,
        state: TokenAccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let mut data = vec![0u8; TokenAccount::LEN];
    token_account.pack_into_slice(&mut data);
    let lamports = Rent::default().minimum_balance(data.len());
    let account = Account {
        lamports,
        data,
        owner: spl_token::ID,
        executable: false,
        rent_epoch: u64::MAX,
    };

    program_test.add_account(address, account);
}

/// Load given test fixture file into ProgramTest.
///
/// Returns address of the account created from the given test fixture.
///
/// NOTE: assumes directory `test-fixtures/` exists in the project root.
pub fn add_test_fixture_from_file<P: AsRef<Path>>(
    program_test: &mut ProgramTest,
    relative_json_file_path: P,
) -> Pubkey {
    let (address, account) =
        TestFixture::from_test_fixtures_file(relative_json_file_path).to_address_and_account();

    program_test.add_account(address, account);

    address
}
