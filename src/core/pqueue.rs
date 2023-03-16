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
        if let Some(key) = self.heap.peek() {
            return self.orders.get(&key.orderid);
        }
        None
    }

    fn pop(&mut self) -> Option<Order> {
        if let Some(key) = self.heap.pop() {
            return self.orders.remove(&key.orderid);
        }
        None
    }

    fn remove(&mut self, orderid: OrderId) -> Option<Order> {
        if let Some(order) = self.orders.remove(&orderid) {
            // This creates a copy of the elements in the heap to satisfy the borrow checker.
            // I'm new to rust so i'll keep this until i find a much better way. Need to figure out
            // a way to not do this needless copy. And just modify using a reference to the existing data
            let mut key_vec = self.heap.to_owned().into_vec();
            key_vec.retain(|k| k.orderid != order.orderid);
            self.heap = BinaryHeap::from(key_vec);
            return Some(order);
        }
        None
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
    use std::str::FromStr;

    use super::*;

    #[test]
    fn can_get_an_inserted_order_back_from_queue() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            Order {
                orderid: 8,
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 4,
                order_type: OrderType::Limit,
                timestamp: 1678170180000,
                trading_pair: TradingPair {
                    order_asset: Asset::BTC,
                    price_asset: Asset::USDC,
                },
            },
            Order {
                orderid: 9,
                price: Decimal::from_str("300.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 10,
                order_type: OrderType::Limit,
                timestamp: 1680848580000,
                trading_pair: TradingPair {
                    order_asset: Asset::DOT,
                    price_asset: Asset::USDT,
                },
            },
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
            Order {
                orderid: 8,
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 8,
                order_type: OrderType::Limit,
                timestamp: 1678170180000,
                trading_pair: TradingPair {
                    order_asset: Asset::DOT,
                    price_asset: Asset::USDT,
                },
            },
            Order {
                orderid: 9,
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 12,
                order_type: OrderType::Limit,
                timestamp: 1680848580000,
                trading_pair: TradingPair {
                    order_asset: Asset::ETH,
                    price_asset: Asset::USDT,
                },
            },
        ];

        orders.iter().for_each(|key| pq.push(*key));

        let head: Order = *pq.peek().unwrap();
        assert_eq!(orders[0], head, "For orders with the same price, the longest staying order should be at the head of the queue");
    }

    #[test]
    fn orders_can_be_removed_from_queue_if_they_are_canceled() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let order = Order {
            orderid: 8,
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            order_type: OrderType::Limit,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::DOT,
                price_asset: Asset::USDT,
            },
        };

        pq.push(order);
        assert_eq!(order, *pq.peek().unwrap());

        pq.remove(order.orderid);
        assert_eq!(None, pq.peek());
    }

    #[test]
    fn orders_can_be_poped_from_queue_when_needed() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let order = Order {
            orderid: 8,
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            order_type: OrderType::Limit,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
        };

        pq.push(order);
        assert_eq!(order, pq.pop().unwrap());
    }
}
