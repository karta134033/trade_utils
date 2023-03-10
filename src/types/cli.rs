use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
#[command(arg_required_else_help = false)]
pub struct Cli {
    #[arg(short = 'c')]
    pub config_path: PathBuf,
    #[arg(short = 'm')]
    pub mode: Mode,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Backtest,
    Hypertune,
    Live,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backtest" => Ok(Mode::Backtest),
            "hypertune" => Ok(Mode::Hypertune),
            "live" => Ok(Mode::Live),
            "b" => Ok(Mode::Backtest),
            "h" => Ok(Mode::Hypertune),
            "l" => Ok(Mode::Live),
            _ => Err(format!("Invalid mode: {}", s)),
        }
    }
}
