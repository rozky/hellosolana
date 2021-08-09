use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::signature::read_keypair_file;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::transaction::Transaction;

static CLUSTER_URL: &str = "http://localhost:8899";
static SOURCE_WALLET_PATH: &str = "~/.config/solana/id.json";
static TARGET_WALLET_PATH: &str = "~/dev/personal/wallets/test-1.json";
static AMOUNT: u64 = 75000;

// transfer money between same account ? is that even possible , yes you can and I will cost you 5000 pretty much for nothing
fn main() {
    if let Err(err) = run(SOURCE_WALLET_PATH, TARGET_WALLET_PATH, AMOUNT) {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}

fn run(source: &str, target: &str, amount: u64) -> Result<(), ClientError> {
    // (1) Load 2 filesystem wallets
    let source_wallet = get_wallet(source);
    let target_wallet = get_wallet(target);

    // (2) Create RPC client to be used to talk to Solana cluster
    let rpc = create_client();

    // (3) Get an initial balance for both accounts
    let source_balance = get_balance(&rpc, &source_wallet)?;
    let target_balance = get_balance(&rpc, &target_wallet)?;

    // (4) Build transfer instruction
    let instruction = solana_sdk::system_instruction::transfer(
        &source_wallet.pubkey(),
        &target_wallet.pubkey(),
        amount,
    );

    // (5) Build a transaction wrapping the transfer instruction
    // Note that only transaction only need to be signed by the source account
    let signers = [&source_wallet];
    let instructions = vec![instruction];

    let (recent_hash, _) = rpc.get_recent_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&source_wallet.pubkey()),
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

    println!("source lost = {}", (source_balance - get_balance(&rpc, &source_wallet)?));
    println!("target gained = {}",  get_balance(&rpc, &target_wallet)? - target_balance);

    Ok(())
}

fn create_client() -> RpcClient {
    RpcClient::new_with_commitment(
        CLUSTER_URL.to_string(),
        CommitmentConfig::confirmed(),
    )
}

fn get_wallet(path: &str) -> Keypair {
    read_keypair_file(&*shellexpand::tilde(path))
        .expect("Example requires a keypair file")
}

fn get_balance(rpc: &RpcClient, wallet: &Keypair) -> Result<u64, ClientError> {
    rpc.get_account(&wallet.pubkey()).map(|b| b.lamports)
}