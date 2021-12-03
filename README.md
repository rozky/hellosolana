# Hello Solana

Collection of mini programs connecting to Solana blockchain using Rust RPC 
Client. Showcasing basic operations like creating account, transferring SOLs between accounts, etc. 

## TODO

// todo transfer money out of empty account ?
// todo print error when creating account ?
// todo is there trans fee associated with creating an account ?
// todo - impact on transfer call if account is owned by system program or payer account ?

## Working with cargo

### Manually search for dependencies

* https://lib.rs/
* https://crates.io/
* https://lib.rs/crates/solana-sdk
* https://lib.rs/crates/solana-client
* https://lib.rs/crates/solana-program
* https://lib.rs/crates/anchor-client (documentation: https://docs.rs/anchor-client/0.18.2/anchor_client/)
* https://lib.rs/crates/anchor-lang


### Analyse dependencies issues

```bash
cargo tree > tree.txt
```

### Keep dependencies up to date

```bash
# https://crates.io/crates/cargo-outdated
cargo install --locked cargo-outdated

## once installed, I can run
cargo outdated
```