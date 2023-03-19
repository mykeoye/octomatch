use rust_decimal::Decimal;

pub type OrderId = u64;
pub type Long = u64;
pub type TimestampMillis = u128;

#[derive(Eq, PartialEq, Copy, Ord, PartialOrd, Clone, Debug)]
pub enum Asset {
    BTC,
    ETH,
    USDT,
    USDC,
    DOT,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Trade {
    pub orderid: OrderId,
    pub side: OrderSide,
    pub price: Decimal,
    pub status: OrderStatus,
    pub quantity: Long,
    pub timestamp: Long,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug, Copy)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug, Copy)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum OrderStatus {
    Created,
    Filled,
    PartialFill,
    Canceled,
    Rejected,
    Expired,
}

#[derive(Debug, PartialEq)]
pub enum Failure<'a> {
    InvalidOrderForBook,
    OrderNotFound(&'a str),
    OrderRejected(&'a str),
    UnsupportedOperation(&'a str),
}
