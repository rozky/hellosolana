use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

// The most basic example for RpcClient usage.
// Just connects to the local cluster (started by "solana-test-validator") and send some RPC
// requests to it.
fn main() {
    if let Err(err) = run() {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), ClientError> {
    let cluster_url = "http://localhost:8899".to_string();

    // (1) Create RPC client to be used to talk to Solana cluster
    let rpc = RpcClient::new_with_commitment(cluster_url, CommitmentConfig::confirmed());


    // (2) Make some calls with the client
    println!("rent[0b] = {}", rpc.get_minimum_balance_for_rent_exemption(0)?);
    println!("rent[128b] = {}", rpc.get_minimum_balance_for_rent_exemption(128)?);
    println!("transaction count = {}", rpc.get_transaction_count()?);
    println!("epoch schedule = {:?}", rpc.get_epoch_schedule()?);
    println!("fees = {:?}", rpc.get_fees()?);
    println!("signature fee = {}", rpc.get_fees()?.fee_calculator.lamports_per_signature);

    Ok(())
}