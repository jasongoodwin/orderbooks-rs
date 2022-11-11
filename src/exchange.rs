use crate::app_config::{AppConfig, ExchangeConfig};
use std::io::Error;
use std::result;
use tokio::sync::mpsc;

pub struct OrderBookData {
    timestamp_ms: u64,
    bids: Vec<f32>,
    asks: Vec<f32>,
}

fn spawn_new(
    exchange_config: ExchangeConfig,
    subscribers_rx: mpsc::Receiver<OrderBookData>,
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
            println!("running")
        }
        // Process each socket concurrently.
    });

    Ok(())
}

pub trait Exchange {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> OrderBookData;
}

pub struct Binance {}
impl Exchange for Binance {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> OrderBookData {
        todo!()
    }
}

pub struct Bitstamp {}
impl Exchange for Bitstamp {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> OrderBookData {
        todo!()
    }
}
