#[derive(Default, Debug)]
pub struct Account {
    pub assets: Vec<Asset>,
    pub positions: Vec<Position>,
}

#[derive(Default, Debug)]
pub struct Position {
    pub symbol: String,
    pub unrealized_profit: f64,
    pub leverage: u64,
    pub entry_price: f64,
    pub position_side: String,
    pub position_amt: f64,
}

#[derive(Default, Debug)]
pub struct Asset {
    pub asset: String,
    pub wallet_balance: f64,
    pub available_balance: f64,
    pub update_timestamp: i64,
}
