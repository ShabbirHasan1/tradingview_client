use std::sync::Arc;
use std::time::Duration;

use tradingview_client::{DefaultTradingViewMessageProcessor, TradingViewClient, TradingViewClientConfig, TradingViewClientMode, TradingViewIndicators, TradingViewMessageProcessor, SPY5_EXT_SYMBOL, SPY5_REG_SYMBOL};

fn main() {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();

    // init env vars
    dotenvy::from_filename("./.env").expect("failed to load env vars");
    let auth_token = std::env::var("AUTH_TOKEN").expect("failed to get AUTH_TOKEN");

    // build message processor
    let message_processor1: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));
    let message_processor2: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));
        
    // build clients
    let vwap_mvwap_ema_crossover = TradingViewIndicators::generate_vwap_mvwap_ema_crossover(
      1,
      "close".to_string(),
      7,
      "close".to_string(),
      25,
      65,
      51,
      21
    );
    let configs = vec![
        TradingViewClientConfig {
            name: "SPY5REG".to_string(),
            auth_token: auth_token.clone(),
            chart_symbols: vec![SPY5_REG_SYMBOL.to_string()],
            quote_symbols: vec![SPY5_REG_SYMBOL.to_string()],
            indicators: vec![
              vwap_mvwap_ema_crossover.clone()
            ],
            timeframe: "5".to_string(),
            range: 300,
            message_processor: message_processor1.clone(),
            mode: TradingViewClientMode::Streaming
        },

        TradingViewClientConfig {
            name: "SPY5EXT".to_string(),
            auth_token: auth_token.clone(),
            chart_symbols: vec![SPY5_EXT_SYMBOL.to_string()],
            quote_symbols: vec![SPY5_EXT_SYMBOL.to_string()],
            indicators: vec![
              vwap_mvwap_ema_crossover.clone()
            ],
            timeframe: "5".to_string(),
            range: 300,
            message_processor: message_processor2.clone(),
            mode: TradingViewClientMode::Streaming
        },
    ];

    // spawn clients on threads
    let mut handles = vec![];
    for config in configs {
        handles.push(std::thread::spawn(move || {
            futures_lite::future::block_on(async {
                let client: TradingViewClient = config.to_client();
                match client.run().await {
                    Ok(_) => (),
                    Err(err) => panic!("{err}"),
                }
            })
        }));
    }

    // watch handles
    loop {
        for handle in &handles {
            if handle.is_finished() {
                panic!("a handle finished");
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    }
}
