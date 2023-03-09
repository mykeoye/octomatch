pub type OrderId = u64;
pub type Long = u64;

#[derive(Eq, PartialEq, Copy, Ord, PartialOrd, Clone, Debug)]
pub enum Asset {
    BTC,
    ETH,
    USDT,
    USDC,
    DOT,
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

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum OrderStatus {
    Created,
    Filled,
    PartialFilled,
    Canceled,
    Rejected,
    Expired,
}

#[derive(Debug, PartialEq)]
pub enum Failure<'a> {
    OrderNotFound(&'a str),
    OrderRejected(&'a str),
}
