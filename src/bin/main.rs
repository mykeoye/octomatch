use octomatch::{
    core::{
        model::TradingPair,
        router::{CancelOrder, PlaceOrder, Request},
        types::{Asset, OrderSide, OrderType},
    },
    Engine, EngineConfig,
};
use rust_decimal_macros::dec;
use uuid::Uuid;

fn main() {
    let mut engine = Engine::new(EngineConfig::build(vec![
        TradingPair::from(Asset::BTC, Asset::USDC),
        TradingPair::from(Asset::BTC, Asset::USDT),
    ]));

    for _ in 1..5 {
        engine.dispatch(Request::PlaceOrder({
            PlaceOrder::from(
                dec!(20.00),
                10,
                OrderSide::Bid,
                OrderType::Limit,
                TradingPair::from(Asset::BTC, Asset::USDC),
            )
        }));
    }

    CancelOrder::from(Uuid::new_v4(), TradingPair::from(Asset::BTC, Asset::USDC));
}
