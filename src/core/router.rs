use std::collections::HashMap;

use rust_decimal::Decimal;
use uuid::Uuid;

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

pub trait Router {
    fn route(&mut self, request: Request);
}

pub struct OrderRouter<T> {
    books: HashMap<TradingPair, T>,
    matcher: Matcher,
}

impl<T> OrderRouter<T> {
    pub fn new() -> Self {
        Self {
            books: HashMap::with_capacity(16),
            matcher: Matcher {},
        }
    }
}

impl<T> Router for OrderRouter<T>
where
    T: OrderBook,
{
    fn route(&mut self, request: Request) {
        // we should run prevalidations before the request gets here then we find the book that
        // matches the trading pair and then pass it to the matcher along with the order
        match request {
            Request::PlaceOrder(p) => {
                let order = p.to_order();
                match self.books.get_mut(&order.trading_pair) {
                    Some(book) => self.matcher.match_order(order, book),
                    None => todo!(),
                };
            }
            Request::Cancel(r) => {
                let _ = match self.books.get_mut(&r.trading_pair) {
                    Some(book) => book.cancel(r.orderid),
                    None => todo!(),
                };
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
