// Load an existing keypair from .env file
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use dotenvy::dotenv;
use std::env;


use solana_client::rpc_client::RpcClient;
use std::str::FromStr;

use solana_sdk::{
    transaction::Transaction,
    instruction::{AccountMeta, Instruction},
    system_instruction, 
    system_program,
};

use spl_token::instruction as token_instruction;
use spl_token::state::Mint;
use solana_sdk::program_pack::Pack;

use spl_associated_token_account::instruction::create_associated_token_account;
use spl_associated_token_account::get_associated_token_address_with_program_id;


use mpl_token_metadata::types::PrintSupply;
use mpl_token_metadata::types::TokenStandard;

const RPC_URL: &str = "https://api.devnet.solana.com";

/// Loads a keypair from a specified environment variable containing a serialized secret key.
/// 
/// This function reads a `.env` file to retrieve the value of the specified environment variable.
/// The secret key is expected to be a string representation of a byte array, formatted as a list
/// of comma-separated integers enclosed in square brackets (e.g., "[1, 2, 3, ...]"). The function
/// removes the enclosing brackets, parses the integers into bytes, and constructs a `Keypair`.
/// 
/// # Arguments
/// 
/// * `env_key` - A string slice that holds the name of the environment variable to read the secret key from.
/// 
/// # Returns
/// 
/// A `Keypair` constructed from the parsed secret key.
/// 
/// # Panics
/// 
/// The function will panic if the environment variable is not set, if the secret key cannot be
/// parsed into a valid byte array, or if the length of the parsed byte array is incorrect.

pub fn load_keypair(env_key: &str) -> Keypair {
    // Load the .env file
    dotenv().ok();

    let secret_key = env::var(env_key).expect(format!("{} must be set", env_key).as_str());

    // Remove the brackets and split the string by commas
    let trimmed = secret_key.trim_matches(&['[', ']'][..]);
    let bytes: Vec<u8> = trimmed
        .split(',')
        .filter_map(|s| s.trim().parse::<u8>().ok()) // Parse each component into u8
        .collect(); // Collect into a Vec<u8>

    // If you need a fixed-size array, you can convert it (only if the length is known)
    let secret_key_byte_array: [u8; 64] = bytes.try_into().expect("Incorrect length");
    let keypair: Keypair = Keypair::from_bytes(&secret_key_byte_array).expect("Invalid secret key");

    keypair
}

pub fn load_key_from_env(env_key: &str) -> String {
    dotenv().ok();

    let value = env::var(env_key).expect(format!("{} must be set", env_key).as_str());

    value
}

pub fn test() {
    println!("Hello, world TEST!");

    dotenv().ok();
    let a = dotenvy::dotenv_iter().unwrap();
    // let mut a = a.collect::<Vec<_>>();
    // println!("{:?}", a);
    

    for item in a {
         let (key, val) = item.unwrap();
        println!("{}={}\n", key, val);
    }

}


pub fn send_sol(keypair: &Keypair, wallet: &Pubkey, amount: u64) -> Result<Signature, solana_client::client_error::ClientError> {
    let client = RpcClient::new(RPC_URL);

    // Create the transfer instruction
    let instruction = system_instruction::transfer(&keypair.pubkey(), wallet, amount);

    // Get the recent blockhash
    let recent_blockhash = client.get_latest_blockhash().unwrap();

    // Create the transaction
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&keypair.pubkey()));
    transaction.sign(&[keypair], recent_blockhash);

    // Send the transaction
    let transaction_result = client.send_and_confirm_transaction(&transaction);
 
    transaction_result
}

pub fn process_transaction_result(transaction_name: &str, transaction_result: Result<Signature, solana_client::client_error::ClientError>) {
    match transaction_result {
        Ok(signature) => {
            println!("{} transaction successful: https://explorer.solana.com/tx/{:?}?cluster=devnet\n", transaction_name, signature);
        }
        Err(err) => {
            eprintln!("Transaction failed: {:?}", err);
        }
    }
}

pub fn send_sol_with_memo(keypair: &Keypair, wallet: &Pubkey, amount: u64, memo: &str) -> Result<Signature, solana_client::client_error::ClientError> {
    let rpc_url = "https://api.devnet.solana.com"; // Use the appropriate cluster URL
    let client = RpcClient::new(rpc_url);

    // Create the transfer instruction
    let instruction = system_instruction::transfer(&keypair.pubkey(), wallet, amount);

    // Create memo instruction
    let memo_program: Pubkey = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap();

    // Get the recent blockhash
    let recent_blockhash = client.get_latest_blockhash().unwrap();

    let memo_instruction = Instruction {
        program_id: memo_program,
        accounts: vec![
            AccountMeta::new(keypair.pubkey(), true),
        ],
        data: memo.to_string().as_bytes().to_vec(),
    };

    // Create the transaction
    let mut transaction = Transaction::new_with_payer(&[instruction, memo_instruction], Some(&keypair.pubkey()));
    transaction.sign(&[keypair], recent_blockhash);

    // Send the transaction
    let transaction_result = client.send_and_confirm_transaction(&transaction);
 
    transaction_result
}

