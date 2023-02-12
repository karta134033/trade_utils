use std::fmt;

pub enum OrderType {
    Market,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderType::Market => write!(f, "MARKET"),
        }
    }
}

#[derive(PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

pub enum TimeInForce {
    Gtc,
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimeInForce::Gtc => write!(f, "GTC"),
        }
    }
}

pub struct Order {
    pub symbol: String,
    pub size: f64, // quantity in binance
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: Option<TimeInForce>,
    pub price: Option<f64>,
    pub reduce_only: bool,
}

impl Order {
    pub fn market_order(symbol: String, order_side: OrderSide, size: f64) -> Order {
        Order {
            symbol,
            order_side,
            size,
            order_type: OrderType::Market,
            time_in_force: None,
            price: None,
            reduce_only: false,
        }
    }
}
