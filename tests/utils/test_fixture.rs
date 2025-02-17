use std::{
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccount;
use solana_sdk::{account::Account, pubkey::Pubkey};

/// This is the json format of
/// `solana account -o <FILENAME>.json --output json <ACCOUNT-PUBKEY>`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestFixture {
    pub pubkey: String,
    pub account: UiAccount,
}

impl TestFixture {
    pub fn from_file<P: AsRef<Path>>(json_file_path: P) -> Self {
        let mut file = File::open(json_file_path).unwrap();
        serde_json::from_reader(&mut file).unwrap()
    }

    /// Loads a TestFixture from `<test_fixtures_dir()>/relative_json_file_path`
    pub fn from_test_fixtures_file<P: AsRef<Path>>(relative_json_file_path: P) -> Self {
        Self::from_file(Self::test_fixtures_dir().join(relative_json_file_path))
    }

    pub fn to_address_and_account(&self) -> (Pubkey, Account) {
        (
            Pubkey::from_str(&self.pubkey).unwrap(),
            self.account.decode().unwrap(),
        )
    }
    /// Returns `/path/to/workspace/root/test-fixtures`
    fn test_fixtures_dir() -> PathBuf {
        workspace_root_dir().join("test-fixtures")
    }
}

/// use cargo to get the absolute path of workspace root
fn workspace_root_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}
