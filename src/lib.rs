//! Octomatch is simple and minimal order matching engine
//!
//! This project was created out of the curious need to understand how
//! matching engines work. It isn't meant to be complete solution but
//! it should help as a learning tool.
//!
//! The current implementation is single threaded and provides no
//! interface for sending commands into the system ie it has no
//! server or a cli
//!
//! Since it is still a work in progress those will be added much later
//!
//! # How to use
//!
//! ```
//! use octomatch::{
//!     core::{
//!         model::TradingPair,
//!         router::{CancelOrder, PlaceOrder, Request},
//!         types::{Asset, OrderSide, OrderType},
//!         },
//!         Engine, EngineConfig,
//!     };
//!     use rust_decimal_macros::dec;
//!     use uuid::Uuid;
//!
//!     let mut engine = Engine::new(EngineConfig::build(vec![
//!         TradingPair::from(Asset::BTC, Asset::USDC),
//!          TradingPair::from(Asset::BTC, Asset::USDT),
//!      ]));
//!
//!     engine.dispatch(Request::PlaceOrder({
//!         PlaceOrder::from(
//!             dec!(20.00),
//!             10,
//!             OrderSide::Bid,
//!             OrderType::Limit,
//!             TradingPair::from(Asset::BTC, Asset::USDC),
//!         )
//!     }));
//!
//! ```
//!
//! Events are dispatched a logs in the terminal so you get to see the output
//! of the requests you disptach, in real time
//!

use crate::core::model::TradingPair;
use crate::core::orderbook::LimitOrderBook;
use crate::core::router::Request;
use crate::core::router::Router;
use log::error;
use log::info;
use std::collections::HashMap;

pub mod core;

/// Configuration for tweaking the engine. Will have support for configuring threadpools much later
pub struct EngineConfig {
    books: Vec<TradingPair>,
}

impl EngineConfig {
    pub fn build(books: Vec<TradingPair>) -> Self {
        Self { books }
    }
}

/// The driver for the order matching engine. Current implementation is single threaded
pub struct Engine {
    /// a single threaded router for manging requests to the engine
    router: Router<LimitOrderBook>,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        let trading_pairs = config.books;
        let mut books: HashMap<TradingPair, LimitOrderBook> =
            HashMap::with_capacity(trading_pairs.len());
        for trading_pair in trading_pairs {
            books.insert(trading_pair, LimitOrderBook::init(trading_pair));
        }
        Self {
            router: Router::with_books(books),
        }
    }

    pub fn dispatch(&mut self, request: Request) {
        if let Err(failure) = self.router.handle(request.clone()) {
            error!("Dispatching request {:?} failed {:?}", failure, request);
        } else {
            info!("Request {:?} successfully dispatched", request)
        }
    }
}
