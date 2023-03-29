## Octomatch
Octomatch is minimal implementation of an order matching engine with support for Market and Limit order types.
Matching is based on price-time priority.

The current implementation is single threaded and has no server or cli for issuing commands. A future iteration
of this project will implement a web server alongside a REST API for clients to interact with.

## Features
- Matching limit and market orders
- Event dispatching via log streams
- Support for multiple order books 
- Best price matching based on price time priority

## How to use
You can run the example provided in the `bin/main.rs` file. Feel free to tweak as you like

### Initializing the engine
```
    // You can load as many books as you want. This example loads two books
    let mut engine = Engine::new(EngineConfig::build(vec![
        TradingPair::from(Asset::BTC, Asset::USDC),
        TradingPair::from(Asset::BTC, Asset::USDT),
    ]));
```
### Dispatching requests

#### Place an order
```
    engine.dispatch(
        PlaceOrder::from(
                dec!(20.00),
                10,
                OrderSide::Bid,
                OrderType::Limit,
                TradingPair::from(Asset::BTC, Asset::USDC)
        )
    )
```

#### Cancel an order
```
    engine.dispatch(
        CancelOrder::from(
            Uuid::new_v4(), 
            TradingPair::from(Asset::BTC, Asset::USDC)
        )
    )
```

#### Order types
```
pub enum OrderType {
    Market,
    Limit,
    Stop,
}
```

If you prefer to view the documentation locally, simple run `cargo doc --open` in the root of the project
in your terminal

## Future additions
- Add multi-threaded support
- Implement a pro-rata matching algorithm
- Mount a HTTP server and expose a Rest api for issuing orders
- Web sockets for listening to events
- Make into a docker image
- Save periodic snapshots of the state of the order book to disk
