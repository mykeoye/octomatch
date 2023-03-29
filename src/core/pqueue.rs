use std::collections::BinaryHeap;

/// A key index is a structure that defines some ordering, as well as information that
/// allows implementations of the order queue determine priority of items
pub trait KeyIndx: Clone + Ord + PartialEq + Copy {}

/// This trait defines the operations that should be performed by the order queue. It is
/// expected that the backing implemenation be a priority queue.
///
/// It is genric over type [T], which is any trait that implements the [KeyIndx] trait.
///  
/// [KeyIndx] provides the ordering, which determines how items are prioritized in the queue
///
pub trait OrderQueue<T: KeyIndx> {
    /// Pushes an item into the queue
    fn push(&mut self, item: T);

    // Gets the item at the head of the queue
    fn peek(&self) -> Option<&T>;

    /// Removes the item at the head of the queue
    fn pop(&mut self) -> Option<T>;

    /// Removes the specified item from the queue. This operation rebalances the queue
    fn remove(&mut self, item: T) -> Option<T>;
}

/// Simple implemenatation of the order queue. Uses a binary heap as a priority queue
/// Orders are prioritized by time and price
pub struct PriceTimePriorityOrderQueue<T> {
    heap: BinaryHeap<T>,
}

impl<T> PriceTimePriorityOrderQueue<T>
where
    T: KeyIndx,
{
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(16),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }
}

impl<T> OrderQueue<T> for PriceTimePriorityOrderQueue<T>
where
    T: KeyIndx,
{
    fn push(&mut self, item: T) {
        self.heap.push(item)
    }

    fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }

    fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    fn remove(&mut self, item: T) -> Option<T> {
        // unfortunately this is the most efficient way to do this using a binary heap
        // rebuilding the binary heap everytime a removal occurs can be costly for large N.
        // For the time being i'll leave this implementation while i research alternative
        // representations
        let mut key_vec = self.heap.to_owned().into_vec();
        key_vec.retain(|k| *k != item);
        self.heap = key_vec.into();
        Some(item)
    }
}

#[cfg(test)]
mod test {
    use crate::core::{
        model::{Order, OrderKey, TradingPair},
        types::{Asset, Long, OrderSide, OrderType, TimestampMillis},
    };
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn can_get_an_inserted_order_back_from_queue() {
        let mut pq: PriceTimePriorityOrderQueue<OrderKey> = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            create_order(
                dec!(200.02),
                OrderSide::Bid,
                4,
                OrderType::Limit,
                TradingPair::from(Asset::BTC, Asset::USDC),
                1678170180000,
            ),
            create_order(
                dec!(300.02),
                OrderSide::Bid,
                10,
                OrderType::Limit,
                TradingPair::from(Asset::DOT, Asset::USDT),
                1680848580000,
            ),
        ];

        orders.iter().for_each(|order| pq.push(order.to_key()));

        let head: OrderKey = *pq.peek().unwrap();
        assert_eq!(
            orders[1].to_key(),
            head,
            "Asserting that the item at the head of the queue is the order with the highest price"
        );
    }

    #[test]
    fn orders_at_the_same_price_are_prioritized_by_time() {
        let mut pq: PriceTimePriorityOrderQueue<OrderKey> = PriceTimePriorityOrderQueue::new();

        let orders = vec![
            create_order(
                dec!(200.02),
                OrderSide::Bid,
                4,
                OrderType::Limit,
                TradingPair::from(Asset::BTC, Asset::USDC),
                1678170180000,
            ),
            create_order(
                dec!(200.02),
                OrderSide::Bid,
                12,
                OrderType::Limit,
                TradingPair::from(Asset::ETH, Asset::USDT),
                1680848580000,
            ),
        ];

        orders.iter().for_each(|order| pq.push(order.to_key()));

        let head: OrderKey = *pq.peek().unwrap();
        assert_eq!(orders[0].to_key(), head, "For orders with the same price, the longest staying order should be at the head of the queue");
    }

    #[test]
    fn orders_can_be_removed_from_queue_if_they_are_canceled() {
        let mut pq: PriceTimePriorityOrderQueue<OrderKey> = PriceTimePriorityOrderQueue::new();

        let order = create_order(
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair::from(Asset::DOT, Asset::USDT),
            1678170180000,
        );

        pq.push(order.to_key());
        assert_eq!(order.to_key(), *pq.peek().unwrap());

        pq.remove(order.to_key());
        assert_eq!(None, pq.peek());
    }

    #[test]
    fn orders_can_be_poped_from_queue_when_needed() {
        let mut pq: PriceTimePriorityOrderQueue<OrderKey> = PriceTimePriorityOrderQueue::new();

        let order = create_order(
            dec!(200.02),
            OrderSide::Bid,
            8,
            OrderType::Limit,
            TradingPair::from(Asset::ETH, Asset::USDC),
            1678170180000,
        );

        pq.push(order.to_key());
        assert_eq!(order.to_key(), pq.pop().unwrap());
    }

    fn create_order(
        price: Decimal,
        side: OrderSide,
        quantity: Long,
        order_type: OrderType,
        trading_pair: TradingPair,
        timestamp: TimestampMillis,
    ) -> Order {
        Order {
            orderid: Uuid::new_v4(),
            price,
            side,
            quantity,
            order_type,
            timestamp,
            trading_pair,
        }
    }
}
