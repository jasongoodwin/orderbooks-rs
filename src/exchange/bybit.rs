// TODO - Was looking at other exchanges to validate the design. bybit works a bit differently - it sends a snapshot and then updates to it.
// This would work, but the Bybit specific details will need to hold the snapshot (eg on validation of the subscription response) and then modify internal state with the diffs.

// //! Bybit specific details for subscribing to and parsing orderbook data.
// use std::io::Read;
// use std::str::FromStr;
//
// use async_trait::async_trait;
// use tokio::time::Instant;
//
// use serde::Deserialize;
//
// use crate::app_config::ExchangeConfig;
// use crate::exchange::{Exchange, OrderBookUpdate};
// use crate::orderbook::Level;
// use crate::result::Result;
//
// pub(crate) const EXCHANGE_KEY: &str = "bybit";
//
//
// impl BybitUpdate {
// }
//
// pub struct Bybit {
//     pub(crate) exchange_config: ExchangeConfig,
// }
//
// #[async_trait]
// impl Exchange for Bybit {
//
//     fn exchange_config(&self) -> &ExchangeConfig {
//         &self.exchange_config
//     }
//
//     fn subscribe_msg(&self) -> String {
//         let pair = self.exchange_config().spot_pair.to_uppercase();
//         let msg = self
//             .exchange_config()
//             .subscription_message_template
//             .replace("{{pair}}", &pair);
//         info!("sub message {}", msg.clone());
//         msg
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
// }
