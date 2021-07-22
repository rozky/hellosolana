use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::signature::read_keypair_file;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), ClientError> {
    let cluster_url = "http://localhost:8899".to_string();

    // (1) Load your wallet account from filesystem (from default location)
    let wallet = read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
        .expect("Example requires a keypair file");

    // (2) Create a new Keypair for the new account
    let account = Keypair::new();
    println!("new account address = {}", account.pubkey());

    // (3) Create RPC client to be used to talk to Solana cluster
    let rpc = RpcClient::new_with_commitment(cluster_url, CommitmentConfig::confirmed());

    let balance = rpc.get_account(&wallet.pubkey())?.lamports;

    // (4) Number of bytes to allocate for the new account data
    let space = 0;

    // (5) Calculate min rent according to expected account data size
    let rent = rpc.get_minimum_balance_for_rent_exemption(space)?;

    // (6) Build create account instruction
    let instruction = solana_sdk::system_instruction::create_account(
        &wallet.pubkey(),
        &account.pubkey(),
        rent,
        space as u64,
        &wallet.pubkey(),
    );

    // (7) Build transaction wrapping the client account transaction
    let signers = [&wallet, &account];
    let instructions = vec![instruction];

    let (recent_hash, _) = rpc.get_recent_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    // (8) Send transaction to the cluster and wait for confirmation
    let result = rpc.send_and_confirm_transaction_with_spinner_and_config(
        &txn,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
    );

    println!("new account balance = {}", rpc.get_account(&account.pubkey())?.lamports);
    println!("cost = {}", balance - rpc.get_account(&wallet.pubkey())?.lamports);

    Ok(())
}