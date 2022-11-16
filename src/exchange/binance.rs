use crate::app_config::ExchangeConfig;
use crate::exchange::{Exchange, OrderBookUpdate};
use crate::orderbook::Level;
use crate::result::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Result as JsonResult;
use std::str::FromStr;

pub(crate) const EXCHANGE_KEY: &str = "binance";

#[derive(Deserialize, Debug)]
pub struct BinanceUpdate {
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}

impl BinanceUpdate {
    fn to_orderbook_update(&self) -> Result<OrderBookUpdate> {
        let mut bids = vec![];
        let mut asks = vec![];

        for (price, amount) in &self.bids {
            bids.push(Level {
                exchange: String::from(EXCHANGE_KEY),
                price: f64::from_str(&*price)?,
                amount: f64::from_str(&*amount)?,
            });
        }

        // FIXME codesmell: duplication
        for (price, amount) in &self.asks {
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

pub struct Binance {
    pub(crate) exchange_config: ExchangeConfig,
}

#[async_trait]
impl Exchange for Binance {
    fn empty_order_book_data(&self) -> OrderBookUpdate {
        OrderBookUpdate {
            exchange: self.exchange_config.id.to_string(),
            bids: vec![],
            asks: vec![],
        }
    }
    // async fn get_order_book_data(&self) -> Result<Vec<u8>> {
    //     todo!()
    // }
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> Result<OrderBookUpdate> {
        println!("update: {}", String::from_utf8(bytes).unwrap());
        Ok(self.empty_order_book_data())
    }

    fn subscribe_msg(&self) -> String {
        let pair = self.exchange_config.spot_pair.to_uppercase();
        let msg = self
            .exchange_config
            .subscription_message_template
            .replace("{{pair}}", &pair);
        println!("sub message {}", msg.clone());
        msg
    }
}

#[cfg(test)]
mod tests {
    // use crate::app_config::AppConfig;
    use super::*;

    #[test]
    fn should_parse_orderbook_data() {
        let msg = r#"
            {
              "lastUpdateId": 27706994933,
              "bids": [
                [
                  "16542.84000000",
                  "0.08815000"
                ],
                [
                  "16542.83000000",
                  "0.06128000"
                ]
              ],
              "asks": [
                [
                  "16543.91000000",
                  "0.02619000"
                ],
                [
                  "16543.92000000",
                  "0.00067000"
                ]
              ]
            }
"#;

        let parsed: BinanceUpdate = serde_json::from_str(msg).unwrap();

        let orderbook_update = parsed.to_orderbook_update().unwrap();

        assert_eq!(
            orderbook_update,
            OrderBookUpdate {
                exchange: "binance".to_string(),
                bids: vec![
                    Level {
                        exchange: "binance".to_string(),
                        price: 16542.84000000,
                        amount: 0.08815,
                    },
                    Level {
                        exchange: "binance".to_string(),
                        price: 16542.83000000,
                        amount: 0.06128000,
                    }
                ],
                asks: vec![
                    Level {
                        exchange: "binance".to_string(),
                        price: 16543.91000000,
                        amount: 0.02619000,
                    },
                    Level {
                        exchange: "binance".to_string(),
                        price: 16543.92000000,
                        amount: 0.00067000,
                    }
                ],
            }
        );

        println!("{:?}", orderbook_update);
    }
}
