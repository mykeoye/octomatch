use super::{
    model::{Event, Order, Spread, TradingPair},
    pqueue::{OrderQueue, PriceTimePriorityOrderQueue},
    types::{Failure, OrderId, OrderSide, OrderStatus, OrderType},
};

const ORDER_BOOK_INITIAL_CAPACITY: usize = 512;

pub trait OrderBook {
    fn cancel(&mut self, order_id: OrderId) -> Result<Event, Failure>;
    fn place(&mut self, order: Order) -> Result<Event, Failure>;
    fn get_top_ask(&self) -> Option<Order>;
    fn get_spread(&self) -> Option<Spread>;
    fn get_top_bid(&self) -> Option<Order>;
}

pub struct LimitOrderBook {
    pub trading_pair: TradingPair,
    pub bids: PriceTimePriorityOrderQueue,
    pub asks: PriceTimePriorityOrderQueue,
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
        todo!()
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

    fn get_spread(&self) -> Option<Spread> {
        todo!()
    }

    fn get_top_ask(&self) -> Option<Order> {
        todo!()
    }

    fn get_top_bid(&self) -> Option<Order> {
        todo!()
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
}
