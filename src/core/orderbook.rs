use super::{types::{OrderId, Failure}, model::{Order, Spread}};

pub trait OrderBook {
    fn amend(&self, order_id: OrderId, new_order: &Order) -> Result<Order, Failure>;
    fn cancel(&self, order_id: OrderId) -> Result<Order, Failure>;
    fn place(&self, order: &Order) -> Result<Order, Failure>;
    fn get_top(&self) -> Option<(Order, Order)>;
    fn get_spread(&self) -> Option<Spread>;
}