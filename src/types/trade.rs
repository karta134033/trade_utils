use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Sell,
    Buy,
    None,
}

impl TradeSide {
    pub fn value(&self) -> f64 {
        match *self {
            TradeSide::Sell => -1.,
            TradeSide::Buy => 1.,
            TradeSide::None => 0.,
        }
    }
}
impl Default for TradeSide {
    fn default() -> Self {
        TradeSide::None
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Trade {
    pub symbol: String,
    pub entry_price: f64,
    pub entry_side: TradeSide,
    pub entry_ts: i64,
    pub exit_price: f64,
    pub position: f64,
    pub tp_price: f64, // take profit
    pub sl_price: f64, // stop loss
}
