use std::collections::HashMap;

use rust_decimal::Decimal;

use super::{
    matcher::Matcher,
    model::{Order, TradingPair},
    orderbook::OrderBook,
    types::{Long, OrderId, OrderSide, OrderType},
    utils::Util,
};

#[derive(Debug)]
pub enum Request {
    PlaceOrder(PlaceOrder),
    Cancel(CancelOrder),
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
            orderid: 3, // this should be uniquely generated
            price: self.price,
            quantity: self.quantity,
            side: self.side,
            order_type: self.order_type,
            trading_pair: self.trading_pair,
            timestamp: Util::current_time_millis(), // require a utility for generating timestamps
        }
    }
}

#[derive(Debug)]
pub struct CancelOrder {
    pub orderid: OrderId,
}

pub trait Router {
    fn route(request: Request);
}

pub struct OrderRouter<T> {
    books: HashMap<TradingPair, T>,
    matcher: Matcher,
}

impl<T> Router for OrderRouter<T>
where
    T: OrderBook,
{
    fn route(request: Request) {
        // we should run prevalidations before the request gets here then we find the book that
        // matches the trading pair and then pass it to the matcher along with the order
        match request {
            Request::PlaceOrder(r) => {}
            Request::Cancel(r) => {}
        }
    }
}
