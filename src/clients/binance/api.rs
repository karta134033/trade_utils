use anyhow::Result;
use chrono::Utc;
use hmac::Hmac;
use hmac::Mac;
use log::info;
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::clients::binance::parser::parse_api_kline;
use crate::types::account::Account;
use crate::types::account::Asset;
use crate::types::account::Position;
use crate::types::instrument::InstrumentInfo;
use crate::types::kline::Kline;
use crate::types::order::Order;

pub const FUTURES_KLINE: &str = "/fapi/v1/klines";
pub const FUTURES_ACCOUNT: &str = "/fapi/v2/account";
pub const FUTURES_EXCHANGE_INFO: &str = "/fapi/v1/exchangeInfo";
pub const FUTURES_ORDER: &str = "/fapi/v1/order";
pub const FUTURES_BASE: &str = "https://fapi.binance.com";

pub struct BinanceFuturesApiClient {
    client: reqwest::Client,
    api_key: String,
    secret_key: String,
}

impl BinanceFuturesApiClient {
    pub fn new(api_key: String, secret_key: String) -> BinanceFuturesApiClient {
        BinanceFuturesApiClient {
            client: reqwest::Client::new(),
            api_key,
            secret_key,
        }
    }

    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str, // "1d, 1h, 1m"
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: Option<&str>,
    ) -> Result<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol", symbol);
        params.insert("interval", interval);
        if start_time.is_some() {
            params.insert("startTime", start_time.unwrap());
        }
        if end_time.is_some() {
            params.insert("endTime", end_time.unwrap());
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
        let mut klines = values
            .into_iter()
            .map(|value| parse_api_kline(value).unwrap())
            .collect::<Vec<_>>();
        klines.sort_by_key(|k| k.close_timestamp);
        Ok(klines)
    }

    pub fn hash_signature(&self, params: &mut Vec<(String, String)>, secret_key: &str) {
        let mut request_string = "".to_owned();
        for (k, v) in params.iter() {
            request_string += &format!("{}={}&", k, v);
        }
        let timestamp = Utc::now().timestamp_millis();
        request_string += &format!("timestamp={}", timestamp);

        let mut signed_key = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).unwrap();
        signed_key.update(request_string.as_bytes());

        let signature = hex::encode(signed_key.finalize().into_bytes());
        params.push(("timestamp".to_owned(), timestamp.to_string()));
        params.push(("signature".to_owned(), signature));
    }

    pub async fn get_account(&self) -> Result<Account> {
        let mut params = Vec::new();
        self.hash_signature(&mut params, &self.secret_key);
        let endpoint = format!("{}{}", FUTURES_BASE, FUTURES_ACCOUNT);
        let request_url = reqwest::Url::parse_with_params(endpoint.as_str(), &params).unwrap();
        let response = self
            .client
            .get(request_url)
            .header("X-MBX-APIKEY", &self.api_key)
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

    pub async fn get_instruments(
        &self,
        symbol_set: &HashSet<String>,
    ) -> Result<HashMap<String, InstrumentInfo>> {
        let endpoint = format!("{}{}", FUTURES_BASE, FUTURES_EXCHANGE_INFO);
        let request_url = reqwest::Url::parse(endpoint.as_str()).unwrap();
        let response = self.client.get(request_url).send().await?;
        let content = response.text().await?;
        let value: Value = serde_json::from_str(content.as_str())?;
        let symbols = value["symbols"].as_array().unwrap();
        let mut symbol_to_instrument_info = HashMap::new();
        for s in symbols {
            let symbol = s["symbol"].as_str().unwrap();
            if s["status"].as_str().unwrap() == "TRADING"
                && s["contractType"].as_str().unwrap() == "PERPETUAL"
                && symbol_set.contains(symbol)
            {
                let mut tick_size = "".to_owned();
                let mut lot_size = "".to_owned();
                let mut min_qty = 0.;
                let filters = s["filters"].as_array().unwrap();
                for f in filters {
                    match f["filterType"].as_str().unwrap() {
                        "PRICE_FILTER" => {
                            tick_size = f["tickSize"].as_str().unwrap().to_owned();
                        }
                        "LOT_SIZE" => {
                            lot_size = f["stepSize"].as_str().unwrap().to_owned();
                            min_qty = f["minQty"].as_str().unwrap().parse::<f64>()?;
                        }
                        _ => {}
                    }
                }
                symbol_to_instrument_info.insert(
                    symbol.to_owned(),
                    InstrumentInfo {
                        symbol: symbol.to_owned(),
                        tick_size,
                        lot_size,
                        min_qty,
                    },
                );
            }
        }
        Ok(symbol_to_instrument_info)
    }

    pub async fn place_order(
        &self,
        order: Order,
        instrument_info: &InstrumentInfo,
    ) -> Result<Value> {
        let mut params = Vec::new();
        params.push(("symbol".to_owned(), order.symbol));
        params.push(("side".to_owned(), order.order_side.to_string()));
        params.push(("type".to_owned(), order.order_type.to_string()));
        params.push(("reduceOnly".to_owned(), order.reduce_only.to_string()));

        let lot_precision =
            instrument_info.lot_size.len() - 1 - instrument_info.lot_size.find('.').unwrap_or(0);
        let quantity_string = format!("{:.*}", lot_precision, order.size);
        params.push(("quantity".to_owned(), quantity_string));

        if order.time_in_force.is_some() {
            params.push((
                "timeInForce".to_owned(),
                order.time_in_force.unwrap().to_string(),
            ));
        }
        if order.price.is_some() {
            let tick_precision = instrument_info.tick_size.len()
                - 1
                - instrument_info.tick_size.find('.').unwrap_or(0);
            let price_string = format!("{:.*}", tick_precision, order.price.unwrap());
            params.push(("price".to_owned(), price_string));
        }
        info!("Order params: {:?}", params);
        self.hash_signature(&mut params, &self.secret_key);
        let endpoint = format!("{}{}", FUTURES_BASE, FUTURES_ORDER);
        let request_url = reqwest::Url::parse_with_params(endpoint.as_str(), &params).unwrap();
        let response = self
            .client
            .post(request_url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;
        let content = response.text().await?;
        let value: Value = serde_json::from_str(content.as_str())?;
        Ok(value)
    }
}
