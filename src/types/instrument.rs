use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InstrumentInfo {
    pub symbol: String,
    pub tick_size: String,
    pub lot_size: String,
    pub min_qty: f64,
}
