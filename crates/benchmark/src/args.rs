use clap::{command, Parser};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "ws://127.0.0.1:3001", help = "Hostname")]
    pub rpc_host: String,

    #[arg(
        short,
        long,
        default_value = "http://127.0.0.1:3000",
        help = "Hostname"
    )]
    pub api_host: String,

    #[arg(short, long, default_value_t = 50, help = "Parallel")]
    pub parallel: u8,

    #[arg(
        short,
        long,
        default_value_t = 10000,
        help = "Amount of clients to connect"
    )]
    pub clients: usize,

    #[arg(
        short,
        long,
        default_value_t = 5,
        help = "Simulation duration in minutes"
    )]
    pub duration: u8,

    #[arg(short, long, default_value_t = 60, help = "Request timeout")]
    pub timeout: u8,

    #[arg(short, long, default_value_t = true, help = "Authenticate WebSocket")]
    pub authenticate: bool,

    #[arg(short, long, default_value_t = 10, help = "Amount of quests to create")]
    pub quests: u8,
}
