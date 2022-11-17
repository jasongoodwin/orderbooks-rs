# orderbooks-rs
Streams order book information from multiple exchanges to get bids/asks across exchanges and provides the best bids/asks and spread across markets.

MIT licenced so feel free to use. This document contains some notes on possible uses, lookouts, and has some extra info in the appendix you may want to look at.

You'll need rust/cargo installed.
This version was built and tested w/ rust 1.56.
The Cargo.lock file is included - it should just work.
A Makefile is provided which has common targets.

# Setup
Not much setup is needed to run, however you can check the Settings.toml file.

## Building Release
`make build` will build a runnable release into `target/release`.
A `Settings.toml` file is included.

## Collecting Metrics
Metrics are exposed for a prometheus scraper at `0.0.0.0:9000`

## Running (server)
`make server` or `RUST_LOG=info cargo run --bin server` will start the application.

## Running (client)
grpc clients can connect to server at `0.0.0.0:10000`

There is a client included so you can see the updates for test/validation.
`make client` or `RUST_LOG=info cargo run --bin client` will start a client to demonstrate the stream works.
It will print all updates to the console.

## Testing
`make test` or `cargo test` will execute the test suite.

## Log Level
You can adjust the log level with the RUST_LOG env variable. eg for debug:
`RUST_LOG="info,orderbooks-rs=debug"`

# Design Notes
Design document has information on the decisions made, gotchas, release notes etc.
See: https://docs.google.com/document/d/1psDVXU6FtZIRYa8W-z8RewljaKUG55_zFdTU5fNDGGQ/edit?usp=sharing

# Ideas on Algo Trading w/ Order Book Data
I've been working on a machine-learning DRL agent somewhat akin to AlphaGo for extracting profit from markets.
It's been an arduous journey, and I do now how a profitable model out of sample, but I'm starting to consider how to improve the approach as over-fit remains a major barrier. I have numerous ideas, however I believe that it may simply be easier to move towards HFT approaches rather than kline data alone.

In order to train a DRL agent to snipe orders via HFT or even on larger timeframes, I need to be able to make faster decisions and ensure orders are filled appropriately. I think that training an agent on multiple order books and just streaming data into it may actually make for some interesting opportunities using machine learning to observe how it learns and makes decisions regarding price action in market depth info.

## Notes on Predatory Strategies
If you're attempting to build on this project to produce a trading bot (HFT or similar) then you need to be aware of predatory strategies that will attempt to beat you in the market. 
The major look-out if you're using order book data are predatory strategies that target especially HFT traders. I'll describe a couple quickly here that specifically attempt to spoof the order book by placing and cancelling orders, but check the appendix for papers that cover the topic in depth.

**Order Fading**, for example, is a strategy where a predatory agent will submit and partially cancel orders in a short period of time to attempt to front run movement. This can dry up liquidity and cause the price to move in the span of the attack.

**Layering** is a similar strategy, with the difference being that the agent doesn't actually intend to fill orders, but only to incite other participants to detect and start to trade the price in a favorable direction.  

# Contributing
CI is pending - it's expected you format, test and lint any contributions.
Please ensure there are no clippy `errors` or compiler warnings.
Feel free to provide a PR.

## Linting
`make lint` or `cargo clippy` will lint the application.

## Formatting
`make fmt` or `cargo fmt` will format the code.

# Appendix/References (mostly ML/HFT related items of interest)
"101 Formulaic Alphas." Contains 101 confirmed formulaic alpha factors that can be derived from OCHLV/kline data.
https://www.researchgate.net/publication/289587760_101_Formulaic_Alphas

"An Empirical detection of HFT strategies."
http://www.smallake.kr/wp-content/uploads/2016/03/448.pdf

“High frequency trading–measurement, detection and response.” Credit Suisse
https://docplayer.net/77225061-High-frequency-trading-measurement-detection-and-response-market-commentary-6-december-2012.html

Repository from the book "Machine Learning for Trading."
The book is also worth reading. Check the alpha factor library if you're considering trading price data (eg Prediction or DRL.)
https://github.com/stefan-jansen/machine-learning-for-trading

