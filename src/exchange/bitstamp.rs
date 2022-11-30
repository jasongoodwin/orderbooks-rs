//! Bitstamp specific details for subscribing to and parsing orderbook data.
use std::str::FromStr;

use async_trait::async_trait;
use tokio::time::Instant;

use serde::Deserialize;

use crate::app_config::ExchangeConfig;
use crate::exchange::{Exchange, OrderBookUpdate};
use crate::orderbook::Level;
use crate::result::Result;
pub(crate) const EXCHANGE_KEY: &str = "bitstamp";

#[derive(Deserialize, Debug)]
// structure for json deserialization
struct BitstampUpdate {
    data: Data,
}
#[derive(Deserialize, Debug)]
struct Data {
    // timestamp: String,
    // microtimestamp: String,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}

impl BitstampUpdate {
    fn to_orderbook_update(&self) -> Result<OrderBookUpdate> {
        let ts = Instant::now();
        let mut bids = vec![];
        let mut asks = vec![];

        for (price, amount) in &self.data.bids {
            bids.push(Level {
                exchange: String::from(EXCHANGE_KEY),
                price: f64::from_str(&*price)?,
                amount: f64::from_str(&*amount)?,
            });
        }

        for (price, amount) in &self.data.asks {
            asks.push(Level {
                exchange: String::from(EXCHANGE_KEY),
                price: f64::from_str(&*price)?,
                amount: f64::from_str(&*amount)?,
            });
        }

        Ok(OrderBookUpdate {
            ts,
            exchange: String::from(EXCHANGE_KEY),
            bids,
            asks,
        })
    }
}

pub struct Bitstamp {
    pub(crate) exchange_config: ExchangeConfig,
}

#[async_trait]
impl Exchange for Bitstamp {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> Result<OrderBookUpdate> {
        let parsed: BitstampUpdate = serde_json::from_slice(&bytes).unwrap();
        parsed.to_orderbook_update()
    }

    fn exchange_config(&self) -> &ExchangeConfig {
        &self.exchange_config
    }

    fn empty_order_book_data(&self) -> OrderBookUpdate {
        OrderBookUpdate {
            ts: Instant::now(),
            exchange: self.exchange_config.id.to_string(),
            bids: vec![],
            asks: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_orderbook_data() {
        let msg = r#"
            {
              "data": {
                "timestamp": "1668182726",
                "microtimestamp": "1668182726938075",
                "bids": [
                  [
                    "16880",
                    "0.29673802"
                  ],
                  [
                    "16878",
                    "0.16020064"
                  ]
                ],
                "asks": [
                  [
                    "16879",
                    "0.29673801"
                  ],
                  [
                    "16877",
                    "0.16020063"
                  ]
                ]
              }
            }
        "#;

        let parsed: BitstampUpdate = serde_json::from_str(msg).unwrap();

        let orderbook_update = parsed.to_orderbook_update().unwrap();

        assert_eq!(
            orderbook_update,
            OrderBookUpdate {
                ts: orderbook_update.ts,
                exchange: "bitstamp".to_string(),
                bids: vec![
                    Level {
                        exchange: "bitstamp".to_string(),
                        price: 16880.0,
                        amount: 0.29673802
                    },
                    Level {
                        exchange: "bitstamp".to_string(),
                        price: 16878.0,
                        amount: 0.16020064
                    }
                ],
                asks: vec![
                    Level {
                        exchange: "bitstamp".to_string(),
                        price: 16879.0,
                        amount: 0.29673801
                    },
                    Level {
                        exchange: "bitstamp".to_string(),
                        price: 16877.0,
                        amount: 0.16020063
                    }
                ],
            }
        );
    }
}
