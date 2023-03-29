use std::{collections::HashMap, convert, sync::Mutex};

use rust_decimal::Decimal;
use uuid::Uuid;

use super::{
    matcher::Matcher,
    model::{Order, TradingPair},
    orderbook::OrderBook,
    types::{Failure, Long, OrderId, OrderSide, OrderType},
    utils::Util,
};

#[derive(Debug, Clone)]
pub enum Request {
    PlaceOrder(PlaceOrder),
    Cancel(CancelOrder),
}

impl Request {
    fn validate(&self) -> Option<Failure> {
        match self {
            Request::PlaceOrder(p) => p.validate(),
            Request::Cancel(c) => c.trading_pair.validate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaceOrder {
    price: Decimal,
    quantity: Long,
    side: OrderSide,
    order_type: OrderType,
    trading_pair: TradingPair,
}

impl PlaceOrder {
    pub fn from(
        price: Decimal,
        quantity: Long,
        side: OrderSide,
        order_type: OrderType,
        trading_pair: TradingPair,
    ) -> Self {
        Self {
            price,
            quantity,
            side,
            order_type,
            trading_pair,
        }
    }

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
    pub fn validate(&self) -> Option<Failure> {
        if self.quantity <= 0 {
            return Some(Failure::OrderRejected(
                "Quantity must be greater than zero".to_string(),
            ));
        }
        return self.trading_pair.validate();
    }
}

#[derive(Debug, Clone)]
pub struct CancelOrder {
    orderid: OrderId,
    trading_pair: TradingPair,
}

impl CancelOrder {
    pub fn from(orderid: OrderId, trading_pair: TradingPair) -> Self {
        Self {
            orderid,
            trading_pair,
        }
    }
}

/// The router interface is responsible for handling different request types and routing an
/// order to the appropriate order book, for matching
#[derive(Debug)]
pub struct Router<T> {
    books: Mutex<HashMap<TradingPair, T>>,
    matcher: Matcher,
}
impl<T> Router<T>
where
    T: OrderBook,
{
    pub fn new() -> Self {
        Self {
            books: Mutex::new(HashMap::with_capacity(16)),
            matcher: Matcher,
        }
    }
    pub fn with_books(books: HashMap<TradingPair, T>) -> Self {
        Self {
            books: Mutex::new(books),
            matcher: Matcher,
        }
    }

    pub fn handle(&self, request: Request) -> Result<(), Failure> {
        match request.validate() {
            Some(failure) => Err(failure),
            None => match request {
                Request::PlaceOrder(p) => {
                    let order = p.to_order();

                    return self
                        .books
                        .try_lock()
                        .map_err(|_| Failure::EngineOverCapacity)
                        .map(|mut book| match book.get_mut(&order.trading_pair) {
                            Some(book) => {
                                self.matcher.match_order(order, book);
                                Ok(())
                            }
                            None => Err(Failure::BookNotFound(format!(
                                "No book found for trading pair {:?}",
                                p.trading_pair
                            ))),
                        })
                        .and_then(convert::identity);
                }
                Request::Cancel(cancel) => {
                    return self
                        .books
                        .try_lock()
                        .map_err(|_| Failure::EngineOverCapacity)
                        .map(|mut book| match book.get_mut(&cancel.trading_pair) {
                            Some(book) => {
                                let _ = book.cancel(cancel.orderid);
                                Ok(())
                            }
                            None => Err(Failure::BookNotFound(format!(
                                "No book found for trading pair {:?}",
                                cancel.trading_pair
                            ))),
                        })
                        .and_then(convert::identity);
                }
            },
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

        let router: Router<LimitOrderBook> = Router::new();
        let result = router.handle(request);
        dbg!(&result);
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

        let router: Router<LimitOrderBook> = Router::new();
        let result = router.handle(request);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            Failure::OrderRejected("Quantity must be greater than zero".to_string())
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
            trading_pair,
        });

        let router = Router::with_books(HashMap::from([(
            TradingPair::from(Asset::BTC, Asset::USDC),
            LimitOrderBook::init(trading_pair),
        )]));
        let result = router.handle(request);
        assert!(result.is_ok())
    }
}
