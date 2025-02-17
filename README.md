# Solana Native Escrow Program

This is extended example code for on-chain escrow program written in native rust.

The code is unaudited, and only intended for explaining the basics of writing
on-chain program on Solana.


## Testing

`testing` feature is used to override the initial manager keypair in `consts.rs`
for testing `Initialize` ix.


Run below command to start the test suite.

```
cargo test-sbf -- --test-threads=1
```

See `Cargo.toml` and disable `testing` feature for testing against devnet or
testnet environment.
