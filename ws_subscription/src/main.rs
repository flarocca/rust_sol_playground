mod raydium;

use raydium::{
    execute_demo, test_pool_created, test_swap_detected, test_swap_exact_input, test_swap_via_api,
};

// https://henrytirla.medium.com/how-to-fetch-newly-created-pairs-pools-on-solana-raydium-dex-5baeed3ce8a3

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://api.mainnet-beta.solana.com";
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let keypair_file_path = "/home/robert/.config/solana/id.json";

    //execute_demo(ws_url, rpc_url).await
    //test_pool_created(rpc_url).await
    //test_swap_detected(rpc_url).await
    test_swap_exact_input(rpc_url, keypair_file_path).await
    //test_swap_via_api(rpc_url).await
}