pub fn create_mint(payer_keypair: &Keypair, mint_keypair: &Keypair) -> Result<Signature, solana_client::client_error::ClientError> {
    let client = RpcClient::new(RPC_URL);

    let mint_pubkey = mint_keypair.pubkey();

    // Create the mint account
    let lamports = client.get_minimum_balance_for_rent_exemption(Mint::LEN).unwrap();
    let create_account_ix = system_instruction::create_account(
        &payer_keypair.pubkey(),
        &mint_pubkey,
        lamports,
        Mint::LEN as u64,
        &spl_token::id(),
    );

    // Initialize mint
    let initialize_mint_ix = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &payer_keypair.pubkey(),
        None,
        2,
    ).unwrap();

    // Create transaction
    let mut transaction = Transaction::new_with_payer(
        &[create_account_ix, initialize_mint_ix],
        Some(&payer_keypair.pubkey()),
    );

    // Sign transaction
    transaction.sign(&[&payer_keypair, &mint_keypair], client.get_latest_blockhash().unwrap());

    // Send transaction
    client.send_and_confirm_transaction(&transaction)
    
    // match client.send_and_confirm_transaction(&transaction) {
    //     Ok(signature) => {
    //         println!("✅ Token Mint created: {}, {}\n\n", mint_pubkey, signature);
    //     }
    //     Err(err) => {
    //         eprintln!("Error creating mint: {:?}", err);
    //     }
    // }

    // mint_keypair
}

pub fn get_associated_account_address(wallet: &Pubkey, mint_pubkey: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(
        wallet,
        mint_pubkey,
        &spl_token::id()
    )
}

pub fn create_associated_account(payer_keypair: &Keypair, wallet: &Pubkey, mint_keypair: &Keypair) -> Result<Signature, solana_client::client_error::ClientError> {
    let client = RpcClient::new(RPC_URL);
    let mint_pubkey = &mint_keypair.pubkey();

    // Create the associated token account
    let create_associated_token_account_ix = create_associated_token_account(
        &payer_keypair.pubkey(),
        wallet,
        mint_pubkey,
        &spl_token::id(),
    );

    // Create transaction
    let mut transaction = Transaction::new_with_payer(
        &[create_associated_token_account_ix],
        Some(&payer_keypair.pubkey()),
    );

    // Sign transaction
    transaction.sign(&[payer_keypair], client.get_latest_blockhash().unwrap());

    // Send transaction
    client.send_and_confirm_transaction(&transaction)
}


pub fn mint_token(
    payer_keypair: &Keypair
    , mint_keypair: &Keypair
    , associated_account_address: &Pubkey
    , mint_amount: u64
) -> Result<Signature, solana_client::client_error::ClientError>
{
    let client = RpcClient::new(RPC_URL);
    
    let mint_to_ix = token_instruction::mint_to(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        associated_account_address,
        &payer_keypair.pubkey(),
        &[],
        mint_amount,
    ).unwrap();

    // Create the transaction
    let mut transaction = Transaction::new_with_payer(
        &[mint_to_ix],
        Some(&payer_keypair.pubkey()),
    );

    // Sign the transaction
    transaction.sign(&[&payer_keypair], client.get_latest_blockhash().unwrap());

    // Send the transaction
    client.send_and_confirm_transaction(&transaction)
}


pub fn create_token_metadata_account(
    payer_keypair: &Keypair
    , mint_pubkey: &Pubkey
    , token_name: &str
    , token_symbol: &str
    , metadata_uri: &str
) -> Result<Signature, solana_client::client_error::ClientError>
{
    let client = RpcClient::new(RPC_URL);

    let (metadata_account, _) = Pubkey::find_program_address(
        &[
            b"metadata".as_ref(),
            &mpl_token_metadata::ID.to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &mpl_token_metadata::ID,
    );

    let args = mpl_token_metadata::instructions:: CreateV1InstructionArgs {
        name: String::from(token_name),
        symbol: String::from(token_symbol),
        uri: String::from(metadata_uri),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
        primary_sale_happened: false,
        is_mutable: true,
        token_standard: TokenStandard::Fungible,
        collection_details: None,
        rule_set: None, 
        decimals: Some(2),
        print_supply: Some(PrintSupply::Zero)
    };

    let create_ix = mpl_token_metadata::instructions::CreateV1 {
      metadata: metadata_account,
      master_edition: None,
      mint: (*mint_pubkey, false),
      authority: payer_keypair.pubkey(),
      payer: payer_keypair.pubkey(),
      update_authority: (payer_keypair.pubkey(), true),
      system_program: system_program::id(),
      sysvar_instructions: solana_program::sysvar::instructions::id(),
      spl_token_program: Some(spl_token::id()),  
    };

    let create_ix = create_ix.instruction(args);
    

    // Get the recent blockhash
    let recent_blockhash = client.get_latest_blockhash().unwrap();

    // Create the transaction
    
    let mut transaction = Transaction::new_with_payer(&[create_ix], Some(&payer_keypair.pubkey()));
    transaction.sign(&[&payer_keypair], recent_blockhash);

    // Send the transaction
    client.send_and_confirm_transaction(&transaction)

    // match client.send_and_confirm_transaction(&transaction) {
    //     Ok(signature) => {
    //         println!("Metadata account created successfully with signature: {:?}", signature);
    //     }
    //     Err(err) => {
    //         eprintln!("Failed to create metadata account: {:?}", err);
    //     }
    // }
}
