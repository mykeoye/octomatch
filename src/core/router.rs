use std::collections::HashMap;

use rust_decimal::Decimal;
use uuid::Uuid;

use super::{
    matcher::Matcher,
    model::{Order, TradingPair},
    orderbook::OrderBook,
    types::{Failure, Long, OrderId, OrderSide, OrderType},
    utils::Util,
};

#[derive(Debug)]
pub enum Request {
    PlaceOrder(PlaceOrder),
    Cancel(CancelOrder),
}

impl Request {
    fn validate(&self) -> Option<Failure> {
        match self {
            Request::PlaceOrder(p) => {
                if p.quantity <= 0 {
                    return Some(Failure::OrderRejected(
                        "Quantity must be greater than zero".to_string(),
                    ));
                }
                p.trading_pair.validate()
            }
            Request::Cancel(c) => c.trading_pair.validate(),
        }
    }
}

#[derive(Debug)]
pub struct PlaceOrder {
    pub price: Decimal,
    pub quantity: Long,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub trading_pair: TradingPair,
}

impl PlaceOrder {
    pub fn to_order(&self) -> Order {
        Order {
            orderid: Uuid::new_v4(),
            price: self.price,
            quantity: self.quantity,
            side: self.side,
            order_type: self.order_type,
            trading_pair: self.trading_pair,
            timestamp: Util::current_time_millis(),
        }
    }
}

#[derive(Debug)]
pub struct CancelOrder {
    pub orderid: OrderId,
    pub trading_pair: TradingPair,
}

/// The router interface is responsible for handling different request types
/// and routing an order to the appropriate order book, for matching
pub trait Router {
    fn handle(&mut self, request: Request) -> Result<(), Failure>;
}

pub struct OrderRouter<T> {
    books: HashMap<TradingPair, T>,
    matcher: Matcher,
}

impl<T> OrderRouter<T> {
    pub fn new() -> Self {
        Self {
            books: HashMap::with_capacity(16),
            matcher: Matcher,
        }
    }
    pub fn with_books(books: HashMap<TradingPair, T>) -> Self {
        Self {
            books,
            matcher: Matcher,
        }
    }
}

impl<T> Router for OrderRouter<T>
where
    T: OrderBook,
{
    fn handle(&mut self, request: Request) -> Result<(), Failure> {
        match request.validate() {
            Some(failure) => Err(failure),
            None => {
                match request {
                    Request::PlaceOrder(p) => {
                        let order = p.to_order();
                        return match self.books.get_mut(&order.trading_pair) {
                            Some(book) => {
                                // should be replaced with a thread pool in subsequent iterations
                                self.matcher.match_order(order, book);
                                Ok(())
                            }
                            None => Err(Failure::BookNotFound(format!(
                                "No book found for trading pair {:?}",
                                p.trading_pair
                            ))),
                        };
                    }
                    Request::Cancel(r) => {
                        return match self.books.get_mut(&r.trading_pair) {
                            Some(book) => {
                                // this should run on a separate thread of execution, utilizing a thread pool
                                let _ = book.cancel(r.orderid);
                                Ok(())
                            }
                            None => Err(Failure::BookNotFound(format!(
                                "No book found for trading pair {:?}",
                                r.trading_pair
                            ))),
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use crate::core::{orderbook::LimitOrderBook, types::Asset};

    use super::*;

    #[test]
    fn placing_an_order_in_an_empty_book_should_fail() {
        let request = Request::PlaceOrder(PlaceOrder {
            price: dec!(300.00),
            quantity: 2,
            side: OrderSide::Bid,
            order_type: OrderType::Limit,
            trading_pair: TradingPair::from(Asset::BTC, Asset::USDC),
        });

        let mut router: OrderRouter<LimitOrderBook> = OrderRouter::new();
        let result = router.handle(request);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            Failure::BookNotFound(
                format!(
                    "No book found for trading pair {:?}",
                    TradingPair::from(Asset::BTC, Asset::USDC)
                )
                .to_string()
            )
        )
    }

    #[test]
    fn an_invalid_order_should_fail_placement() {
        let request = Request::PlaceOrder(PlaceOrder {
            price: dec!(300.00),
            quantity: 0,
            side: OrderSide::Bid,
            order_type: OrderType::Limit,
            trading_pair: TradingPair::from(Asset::BTC, Asset::USDC),
        });

        let mut router: OrderRouter<LimitOrderBook> = OrderRouter::new();
        let result = router.handle(request);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            Failure::OrderRejected("Quantity must be greater than zero".to_string(),)
        )
    }

    #[test]
    fn a_valid_order_should_be_routed_successfully() {
        let trading_pair = TradingPair::from(Asset::BTC, Asset::USDC);

        let request = Request::PlaceOrder(PlaceOrder {
            price: dec!(300.00),
            quantity: 10,
            side: OrderSide::Bid,
            order_type: OrderType::Limit,
            trading_pair: trading_pair,
        });

        let mut router: OrderRouter<LimitOrderBook> = OrderRouter::with_books(HashMap::from([(
            TradingPair::from(Asset::BTC, Asset::USDC),
            LimitOrderBook::init(trading_pair),
        )]));
        let result = router.handle(request);
        assert!(result.is_ok())
    }
}
