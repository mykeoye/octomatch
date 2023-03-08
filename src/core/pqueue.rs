use std::collections::{BinaryHeap, HashMap};
use super::model::{OrderKey, Order};
use super::types::{OrderId};

pub trait OrderQueue {
    fn push(&mut self, order: Order);
    fn peek(&self) -> Option<&Order>;
    fn pop(&mut self) -> Option<Order>;
    fn remove(&mut self, order_id: OrderId) -> Option<Order>;
}

pub struct PriceTimePriorityOrderQueue {
    heap: BinaryHeap<OrderKey>,
    orders: HashMap<OrderId, Order>,
}

impl PriceTimePriorityOrderQueue {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(16),
            orders: HashMap::with_capacity(16),
        }
    }
    fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
            orders: HashMap::with_capacity(capacity),
        }
    }
}

impl OrderQueue for PriceTimePriorityOrderQueue {

    fn push(&mut self, order: Order) {
        if self.orders.contains_key(&order.order_id) {
            return;
        }
        self.heap.push(order.to_key());
        self.orders.insert(order.order_id, order);
    }

    fn peek(&self) -> Option<&Order> {
        if let Some(key) = self.heap.peek() {
            return self.orders.get(&key.order_id)
        }
        None
    }

    fn pop(&mut self) -> Option<Order> {
        if let Some(key) = self.heap.pop() {
            return self.orders.remove(&key.order_id)
        }
        None
    }

    fn remove(&mut self, order_id: OrderId) -> Option<Order> {
        if let Some(order) = self.orders.remove(&order_id) {
            // This creates a copy of the elements in the heap to satisfy the borrow checker.
            // i'm new to rust so i let this slide. Need to figure out a way to not do this
            // needless copy. And just modify using a reference to the existing data
            let mut key_vec = self.heap.to_owned().into_vec();
            key_vec.retain(|k| k.order_id != order.order_id);
            self.heap = BinaryHeap::from(key_vec);
            return Some(order)
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use rust_decimal::Decimal;
    use crate::core::types::OrderSide;

    use super::*;

    #[test]
    fn can_get_an_inserted_order_back_from_queue() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            Order { 
                order_id: 8, 
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 4,
                timestamp: 1678170180000
            },
            Order {
                order_id: 9,
                price: Decimal::from_str("300.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 10,
                timestamp: 1680848580000
            }
        ];

        orders.iter().for_each(|key| pq.push(*key));

        let head: Order = *pq.peek().unwrap();
        assert_eq!(orders[1], head, "Asserting that the item at the head of the queue is the order with the highest price");
    }

    #[test]
    fn orders_at_the_same_price_are_prioritized_by_time() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            Order { 
                order_id: 8, 
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 8,
                timestamp: 1678170180000
            },
            Order {
                order_id: 9,
                price: Decimal::from_str("200.02").unwrap(),
                side: OrderSide::Bid,
                quantity: 12,
                timestamp: 1680848580000
            }
        ];

        orders.iter().for_each(|key| pq.push(*key));

        let head: Order = *pq.peek().unwrap();
        assert_eq!(orders[0], head, "For orders with the same price, the longest staying order should be at the head of the queue");
    }

    #[test]
    fn orders_can_be_removed_from_queue_if_they_are_canceled() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let order = Order { 
            order_id: 8, 
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            timestamp: 1678170180000
        };

        pq.push(order);
        assert_eq!(order, *pq.peek().unwrap());

        pq.remove(order.order_id);
        assert_eq!(None, pq.peek());
    }

    #[test]
    fn orders_can_be_poped_from_queue_when_needed() {
        let mut pq = PriceTimePriorityOrderQueue::new();

        let order = Order { 
            order_id: 8, 
            price: Decimal::from_str("200.02").unwrap(),
            side: OrderSide::Bid,
            quantity: 8,
            timestamp: 1678170180000
        };

        pq.push(order);
        assert_eq!(order, pq.pop().unwrap());
    }
}
