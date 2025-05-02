use bootcamp3::utils::*;
use solana_sdk::signer::Signer;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let keypair = load_keypair("SECRET_KEY");
    println!("🔑 Key loaded from .env file: {}\n ", keypair.try_pubkey().unwrap());

    let wallet: Pubkey = Pubkey::from_str(load_key_from_env("SOME_PUBKEY").as_str()).unwrap();
    process_transaction_result("💰 Send Sol", send_sol(&keypair, &wallet, 20_000_000));
}

// cargo run --bin 2-send-sol
