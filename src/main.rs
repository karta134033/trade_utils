use clap::Parser;
use trade_utils::types::cli::Cli;

fn main() {
    let args = Cli::parse();
    println!("args: {:?}", args);
}
