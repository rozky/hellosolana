use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::signature::read_keypair_file;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::transaction::Transaction;
use rand::rngs::OsRng;

static CLUSTER_URL: &str = "http://localhost:8899";
static FEE_PAYER_FILESYSTEM_WALLET_PATH: &str = "~/.config/solana/id.json";
static AMOUNT: u64 = 75000;

// transfer money between same account ? is that even possible , yes you can and I will cost you 5000 pretty much for nothing
fn main() {
    if let Err(err) = run(FEE_PAYER_FILESYSTEM_WALLET_PATH, AMOUNT) {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}

fn run(source: &str, amount: u64) -> Result<(), ClientError> {
    // (1) Load 2 filesystem wallets
    let fee_payer_wallet = get_filesystem_wallet(source);
    let recipient_wallet = new_keys();

    // (2) Create RPC client to be used to talk to Solana cluster
    let rpc = rpc_client();

    // (3) Get an initial balance for both accounts
    let fee_payer_balance = get_balance(&rpc, &fee_payer_wallet)?;

    // (4) Build transfer instruction
    let instruction = solana_sdk::system_instruction::transfer(
        &fee_payer_wallet.pubkey(),
        &recipient_wallet.pubkey(),
        amount,
    );

    // (5) Build a transaction wrapping the transfer instruction
    // Note that only transaction only need to be signed by the source account
    let signers = [&fee_payer_wallet];
    let instructions = vec![instruction];

    let (recent_hash, _) = rpc.get_recent_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&fee_payer_wallet.pubkey()),
        &signers,
        recent_hash,
    );

    // (6) Send transaction to the cluster and wait for confirmation
    rpc.send_and_confirm_transaction_with_spinner_and_config(
        &txn,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
    )?;

    println!("fee payer balance delta  = {}", (fee_payer_balance - get_balance(&rpc, &fee_payer_wallet)?));
    println!("recipient balance = {}",  get_balance(&rpc, &recipient_wallet)?);

    Ok(())
}

///
/// RPC Client helpers
///

fn rpc_client() -> RpcClient {
    RpcClient::new_with_commitment(
        CLUSTER_URL.to_string(),
        CommitmentConfig::confirmed(),
    )
}

fn get_balance(rpc: &RpcClient, wallet: &Keypair) -> Result<u64, ClientError> {
    rpc.get_account(&wallet.pubkey()).map(|b| b.lamports)
}

///
/// Other Helpers
///

fn get_filesystem_wallet(wallet_path: &str) -> Keypair {
    read_keypair_file(&*shellexpand::tilde(wallet_path)).unwrap()
}

fn new_keys() -> Keypair {
    Keypair::generate(&mut OsRng)
}