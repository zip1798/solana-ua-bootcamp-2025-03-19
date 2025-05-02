use bootcamp3::utils::*;
use solana_sdk::signer::Signer;
use solana_sdk::signer::keypair::Keypair;

#[tokio::main]
async fn main() {
    let keypair = load_keypair("SECRET_KEY");
    println!("🔑 Key loaded from .env file: {}\n ", keypair.try_pubkey().unwrap());

    let mint_keypair = Keypair::new();
    process_transaction_result("✅ Create token mint", create_mint(&keypair, &mint_keypair));
    println!("💸 Mint pubkey: {}\n ", mint_keypair.try_pubkey().unwrap());
}

// cargo run --bin 2-create-token-mint
