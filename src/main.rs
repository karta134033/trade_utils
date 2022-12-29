use async_std::task;
use chrono::NaiveDateTime;
use trade_utils::clients::mongo_client::MongoClient;

pub const LOCAL_MONGO_CONNECTION_STRING: &str = "mongodb://localhost:27017";
pub const BTCUSDT_15M: &str = "BTCUSDT_15m";
pub const KLINE_DB: &str = "klines";

fn main() {
    let from_datetime =
        NaiveDateTime::parse_from_str("2022-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let to_datetime =
        NaiveDateTime::parse_from_str("2022-12-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let from_ts_ms = from_datetime.timestamp_millis();
    let to_ts_ms = to_datetime.timestamp_millis();

    let mongo_clinet = task::block_on(MongoClient::new(LOCAL_MONGO_CONNECTION_STRING));
    let klines =
        task::block_on(mongo_clinet.get_klines(KLINE_DB, BTCUSDT_15M, from_ts_ms, Some(to_ts_ms)));
    println!("klines: {:?}", klines);
}
