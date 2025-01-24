mod api;
mod commands;
mod raydium;

//use raydium::execute_demo;

//use raydium::{
//    execute_demo, test_pool_created, test_swap_detected, test_swap_exact_input, test_swap_via_api,
//};

// https://henrytirla.medium.com/how-to-fetch-newly-created-pairs-pools-on-solana-raydium-dex-5baeed3ce8a3

//#[tokio::main]
//async fn main() -> anyhow::Result<()> {
//    let ws_url = "wss://api.mainnet-beta.solana.com";
//    let rpc_url = "https://api.mainnet-beta.solana.com";
//    //let keypair_file_path = "/Users/flr/.config/solana/wallet.json";
//
//    execute_demo(ws_url, rpc_url).await
//    //test_pool_created(rpc_url).await
//    //test_swap_detected(rpc_url).await
//    //test_swap_exact_input(rpc_url, keypair_file_path).await
//    //test_swap_via_api(rpc_url).await
//}

#[tokio::main]
async fn main() {
    let commands = commands::get_commands();

    let mut clap_commands = clap::Command::new("solana-raydium-bot")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true);

    for command in commands.values() {
        clap_commands = clap_commands.subcommand(command.create());
    }

    let matches = clap_commands.get_matches();
    match matches.subcommand() {
        Some(subcommand) => {
            let (subcommand_name, subcommand_args) = subcommand;
            let command = commands.get(subcommand_name).unwrap();
            command
                .execute(subcommand_args)
                .await
                .expect("Failed to execute command");
        }
        _ => {
            println!("No subcommand provided");
        }
    }
}
