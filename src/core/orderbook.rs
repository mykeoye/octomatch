use rust_decimal::Decimal;
use super::{
    model::{Event, Order, TradingPair},
    pqueue::{OrderQueue, PriceTimePriorityOrderQueue},
    types::{Failure, OrderId, OrderSide, OrderStatus, OrderType},
};

const ORDER_BOOK_INITIAL_CAPACITY: usize = 512;

pub trait OrderBook {
    fn cancel(&mut self, order_id: OrderId) -> Result<Event, Failure>;
    fn place(&mut self, order: Order) -> Result<Event, Failure>;
    fn peek_top_ask(&self) -> Option<&Order>;
    fn peek_top_bid(&self) -> Option<&Order>;
    fn get_spread(&self) -> Option<Decimal>;
}

pub struct LimitOrderBook {
    trading_pair: TradingPair,
    bids: PriceTimePriorityOrderQueue,
    asks: PriceTimePriorityOrderQueue,
}

impl LimitOrderBook {
    pub fn init(trading_pair: TradingPair) -> LimitOrderBook {
        Self {
            trading_pair,
            bids: PriceTimePriorityOrderQueue::with_capacity(ORDER_BOOK_INITIAL_CAPACITY),
            asks: PriceTimePriorityOrderQueue::with_capacity(ORDER_BOOK_INITIAL_CAPACITY),
        }
    }
}

impl OrderBook for LimitOrderBook {
    fn cancel(&mut self, order_id: OrderId) -> Result<Event, Failure> {
        // check the bid queues to see if we can find the order
        if let Some(_) = self.bids.find(order_id) {
            self.bids.remove(order_id);
            return Ok(Event {
                order_id,
                status: OrderStatus::Canceled,
                at_price: String::from(""),
            });
        }
        // else we check the ask queues to see if we can find it there
        if let Some(_) = self.asks.find(order_id) {
            self.asks.remove(order_id);
            return Ok(Event {
                order_id,
                status: OrderStatus::Canceled,
                at_price: String::from(""),
            });
        }
        Err(Failure::OrderNotFound("No order found with the given id"))
    }

    fn place(&mut self, order: Order) -> Result<Event, Failure> {
        if OrderType::Market == order.order_type {
            return Err(Failure::OrderRejected(
                "Only limit orders can be placed in the orderbook",
            ));
        }
        match order.side {
            OrderSide::Bid => self.bids.push(order),
            OrderSide::Ask => self.asks.push(order),
        };
        Ok(Event {
            status: OrderStatus::Created,
            order_id: order.order_id,
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
        self.asks.peek()
    }

    fn peek_top_bid(&self) -> Option<&Order> {
        self.bids.peek()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use rust_decimal::Decimal;

    use crate::core::{
        model::{Order, TradingPair},
        types::{Asset, Failure, OrderSide, OrderStatus, OrderType},
    };

    use super::{LimitOrderBook, OrderBook};

    #[test]
    fn can_place_a_limit_order_in_the_order_book() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::BTC,
            price_asset: Asset::ETH,
        });

        let result = orderbook.place(Order {
            order_id: 8,
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            order_type: OrderType::Limit,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
        });

        let event = result.unwrap();
        assert_eq!(OrderStatus::Created, event.status);
    }

    #[test]
    fn market_orders_cannot_be_inserted_into_the_orderbook() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::BTC,
            price_asset: Asset::USDT,
        });

        let result = orderbook.place(Order {
            order_id: 8,
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            order_type: OrderType::Market,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
        });

        let failure = result.unwrap_err();
        assert_eq!(
            Failure::OrderRejected("Only limit orders can be placed in the orderbook"),
            failure
        );
    }

    #[test]
    fn canceling_a_limit_order_is_should_be_allowed() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::BTC,
            price_asset: Asset::USDT,
        });

        let result = orderbook.place(Order {
            order_id: 8,
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            order_type: OrderType::Limit,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
        });

        let event = result.unwrap();
        let result = orderbook.cancel(event.order_id);

        let event = result.unwrap();
        assert_eq!(OrderStatus::Canceled, event.status);
    }

    #[test]
    fn an_empty_orderbook_should_have_no_spread() {
        let orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::BTC,
            price_asset: Asset::USDT,
        });

        let spread = orderbook.get_spread();
        assert_eq!(spread, None)
    }

    #[test]
    fn an_orderbook_with_a_single_bid_or_ask_has_no_spread() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::BTC,
            price_asset: Asset::USDT,
        });

        let result = orderbook.place(Order {
            order_id: 8,
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            order_type: OrderType::Limit,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
        });

        let _event = result.unwrap();

        let spread = orderbook.get_spread();
        assert_eq!(spread, None)
    }

    #[test]
    fn the_spread_can_be_gotten_for_a_book_with_both_sides() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::BTC,
            price_asset: Asset::USDT,
        });

        let orders = vec![
            Order {
                order_id: 8,
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Ask,
                quantity: 8,
                order_type: OrderType::Limit,
                timestamp: 1678170180000,
                trading_pair: TradingPair {
                    order_asset: Asset::ETH,
                    price_asset: Asset::USDC,
                },
            },
            Order {
                order_id: 18,
                price: Decimal::from_str("100.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 8,
                order_type: OrderType::Limit,
                timestamp: 1678170180000,
                trading_pair: TradingPair {
                    order_asset: Asset::ETH,
                    price_asset: Asset::USDC,
                },
            },
        ];

        for order in orders.iter() {
            let _res = orderbook.place(*order);
        }

        let spread = orderbook.get_spread().unwrap();
        assert_eq!(spread, Decimal::from_str("-100.00").unwrap());
    }
}
