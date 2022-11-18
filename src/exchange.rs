use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};
use tokio_tungstenite::tungstenite::Message::Pong;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::app_config::ExchangeConfig;
use crate::exchange::binance::Binance;
use crate::exchange::bitstamp::Bitstamp;
use crate::orderbook::Level;
use crate::result::Result;

mod binance;
mod bitstamp;

#[derive(Debug, PartialEq)]
pub struct OrderBookUpdate {
    // Timestamp of OrderBookUpdate creation for metrics. This is a little late but captures our code.
    pub(crate) ts: Instant,
    pub(crate) exchange: String,
    // Note: we use the Level struct which will duplicate the exchange in each level.
    // Could be optimized by using another struct but it's simpler like this for now.
    pub(crate) bids: Vec<Level>,
    pub(crate) asks: Vec<Level>,
}

#[async_trait]
/// trait representing the specific implementation details needed for a specific exchange
trait Exchange {
    fn parse_order_book_data(&self, bytes: Vec<u8>) -> Result<OrderBookUpdate>;
    fn empty_order_book_data(&self) -> OrderBookUpdate;
    fn subscribe_msg(&self) -> String;
}

// TODOs:
// 1. validate subscription reply
// 2. refactor to test message receipt/reply.
// 3. If we don't get a message in x period of time, should probably close connection and re-establish.

/// produces a thread to establish and manages connection/subscription to an exchange.
pub async fn create_exchange_ws_connection(
    exchange_config: ExchangeConfig,
    subscribers_tx: mpsc::Sender<OrderBookUpdate>,
) -> Result<()> {
    // TODO no result type needed here.

    tokio::spawn(async move {
        // there are two nested loops here. If an error is encountered in the inner loop,
        // we can drop the connection, break out of it, and let the connection be re-established.
        // This is similar to the actor model in eg erlang/akka where we assume state can become corrupt,
        // and we just throw away the state on error and recreate it instead of writing any defensive code.
        // see the "LET IT CRASH" design philosophy https://wiki.c2.com/?LetItCrash

        // Secondary we need to assume that we don't have order book information for the exchange
        // anymore if failures are encountered.
        // Eg if the exchange endpoint goes down, we want to signal that there are no bids/asks for
        // the exchange available until we re-establish stability.
        loop {
            // outer loop will establish connection
            info!(
                "starting exchange ws order book collection for: [{:?}]",
                exchange_config.clone()
            );
            let exchange: Box<dyn Exchange + Sync + Send> = match exchange_config.id.as_str() {
                binance::EXCHANGE_KEY => Box::new(Binance {
                    exchange_config: exchange_config.clone(),
                }),
                bitstamp::EXCHANGE_KEY => Box::new(Bitstamp {
                    exchange_config: exchange_config.clone(),
                }),
                id => Err(format!("unsupported exchange configured: {}", id)).unwrap(), // will crash the app.
            };

            let (mut ws_stream, _) = match connect_async(exchange_config.endpoint.clone()).await {
                Ok(x) => x,
                Err(_) => {
                    error!(
                        "issue connecting to {}... Will retry.",
                        exchange_config.id.as_str()
                    );
                    sleep(Duration::from_millis(100)).await; // wait 100ms to avoid hammering a failing endpoint.
                    continue;
                }
            };

            info!(
                "WebSocket handshake has been successfully completed for {}",
                exchange_config.id.as_str()
            );

            // subscribe
            if !ws_stream
                .send(Message::text(exchange.subscribe_msg()))
                .await
                .is_ok()
            {
                error!(
                    "issue sending subscribe for {}... Will retry.",
                    exchange_config.id.as_str()
                );
                sleep(Duration::from_millis(100)).await; // wait 100ms to avoid hammering a failing endpoint.
                continue;
            }

            // Get the reply message. If anything not as expected, we just continue the loop w/ a delay.
            match ws_stream.next().await {
                None => {
                    error!("nothing after subscription!! This isn't expected! Will restart connection.");
                    sleep(Duration::from_millis(100)).await; // wait 100ms to avoid hammering a failing endpoint.
                    continue;
                }
                Some(Ok(msg)) => {
                    // FIXME need validate the subscription reply is as expected. implement in Exchange trait.
                    info!("got subscription reply {:?}", msg.to_string())
                }
                Some(Err(e)) => {
                    error!(
                        "Need to retry connection... got an error trying to subscribe to {}... {:?}",
                        exchange_config.id.as_str(),
                        e
                    );
                    sleep(Duration::from_millis(100)).await; // wait 100ms to avoid hammering a failing endpoint.
                    continue;
                }
            }

            loop {
                // inner loop will process any input received.
                match ws_stream.next().await {
                    // We explicitly handle ping frames and reply w/ a pong frame (binance will disconnect after 10m if not handled)
                    Some(Ok(msg)) if msg.is_ping() => {
                        info!(
                            "ping frame received for {}. Sending pong",
                            exchange_config.id.as_str()
                        );
                        if ws_stream.send(Pong(vec![])).await.is_err() {
                            error!(
                                "error sending ping reply to {}... Will retry disconnect/retry.",
                                exchange_config.id.as_str()
                            );
                            break;
                        }
                    }
                    Some(Ok(msg)) => {
                        match exchange.parse_order_book_data(msg.into_data()) {
                            Ok(order_book_update) => {
                                // can possibly spawn this instead of awaiting, but need to ensure order.
                                subscribers_tx
                                    .send(order_book_update)
                                    .await
                                    .expect("unexpected error sending to channel. Panic!");
                            }
                            Err(e) => {
                                subscribers_tx
                                    .send(exchange.empty_order_book_data())
                                    .await
                                    .expect("unexpected error sending to channel. Panic!");
                                error!("Restarting connection as we couldn't parse an orderbook update for {}!: {:?}", exchange_config.id.as_str(), e);
                                break;
                            }
                        }
                    }
                    None => info!("No message received off of ws..."),
                    Some(Err(e)) => {
                        error!(
                            "Error encountered on ws. will restart connection... {:?}",
                            e
                        );
                        break;
                    }
                }
            }

            subscribers_tx
                .send(exchange.empty_order_book_data())
                .await
                .expect("unexpected error sending to channel");
            sleep(Duration::from_millis(100)).await; // wait 100ms to avoid hammering a failing endpoint.
        }
    });

    Ok(())
}
