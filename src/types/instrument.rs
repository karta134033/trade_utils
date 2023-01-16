use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InstrumentInfo {
    pub symbol: String,
    pub tick_size: f64,
    pub lot_size: f64,
    pub min_qty: f64,
}
