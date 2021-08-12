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
/// - solana-tests-validator
/// - anchor build
/// - anchor deploy

/// You can explore your Local Net using https://explorer.solana.com/ just connect it
/// to http://localhost:8899

// TODO - add example for associated token account
#[test]
fn open_native_token_account_example() {
    let rpc = rpc_client();
    let fee_payer = get_filesystem_wallet(FEE_PAYER_FILESYSTEM_WALLET_PATH);
    let token_account = new_keys();

    // let 1st create some native token account
    create_native_token_account(
        &rpc,
        &fee_payer,
        &token_account,
        TOKEN_ACCOUNT_BALANCE_SOLS,
    ).unwrap();

    // the token account should have expected balance
    println!(
        "token account[{:?}] balance = {}",
        token_account.pubkey(),
        get_balance(&rpc, &token_account).unwrap()
    );
}

#[test]
fn close_native_token_account_example() {
    let rpc = rpc_client();
    let fee_payer = get_filesystem_wallet(FEE_PAYER_FILESYSTEM_WALLET_PATH);
    let token_account = new_keys();

    // let 1st create some native token account
    create_native_token_account(
        &rpc,
        &fee_payer,
        &token_account,
        TOKEN_ACCOUNT_BALANCE_SOLS,
    ).unwrap();

    // now let's close the token account and transfer it's balance to a new account
    let recipient = new_keys();
    close_native_token_account(
        &rpc,
        &token_account.pubkey(),
        &fee_payer,
        &recipient.pubkey(),
    ).unwrap();

    // full balance of the token account should have been transferred to the new account
    println!(
        "recipient account[{:?}] balance = {}",
        token_account.pubkey(),
        get_balance(&rpc, &recipient).unwrap()
    );
}

fn create_native_token_account(
    rpc: &RpcClient,
    fee_payer: &Keypair,
    token_account: &Keypair,
    balance: f64,
) -> Result<Signature, ClientError> {
    let spl_token_program_id = &spl_token::id();
    let native_token_mint_id = &spl_token::native_mint::id();

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

    let signings = &[fee_payer, token_account];
    send_instructions(&rpc, &fee_payer.pubkey(), &instructions, signings)
}

fn close_native_token_account(
    rpc: &RpcClient,
    token_account: &Pubkey,
    token_account_owner: &Keypair,
    token_account_balance_recipient: &Pubkey,
) -> Result<Signature, ClientError> {
    let spl_token_program_id = &spl_token::id();

    let instructions = vec![
        spl_token::instruction::close_account(
            spl_token_program_id,
            &token_account,
            token_account_balance_recipient,
            &token_account_owner.pubkey(),
            &[&token_account_owner.pubkey()],
        )
            .unwrap(),
    ];

    let signings = &[token_account_owner];
    send_instructions(&rpc, &token_account_owner.pubkey(), &instructions, signings)
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

fn rpc_client() -> RpcClient {
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