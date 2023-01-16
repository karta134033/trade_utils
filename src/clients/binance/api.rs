use anyhow::Result;
use chrono::Utc;
use hmac::Hmac;
use hmac::Mac;
use log::info;
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;

use crate::clients::binance::parser::parse_api_kline;
use crate::types::account::Account;
use crate::types::account::Asset;
use crate::types::account::Position;
use crate::types::instrument::InstrumentInfo;
use crate::types::kline::Kline;

pub const FUTURES_KLINE: &str = "/fapi/v1/klines";
pub const FUTURES_ACCOUNT: &str = "/fapi/v2/account";
pub const FUTURES_EXCHANGE_INFO: &str = "/fapi/v1/exchangeInfo";
pub const FUTURES_BASE: &str = "https://fapi.binance.com";

pub struct BinanceFuturesApiClient {
    client: reqwest::Client,
}

impl BinanceFuturesApiClient {
    pub fn new() -> BinanceFuturesApiClient {
        BinanceFuturesApiClient {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str, // "1d, 1h, 1m"
        start_time: &str,
        end_time: Option<&str>,
        limit: Option<&str>,
    ) -> Result<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol", symbol);
        params.insert("interval", interval);
        params.insert("startTime", start_time);
        if end_time.is_some() {
            params.insert("end_time", end_time.unwrap());
        }
        if limit.is_some() {
            params.insert("limit", limit.unwrap());
        }

        let endpoint = format!("{}{}", FUTURES_BASE, FUTURES_KLINE);
        let request_url = reqwest::Url::parse_with_params(endpoint.as_str(), &params).unwrap();
        println!("request_url: {:?}", request_url.to_string());

        let response = self.client.get(request_url).send().await?;
        let content = response.text().await?;
        let values: Vec<Value> = serde_json::from_str(content.as_str())?;
        let klines = values
            .into_iter()
            .map(|value| parse_api_kline(value).unwrap())
            .collect::<Vec<_>>();
        Ok(klines)
    }

    pub async fn get_account(&self, api_key: &str, secret_key: &str) -> Result<Account> {
        let timestamp = Utc::now().timestamp_millis();
        let mut params = Vec::new();
        params.push(("timestamp".to_owned(), timestamp.to_string()));

        let request_string = format!("timestamp={}", timestamp);
        let mut signed_key = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).unwrap();
        signed_key.update(request_string.as_bytes());

        let signature = hex::encode(signed_key.finalize().into_bytes());
        info!("signed_key: {}", signature);
        params.push(("signature".to_owned(), signature));

        let endpoint = format!("{}{}", FUTURES_BASE, FUTURES_ACCOUNT);

        let request_url = reqwest::Url::parse_with_params(endpoint.as_str(), &params).unwrap();
        let response = self
            .client
            .get(request_url)
            .header("X-MBX-APIKEY", api_key)
            .send()
            .await?;
        let content = response.text().await?;
        let value: Value = serde_json::from_str(content.as_str())?;
        let mut account = Account {
            ..Default::default()
        };
        let asset_values = value["assets"].as_array().unwrap();
        let position_values = value["positions"].as_array().unwrap();
        for v in asset_values.iter() {
            let asset = v["asset"].as_str().unwrap().to_owned();
            let wallet_balance = v["walletBalance"].as_str().unwrap().parse::<f64>()?;
            let available_balance = v["availableBalance"].as_str().unwrap().parse::<f64>()?;
            let update_timestamp = v["updateTime"].as_i64().unwrap();
            if available_balance != 0. {
                account.assets.push(Asset {
                    asset,
                    wallet_balance,
                    available_balance,
                    update_timestamp,
                });
            }
        }
        for v in position_values.iter() {
            let symbol = v["symbol"].as_str().unwrap().to_owned();
            let unrealized_profit = v["unrealizedProfit"].as_str().unwrap().parse::<f64>()?;
            let leverage = v["leverage"].as_str().unwrap().parse::<u64>()?;
            let entry_price = v["entryPrice"].as_str().unwrap().parse::<f64>()?;
            let position_side = v["positionSide"].as_str().unwrap().to_owned();
            let position_amt = v["positionAmt"].as_str().unwrap().parse::<f64>()?;
            if position_amt != 0. {
                account.positions.push(Position {
                    symbol,
                    unrealized_profit,
                    leverage,
                    entry_price,
                    position_side,
                    position_amt,
                });
            }
        }
        Ok(account)
    }

    pub async fn get_instruments(&self, symbol: String) -> Result<Option<InstrumentInfo>> {
        let endpoint = format!("{}{}", FUTURES_BASE, FUTURES_EXCHANGE_INFO);
        let request_url = reqwest::Url::parse(endpoint.as_str()).unwrap();
        let response = self.client.get(request_url).send().await?;
        let content = response.text().await?;
        let value: Value = serde_json::from_str(content.as_str())?;
        let symbols = value["symbols"].as_array().unwrap();
        for s in symbols {
            if s["status"].as_str().unwrap() == "TRADING"
                && s["contractType"].as_str().unwrap() == "PERPETUAL"
                && s["symbol"].as_str().unwrap() == symbol
            {
                let mut tick_size = 0.;
                let mut lot_size = 0.;
                let mut min_qty = 0.;
                let filters = s["filters"].as_array().unwrap();
                for f in filters {
                    match f["filterType"].as_str().unwrap() {
                        "PRICE_FILTER" => {
                            tick_size = f["tickSize"].as_str().unwrap().parse::<f64>()?;
                        }
                        "LOT_SIZE" => {
                            lot_size = f["stepSize"].as_str().unwrap().parse::<f64>()?;
                            min_qty = f["minQty"].as_str().unwrap().parse::<f64>()?;
                        }
                        _ => {}
                    }
                }
                return Ok(Some(InstrumentInfo {
                    symbol,
                    tick_size,
                    lot_size,
                    min_qty,
                }));
            }
        }
        Ok(None)
    }
}
