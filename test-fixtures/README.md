# test-fixtures

Accounts cloned from mainnnet to load into `ProgramTest`.[^program_test]

[^program_test]: Reference [`ProgramTest`](https://docs.rs/solana-program-test/2.0.22/solana_program_test/struct.ProgramTest.html)

## JSON format

Format is `{ account: solana_account_decoder::UiAccount, pubkey: string }`.[^account_decoder]

[^account_decoder]: Reference [`solana_account_decoder::UiAccount`](https://docs.rs/solana-account-decoder/latest/solana_account_decoder/struct.UiAccount.html)

Example:

```json
{
  "pubkey": "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy",
  "account": {
    "lamports": 1141440,
    "data": ["AgAAAMycX43n4SCSmKum8wpo7i+bnBwt/dPJe1JGXP6NSmNK", "base64"],
    "owner": "BPFLoaderUpgradeab1e11111111111111111111111",
    "executable": true,
    "rentEpoch": 231,
    "space": 36
  }
}
```

Deserialize into `solana_sdk::Account`[^account] using `.decode()`[^decode] (`WritableAccount` trait[^writable_account])).

[^account]: Reference [`solana_sdk::Account`](https://docs.rs/solana-sdk/2.0.22/solana_sdk/account/struct.Account.html)
[^decode]: Reference  [`.decode()`](https://docs.rs/solana-account-decoder/latest/solana_account_decoder/struct.UiAccount.html#method.decode)
[^writable_account]: Reference [`trait WritableAccount`](https://docs.rs/solana-sdk/2.0.22/solana_sdk/account/trait.WritableAccount.html))

## Cloning from mainnet

Set solana cli config to mainnet, then in workspace root:

```sh
solana account --output-file test-fixtures/<NEW-FILENAME>.json --output json <ACCOUNT-PUBKEY>
```

## Why not use the solana-program-test built-in fixtures feature?

- json format for better human-readability and easy manipulation of pubkey, owner etc
- more flexibility - some tests don't want/need to load all accounts in this folder
