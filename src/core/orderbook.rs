use std::collections::HashMap;

use super::{
    model::{Event, Order, OrderKey, TradingPair},
    pqueue::{OrderQueue, PriceTimePriorityOrderQueue},
    types::{Failure, Long, OrderId, OrderSide, OrderStatus, OrderType},
};
use rust_decimal::Decimal;

/// The order queues should be able to hold these number of items when created
const ORDER_BOOK_INITIAL_CAPACITY: usize = 16;

/// This trait defines the operations that can be performed by the orderbook. It
/// embodies the basic operations that are typical of an orderbook
pub trait OrderBook {
    /// Cancel an open order in the book. Cancelling a non-existent order should fail
    fn cancel(&mut self, orderid: OrderId) -> Result<Event, Failure>;

    /// Place an order into the book, should the order already exists it should also fail
    fn place(&mut self, order: Order) -> Result<Event, Failure>;

    /// Gets the ask at the top of the book (head of the ask queue)
    fn peek_top_ask(&self) -> Option<&Order>;

    /// Gets the bid at the top of the book (head of the bid queue)
    fn peek_top_bid(&self) -> Option<&Order>;

    /// Gets the spread, which is the difference between the top ask and bid prices
    fn get_spread(&self) -> Option<Decimal>;

    /// Allows for the modification of the order quantity in-place
    fn modify_quantity(&mut self, orderid: OrderId, qty: Long);

    /// Removes the top bid from the head of the queue
    fn pop_top_bid(&mut self) -> Option<Order>;

    /// Removes the top ask from the head of the ask queue
    fn pop_top_ask(&mut self) -> Option<Order>;
}

/// An implementation of the [OrderBook] trait. This implementation uses two queues one for
/// storing bid order and the other for storing ask orders. Together the form the orderbook
///
/// Each [TradingPair] has its own limit book, so there will be N order books at a given time
/// for N trading pairs. The trading pairs represent the order and price assets been traded
///
/// Each orderbook can only trade assets for the trading pair that it supports
///
pub struct LimitOrderBook {
    trading_pair: TradingPair,
    bids: PriceTimePriorityOrderQueue<OrderKey>,
    asks: PriceTimePriorityOrderQueue<OrderKey>,
    orders: HashMap<OrderId, Order>,
}

impl LimitOrderBook {
    pub fn init(trading_pair: TradingPair) -> LimitOrderBook {
        Self {
            trading_pair,
            bids: PriceTimePriorityOrderQueue::with_capacity(ORDER_BOOK_INITIAL_CAPACITY),
            asks: PriceTimePriorityOrderQueue::with_capacity(ORDER_BOOK_INITIAL_CAPACITY),
            orders: HashMap::with_capacity(ORDER_BOOK_INITIAL_CAPACITY),
        }
    }
}

impl OrderBook for LimitOrderBook {
    fn cancel(&mut self, orderid: OrderId) -> Result<Event, Failure> {
        match self.orders.remove(&orderid) {
            Some(order) => {
                match order.side {
                    OrderSide::Bid => self.bids.remove(order.to_key()),
                    OrderSide::Ask => self.asks.remove(order.to_key()),
                };
                return Ok(Event {
                    orderid,
                    status: OrderStatus::Canceled,
                    at_price: String::from(""),
                });
            }
            None => Err(Failure::OrderNotFound(
                "No order found with the given id".to_string(),
            )),
        }
    }

    fn place(&mut self, order: Order) -> Result<Event, Failure> {
        if OrderType::Market == order.order_type {
            return Err(Failure::OrderRejected(
                "Only limit orders can be placed in the orderbook".to_string(),
            ));
        }
        if self.trading_pair != order.trading_pair {
            return Err(Failure::InvalidOrderForBook);
        }

        self.orders.insert(order.orderid, order);

        match order.side {
            OrderSide::Bid => self.bids.push(order.to_key()),
            OrderSide::Ask => self.asks.push(order.to_key()),
        };
        Ok(Event {
            status: OrderStatus::Created,
            orderid: order.orderid,
            at_price: String::from(""),
        })
    }

    fn get_spread(&self) -> Option<Decimal> {
        match self.bids.peek() {
            Some(bid) => match self.asks.peek() {
                Some(ask) => Some(bid.price - ask.price),
                None => None,
            },
            None => None,
        }
    }

