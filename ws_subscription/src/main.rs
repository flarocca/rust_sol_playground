mod raydium;

use raydium::execute_demo;

// https://henrytirla.medium.com/how-to-fetch-newly-created-pairs-pools-on-solana-raydium-dex-5baeed3ce8a3

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://api.mainnet-beta.solana.com";
    let rpc_url = "https://api.mainnet-beta.solana.com";

    execute_demo(ws_url, rpc_url).await
}
