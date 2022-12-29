#[derive(Debug, PartialEq, Clone)]
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

#[derive(Default)]
pub struct Trade {
    entry_price: f64,
    entry_side: TradeSide,
    exit_price: f64,
    position: f64,
    tp_price: f64, // take profit
    sl_price: f64, // stop loss
}
