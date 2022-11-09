# orderbooks-rs
Streams order book information from multiple exchanges to get bids/asks across exchanges and provides the best bids/asks and spread across markets.

MIT licenced so feel free to use. Document contains some notes on possible uses, lookouts, and has some extra info in the appendix you may want to look at.

# Setup
Not much setup is needed. You'll need rust/cargo installed.

# Running
`make run` or `cargo run` will start the application.

# Testing
`make test` or `cargo test` will execute the test suite.

# Design Notes

# Ideas on Algo Trading w/ Order Book Data
I've been working on a machine-learning DRL agent somewhat akin to AlphaGo for extracting profit from markets.
It's been an arduous journey, and I do now how a profitable model out of sample, but I'm starting to consider how to improve the approach as over-fit remains a major barrier. I have numerous ideas, however I believe that it may simply be easier to move towards HFT approaches rather than kline data alone.

In order to train a DRL agent to snipe orders via HFT or even on larger timeframes, I need to be able to make faster decisions and ensure orders are filled appropriately. I think that training an agent on multiple order books and just streaming data into it may actually make for some interesting opportunities using machine learning to observe how it learns and makes decisions regarding price action in market depth info.

# Notes on Predatory Strategies
If you're attempting to build on this project to produce a trading bot (HFT or similar) then you need to be aware of predatory strategies that will attempt to beat you in the market. 
The major look-out if you're using order book data are predatory strategies that target especially HFT traders. I'll describe a couple quickly here that specifically attempt to spoof the order book by placing and cancelling orders, but check the appendix for papers that cover the topic in depth.

**Order Fading**, for example, is a strategy where a predatory agent will submit and partially cancel orders in a short period of time to attempt to front run movement. This can dry up liquidity and cause the price to move in the span of the attack.

**Layering** is a similar strategy, with the difference being that the agent doesn't actually intend to fill orders, but only to incite other participants to detect and start to trade the price in a favorable direction.  

# Appendix/References
"101 Formulaic Alphas." Contains 101 confirmed formulaic alpha factors that can be derived from OCHLV data.
https://www.researchgate.net/publication/289587760_101_Formulaic_Alphas

"An Empirical detection of HFT strategies."
http://www.smallake.kr/wp-content/uploads/2016/03/448.pdf

“High frequency trading–measurement, detection and response.” Credit Suisse
https://docplayer.net/77225061-High-frequency-trading-measurement-detection-and-response-market-commentary-6-december-2012.html

Repository from the book "Machine Learning for Trading."
The book is also worth reading. Check the alpha factor library if you're considering trading price data (eg Prediction or DRL.)
https://github.com/stefan-jansen/machine-learning-for-trading

