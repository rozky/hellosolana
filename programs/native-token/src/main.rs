use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::signature::read_keypair_file;
use rand::rngs::OsRng;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::program_pack::Pack;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;

static CLUSTER_URL: &str = "http://localhost:8899";
static FEE_PAYER_FILESYSTEM_WALLET_PATH: &str = "~/.config/solana/id.json";
static TOKEN_ACCOUNT_BALANCE_SOLS: f64 = 5 as f64;

/// Important: Make sure to run a "Local Net" with the program deploy to run this tests.
/// - solana-test-validator
/// - anchor build
/// - anchor deploy

/// You can explore your Local Net using https://explorer.solana.com/ just connect it
/// to http://localhost:8899

fn main() {
    let rpc = new_client();
    let token_account = new_keys();

    if let Err(err) = create_native_token_account(
        &rpc,
        FEE_PAYER_FILESYSTEM_WALLET_PATH,
        &token_account,
        TOKEN_ACCOUNT_BALANCE_SOLS,
    ) {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }

    println!(
        "token account[{:?}] balance = {}",
        token_account.pubkey(),
        get_balance(&rpc, &token_account).unwrap()
    );
}

fn create_native_token_account(
    rpc: &RpcClient,
    fee_payer_filesystem_wallet: &str,
    token_account: &Keypair, balance: f64,
) -> Result<Signature, ClientError> {
    let spl_token_program_id = &spl_token::id();
    let native_token_mint_id = &spl_token::native_mint::id();

    let fee_payer = get_filesystem_wallet(fee_payer_filesystem_wallet);

    let instructions = vec![
        create_account_instruction(
            &rpc,
            &fee_payer.pubkey(),
            &token_account.pubkey(),
            spl_token::state::Account::LEN,
            spl_token_program_id,
            solana_sdk::native_token::sol_to_lamports(balance),
        ),
        spl_token::instruction::initialize_account(
            spl_token_program_id,
            &token_account.pubkey(),
            native_token_mint_id,
            &fee_payer.pubkey(),
        )
            .unwrap(),
    ];

    let signings = &[&fee_payer, &token_account];
    send_instructions(&rpc, &fee_payer.pubkey(), &instructions, signings)
}


///
/// Instruction builders
///

fn create_account_instruction(
    rpc: &RpcClient,
    fee_payer: &Pubkey,
    new_account: &Pubkey,
    new_account_space: usize,
    new_account_owner: &Pubkey,
    new_account_balance: u64,
) -> Instruction {
    let rent_exempt_balance = rpc
        .get_minimum_balance_for_rent_exemption(new_account_space)
        .unwrap();

    solana_sdk::system_instruction::create_account(
        fee_payer,
        new_account,
        new_account_balance + rent_exempt_balance,
        new_account_space as u64,
        new_account_owner,
    )
}

///
/// RPC Client helpers
///

fn new_client() -> RpcClient {
    RpcClient::new_with_commitment(
        CLUSTER_URL.to_string(),
        CommitmentConfig::confirmed(),
    )
}

fn send_instructions<T: Signers>(
    rpc: &RpcClient,
    fee_payer: &Pubkey,
    instructions: &[Instruction],
    signings: &T,
) -> Result<Signature, ClientError> {
    let (recent_hash, _) = rpc.get_recent_blockhash().unwrap();

    let txn =
        Transaction::new_signed_with_payer(&instructions, Some(&fee_payer), signings, recent_hash);

    send_tx(&rpc, &txn)
}

fn send_tx(rpc: &RpcClient, tx: &Transaction) -> Result<Signature, ClientError> {
    rpc.send_and_confirm_transaction_with_spinner_and_config(
        &tx,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
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