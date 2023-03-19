use super::model::{Order, OrderKey};
use super::types::{Long, OrderId};
use std::collections::{BinaryHeap, HashMap};

/// Defines the operations to be performed by an order queue. The implementation of this
/// is a priority queue that orders items based on some defined prioritization strategy
/// this is left entirely to the implementation of this trait
pub trait OrderQueue {
    fn push(&mut self, order: Order);
    fn peek(&self) -> Option<&Order>;
    fn pop(&mut self) -> Option<Order>;
    fn remove(&mut self, orderid: OrderId) -> Option<Order>;
    fn find(&self, orderid: OrderId) -> Option<&Order>;
    fn modify_quantity(&mut self, orderid: OrderId, quantity: Long);
}

pub struct PriceTimePriorityOrderQueue {
    heap: BinaryHeap<OrderKey>,
    orders: HashMap<OrderId, Order>,
}

impl PriceTimePriorityOrderQueue {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(16),
            orders: HashMap::with_capacity(16),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
            orders: HashMap::with_capacity(capacity),
        }
    }
}

impl OrderQueue for PriceTimePriorityOrderQueue {
    fn push(&mut self, order: Order) {
        if self.orders.contains_key(&order.orderid) {
            return;
        }
        self.heap.push(order.to_key());
        self.orders.insert(order.orderid, order);
    }

    fn peek(&self) -> Option<&Order> {
        match self.heap.peek() {
            Some(key) => self.orders.get(&key.orderid),
            None => None,
        }
    }

    fn pop(&mut self) -> Option<Order> {
        match self.heap.pop() {
            Some(key) => self.orders.remove(&key.orderid),
            None => None,
        }
    }

    fn remove(&mut self, orderid: OrderId) -> Option<Order> {
        match self.orders.remove(&orderid) {
            Some(order) => {
                // This seems to be the only way to effectively remove an item from the heap, by
                // iterating orver all the items, excluding the item we want to remove and rebuilding
                let mut key_vec = self.heap.to_owned().into_vec();
                key_vec.retain(|k| k.orderid != order.orderid);
                self.heap = key_vec.into();
                Some(order)
            }
            None => None,
        }
    }

    fn find(&self, orderid: OrderId) -> Option<&Order> {
        self.orders.get(&orderid)
    }

    fn modify_quantity(&mut self, orderid: OrderId, quantity: Long) {
        if let Some(order) = self.orders.get_mut(&orderid) {
            order.quantity = quantity;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::{
        model::TradingPair,
        types::{Asset, OrderSide, OrderType},
    };
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn can_get_an_inserted_order_back_from_queue() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            create_order(
                8,
                dec!(200.02),
                OrderSide::Bid,
                4,
                OrderType::Limit,
                TradingPair {
                    order_asset: Asset::BTC,
                    price_asset: Asset::USDC,
                },
                1678170180000,
            ),
            create_order(
                9,
                dec!(300.02),
                OrderSide::Bid,
                10,
                OrderType::Limit,
                TradingPair {
                    order_asset: Asset::DOT,
                    price_asset: Asset::USDT,
                },
                1680848580000,
            ),
        ];

        orders.iter().for_each(|key| pq.push(*key));

        let head: Order = *pq.peek().unwrap();
        assert_eq!(
            orders[1], head,
            "Asserting that the item at the head of the queue is the order with the highest price"
        );
    }

    #[test]
    fn orders_at_the_same_price_are_prioritized_by_time() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            create_order(
                8,
                dec!(200.02),
                OrderSide::Bid,
                4,
                OrderType::Limit,
                TradingPair {
                    order_asset: Asset::BTC,
                    price_asset: Asset::USDC,
                },
                1678170180000,
            ),
            create_order(
                9,
                dec!(200.02),
                OrderSide::Bid,
                12,
                OrderType::Limit,
                TradingPair {
                    order_asset: Asset::ETH,
                    price_asset: Asset::USDT,
                },
                1680848580000,
            ),
        ];

        orders.iter().for_each(|key| pq.push(*key));

        let head: Order = *pq.peek().unwrap();
        assert_eq!(orders[0], head, "For orders with the same price, the longest staying order should be at the head of the queue");
    }

    #[test]
    fn orders_can_be_removed_from_queue_if_they_are_canceled() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let order = create_order(
            8,
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair {
                order_asset: Asset::DOT,
                price_asset: Asset::USDT,
            },
            1678170180000,
        );

        pq.push(order);
        assert_eq!(order, *pq.peek().unwrap());

        pq.remove(order.orderid);
        assert_eq!(None, pq.peek());
    }

    #[test]
    fn orders_can_be_poped_from_queue_when_needed() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let order = create_order(
            8,
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
            1678170180000,
        );

        pq.push(order);
        assert_eq!(order, pq.pop().unwrap());
    }

    fn create_order(
        orderid: OrderId,
        price: Decimal,
        side: OrderSide,
        quantity: Long,
        order_type: OrderType,
        trading_pair: TradingPair,
        timestamp: Long,
    ) -> Order {
        Order {
            orderid,
            price,
            side,
            quantity,
            order_type,
            timestamp,
            trading_pair,
        }
    }
}
