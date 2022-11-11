//! contains an abstraction for the application configuration.
// [jasongoodwin - 2022/11/10] may need to be made a bit more exchange specific.
use crate::result::Result;
use config::Config;

// toml file name/location for application config. (Settings.toml used to follow the lib's examples)
// TODO [jasongoodwin 2022/11/10] - make this configurable (env variable or arg)
#[cfg(not(test))]
const SETTINGS: &str = "Settings";

// a stable file is used for testing.
#[cfg(test)]
const SETTINGS: &str = "TestSettings";

#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeConfig {
    pub(crate) id: String,
    pub(crate) endpoint: String,
    pub(crate) subscription_message_template: String,
    pub(crate) spot_pairs: Vec<String>,
}

pub struct AppConfig {
    config: Config,
}

impl AppConfig {
    /// returns a new app config holding the settings. (note the file is dynamically read currently...)
    pub fn new() -> Result<AppConfig> {
        println!("Using config file: {}.toml", SETTINGS);
        let config: Config = Config::builder()
            .add_source(config::File::with_name(SETTINGS))
            .build()?;

        Ok(AppConfig { config })
    }

    /// returns a list of enabled spot pairs.
    pub fn spot_pairs(&self) -> Result<Vec<String>> {
        Ok(self.config.get("spot.pairs")?)
    }

    /// returns a list of enabled exchanges. They should also exist in the [exchanges] config section!
    pub fn enabled_exchanges(&self) -> Result<Vec<String>> {
        Ok(self.config.get("enabled_exchanges")?)
    }

    /// get_exchange_configs gets the ExchangeConfig for each of the enabled_exchanges.
    pub fn exchange_configs(&self) -> Result<Vec<ExchangeConfig>> {
        let mut exchange_configs = vec![];
        for id in self.enabled_exchanges()?.into_iter() {
            // get the conf.
            let endpoint = self.config.get::<String>(&*format!("{}.endpoint", id))?;

            let subscription_message_template = self
                .config
                .get::<String>(&*format!("{}.subscription_message_template", id))?;

            let spot_pairs = self.spot_pairs()?;

            exchange_configs.push(ExchangeConfig {
                id,
                endpoint,
                subscription_message_template,
                spot_pairs,
            });
        }

        Ok(exchange_configs)
    }
}

#[cfg(test)]
mod tests {
    // use crate::app_config::AppConfig;
    use super::*;

    #[test]
    fn should_instantiate() {
        let conf = AppConfig::new().unwrap();
    }

    #[test]
    fn should_provide_spot_pairs() -> Result<()> {
        let conf = AppConfig::new()?;
        let spot_pairs = conf.spot_pairs().unwrap();

        assert_eq!(spot_pairs.len(), 2);
        assert!(spot_pairs.contains(&"BTCUSDT".into()));
        assert!(spot_pairs.contains(&"ETHBTC".into()));

        Ok(())
    }

    #[test]
    fn should_provide_exchange_configs() -> Result<()> {
        let conf = AppConfig::new()?;
        let exchange_configs = conf.exchange_configs().unwrap();

        println!("{:?}", exchange_configs.get(1).unwrap());
        assert_eq!(exchange_configs.len(), 2);
        assert!(exchange_configs.contains(&ExchangeConfig {
            id: "bitstamp".to_string(),
            endpoint: "wss://ws.bitstamp.net".to_string(),
            subscription_message_template: r#"{
    "event": "bts:subscribe",
    "data": {
        "channel": "order_book_{{pair}}}}"
    }
}"#
            .to_string(),
            spot_pairs: vec!["BTCUSDT".into(), "ETHBTC".into()]
        }));

        assert!(exchange_configs.contains(&ExchangeConfig {
            id: "binance".to_string(),
            endpoint: "wss://stream.binance.com:9443".to_string(),
            subscription_message_template: r#"{
  "method": "SUBSCRIBE",
  "params": [
    "{{pair}}@depth10@100ms"
  ],
  "id": 1
}"#
            .to_string(),
            spot_pairs: vec!["BTCUSDT".into(), "ETHBTC".into()]
        }));

        Ok(())
    }
}
