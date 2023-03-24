use rust_decimal::Decimal;
use std::{cmp::Ordering, fmt::Debug};

use super::{
    pqueue::Keyable,
    types::{Asset, Failure, Long, OrderId, OrderSide, OrderStatus, OrderType, TimestampMillis},
};

#[derive(PartialEq, Eq, Copy, Ord, PartialOrd, Clone, Debug)]
pub struct Order {
    pub orderid: OrderId,
    pub price: Decimal,
    pub quantity: Long,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub timestamp: TimestampMillis,
    pub trading_pair: TradingPair,
}

impl Order {
    pub fn to_key(&self) -> OrderKey {
        OrderKey {
            orderid: self.orderid,
            price: self.price,
            side: self.side,
            timestamp: self.timestamp,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct TradingPair {
    pub order_asset: Asset,
    pub price_asset: Asset,
}

impl TradingPair {
    pub fn from(order_asset: Asset, price_asset: Asset) -> Self {
        Self {
            order_asset,
            price_asset,
        }
    }
}

impl TradingPair {
    pub fn validate(&self) -> Option<Failure> {
        if self.order_asset == self.price_asset {
            return Some(Failure::OrderRejected(
                "Trading pair must be different for price and order asset type".to_string(),
            ));
        }
        None
    }
}

#[derive(Debug)]
pub struct Event {
    pub status: OrderStatus,
    pub orderid: OrderId,
    pub at_price: String,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            status: OrderStatus::Canceled,
            orderid: Default::default(),
            at_price: Default::default(),
        }
    }
}

#[derive(Clone, Eq, Copy, Debug)]
pub struct OrderKey {
    pub orderid: OrderId,
    pub price: Decimal,
    pub side: OrderSide,
    pub timestamp: TimestampMillis,
}

impl Keyable for OrderKey {}

// The ordering determines how the orders are arranged in the queue. For price time priority
// ordering, we want orders inserted based on the price and the time of entry. For Bids this
// means the highest price gets the top priority, for Asks the lowest price gets the top priority
// For orders with the same price, the longest staying in the queue gets the higher priority
impl Ord for OrderKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price > other.price {
            match self.side {
                OrderSide::Bid => Ordering::Greater,
                OrderSide::Ask => Ordering::Less,
            }
        } else if self.price < other.price {
            match self.side {
                OrderSide::Bid => Ordering::Less,
                OrderSide::Ask => Ordering::Greater,
            }
        } else {
            other.timestamp.cmp(&self.timestamp)
        }
    }
}

impl PartialOrd for OrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderKey {
    fn eq(&self, other: &Self) -> bool {
        self.orderid == other.orderid
            && self.price == other.price
            && self.side == other.side
            && self.timestamp == other.timestamp
    }
}
