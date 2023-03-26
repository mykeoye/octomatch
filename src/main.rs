use octomatch::{
    core::{
        model::TradingPair,
        router::{PlaceOrder, Request},
        types::{Asset, OrderSide, OrderType},
    },
    Engine, EngineConfig,
};
use rust_decimal_macros::dec;

fn main() {
    let mut engine = Engine::new(EngineConfig::build(vec![
        TradingPair::from(Asset::BTC, Asset::USDC),
        TradingPair::from(Asset::BTC, Asset::USDT),
    ]));

    for _ in 1..10 {
        engine.dispatch(Request::PlaceOrder({
            PlaceOrder {
                price: dec!(20.00),
                quantity: 10,
                side: OrderSide::Bid,
                order_type: OrderType::Limit,
                trading_pair: TradingPair::from(Asset::BTC, Asset::USDC),
            }
        }));
    }
}
