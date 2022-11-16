use crate::app_config::ExchangeConfig;
use crate::exchange::{Exchange, OrderBookUpdate};
use crate::orderbook::Level;
use crate::result::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::str::FromStr;

pub(crate) const EXCHANGE_KEY: &str = "bitstamp";

#[derive(Deserialize, Debug)]
pub struct BitstampUpdate {
    data: Data,
}
#[derive(Deserialize, Debug)]
pub struct Data {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}

impl BitstampUpdate {
    fn to_orderbook_update(&self) -> Result<OrderBookUpdate> {
        let mut bids = vec![];
        let mut asks = vec![];

        for (price, amount) in &self.data.bids {
            bids.push(Level {
                exchange: String::from(EXCHANGE_KEY),
                price: f64::from_str(&*price)?,
                amount: f64::from_str(&*amount)?,
            });
        }

        // FIXME codesmell: duplication
        for (price, amount) in &self.data.asks {
            asks.push(Level {
                exchange: String::from(EXCHANGE_KEY),
                price: f64::from_str(&*price)?,
                amount: f64::from_str(&*amount)?,
            });
        }

        Ok(OrderBookUpdate {
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

    fn empty_order_book_data(&self) -> OrderBookUpdate {
        OrderBookUpdate {
            exchange: self.exchange_config.id.to_string(),
            bids: vec![],
            asks: vec![],
        }
    }

    fn subscribe_msg(&self) -> String {
        let pair = self.exchange_config.spot_pair.to_lowercase();
        let msg = self
            .exchange_config
            .subscription_message_template
            .replace("{{pair}}", &pair);
        info!("sub message {}", msg.clone());
        msg
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
