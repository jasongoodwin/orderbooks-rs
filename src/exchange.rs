use crate::app_config::{AppConfig, ExchangeConfig};
use crate::orderbook::Level;
use std::io::Error;
use std::result;
use tokio::sync::mpsc;

#[derive(Debug, Default, PartialEq)]
pub struct OrderBookUpdate {
    pub(crate) bids: Vec<Level>,
    pub(crate) asks: Vec<Level>,
}

fn spawn_new(
    exchange_config: ExchangeConfig,
    subscribers_tx: mpsc::Sender<OrderBookUpdate>,
) -> crate::result::Result<()> {
    let exchange: Box<dyn Exchange> = match exchange_config.id.as_str() {
        "binance" => Box::new(Binance {}),
        "bitstamp" => Box::new(Bitstamp {}),
        id => Err(format!("unsupported exchange configured: {}", id))?,
    };

    tokio::spawn(async move {
        info!(
            "starting exchange order book collection for: [{:?}]",
            exchange_config
        );
        loop {
            subscribers_tx
                .send(OrderBookUpdate::default())
                .await
                .expect("failed to send order book data! this will crash the thread...");
        }
    });

    Ok(())
}

pub trait Exchange {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> OrderBookUpdate;
}

pub struct Binance {}
impl Exchange for Binance {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> OrderBookUpdate {
        todo!()
    }
}

pub struct Bitstamp {}
impl Exchange for Bitstamp {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> OrderBookUpdate {
        todo!()
    }
}