    fn peek_top_ask(&self) -> Option<&Order> {
        if let Some(key) = self.asks.peek() {
            return self.orders.get(&key.orderid);
        }
        None
    }

    fn peek_top_bid(&self) -> Option<&Order> {
        if let Some(key) = self.bids.peek() {
            return self.orders.get(&key.orderid);
        }
        None
    }

    fn modify_quantity(&mut self, orderid: OrderId, quantity: Long) {
        if let Some(order) = self.orders.get_mut(&orderid) {
            order.quantity = quantity
        }
    }

    fn pop_top_bid(&mut self) -> Option<Order> {
        if let Some(key) = self.bids.pop() {
            return self.orders.remove(&key.orderid);
        }
        None
    }

    fn pop_top_ask(&mut self) -> Option<Order> {
        if let Some(key) = self.asks.pop() {
            return self.orders.remove(&key.orderid);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use crate::core::{
        model::{Order, TradingPair},
        types::{Asset, Failure, Long, OrderSide, OrderStatus, OrderType},
        utils::Util,
    };

    use super::{LimitOrderBook, OrderBook};

    #[test]
    fn can_place_a_limit_order_in_the_order_book() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::BTC, Asset::ETH));

        let order = create_order(
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair {
                order_asset: Asset::BTC,
                price_asset: Asset::ETH,
            },
        );
        let result = orderbook.place(order);

        let event = result.unwrap();
        assert_eq!(OrderStatus::Created, event.status);

        // we expect the order we placed to be at the top of the book
        let order_placed = orderbook.peek_top_bid();
        assert_eq!(*order_placed.unwrap(), order);
    }

    #[test]
    fn market_orders_cannot_be_inserted_into_the_orderbook() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::BTC, Asset::USDT));

        let result = orderbook.place(create_order(
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Market,
            TradingPair {
                order_asset: Asset::BTC,
                price_asset: Asset::USDT,
            },
        ));

        let failure = result.unwrap_err();
        assert_eq!(
            Failure::OrderRejected("Only limit orders can be placed in the orderbook".to_string()),
            failure
        );
    }

    #[test]
    fn canceling_a_limit_order_is_should_be_allowed() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::BTC, Asset::USDT));

        let result = orderbook.place(create_order(
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair::from(Asset::BTC, Asset::USDT),
        ));

        let event = result.unwrap();
        let result = orderbook.cancel(event.orderid);

        let event = result.unwrap();
        assert_eq!(OrderStatus::Canceled, event.status);
    }

    #[test]
    fn an_empty_orderbook_should_have_no_spread() {
        let orderbook = LimitOrderBook::init(TradingPair::from(Asset::BTC, Asset::USDT));

        let spread = orderbook.get_spread();
        assert_eq!(spread, None)
    }

    #[test]
    fn an_orderbook_with_a_single_bid_or_ask_has_no_spread() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::ETH, Asset::USDC));

        let result = orderbook.place(create_order(
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair::from(Asset::ETH, Asset::USDC),
        ));

        let _event = result.unwrap();

        let spread = orderbook.get_spread();
        assert_eq!(spread, None)
    }

    #[test]
    fn the_spread_can_be_gotten_for_a_book_with_both_sides() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::ETH, Asset::USDC));

        let orders = vec![
            create_order(
                dec!(200.02),
                OrderSide::Ask,
                8,
                OrderType::Limit,
                TradingPair::from(Asset::ETH, Asset::USDC),
            ),
            create_order(
                dec!(100.02),
                OrderSide::Bid,
                8,
                OrderType::Limit,
                TradingPair::from(Asset::ETH, Asset::USDC),
            ),
        ];

        for order in orders.iter() {
            let _res = orderbook.place(*order);
        }

        let spread = orderbook.get_spread().unwrap();
        assert_eq!(spread, Decimal::from_str("-100.00").unwrap());
    }

    fn create_order(
        price: Decimal,
        side: OrderSide,
        quantity: Long,
        order_type: OrderType,
        trading_pair: TradingPair,
    ) -> Order {
        Order {
            orderid: Uuid::new_v4(),
            price,
            side,
            quantity,
            order_type,
            timestamp: Util::current_time_millis(),
            trading_pair,
        }
    }
}
