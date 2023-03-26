use crate::core::model::TradingPair;
use crate::core::orderbook::LimitOrderBook;
use crate::core::router::Request;
use crate::core::router::Router;
use std::collections::HashMap;

pub mod core;

pub struct EngineConfig {
    books: Vec<TradingPair>,
}

impl EngineConfig {
    pub fn build(books: Vec<TradingPair>) -> Self {
        Self { books }
    }
}

pub struct Engine {
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
        if let Err(failure) = self.router.handle(request) {
            eprintln!("Dispatching request {:?} failed", failure);
        }
    }
}
