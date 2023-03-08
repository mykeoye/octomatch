
pub type OrderId = u64;
pub type Long = u64;

#[derive(Eq, PartialEq, Clone, Debug)]
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

#[derive(Debug)]
pub enum Failure {
    OrderNotFound(String)
}