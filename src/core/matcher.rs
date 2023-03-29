use std::fmt::Debug;

use super::{
    model::Order,
    orderbook::OrderBook,
    types::{Long, OrderSide, OrderStatus, OrderType, Trade},
};

/// A match is a structure which contains a list of executed orders (trades) as well as fields
/// indicating if the match was done in full or partially, along with the quantity left
#[derive(Debug)]
pub struct Match<T> {
    /// list of matches found by the matcher
    matches: Vec<T>,

    /// the state of the match run, can be partial, full or no-match
    state: MatchState,

    /// number of items left to complete a full match
    qty_left: Long,
}

impl<T> Match<T>
where
    T: Clone + Debug + Copy,
{
    pub fn new() -> Self {
        Self {
            matches: Vec::with_capacity(4),
            state: MatchState::NoMatch,
            qty_left: 0,
        }
    }

    pub fn add_match(&mut self, trade: T) {
        self.matches.push(trade)
    }

    pub fn get_matches(&self) -> Vec<T> {
        self.matches.clone()
    }

    pub fn update_state(&mut self, state: MatchState) {
        match state {
            MatchState::Full | MatchState::NoMatch => self.update_qty_left(0),
            MatchState::Partial => (),
        }
        self.state = state
    }

    pub fn get_state(&self) -> MatchState {
        self.state.clone()
    }

    pub fn update_qty_left(&mut self, qty: Long) {
        self.qty_left = qty
    }

    pub fn get_qty_left(&self) -> Long {
        self.qty_left
    }

    pub fn is_partial(&self) -> bool {
        self.state == MatchState::Partial
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MatchState {
    Full,
    Partial,
    NoMatch,
}
/// Implements a matcher with takes an order and its respective book and attempts to find a set
/// of matching trades (bids to asks and vice-versa)
#[derive(Debug)]
pub struct Matcher;

impl Matcher {
    pub fn match_order<T: OrderBook>(&self, order: Order, orderbook: &mut T) -> Match<Trade> {
        let mut matches = Match::new();
        match order.order_type {
            // a market order is matched immediately at the best available price. In cases
            // where the engine is unable to fill the match completely, the order is partially
            // filled and the remaining part of the order is left on the book
            OrderType::Market => {
                if let Some(opp_order) = Self::get_opposite_order(order.side, orderbook) {
                    Self::do_match(order, opp_order.clone(), orderbook, &mut matches)
                }
                // an early return with the state being MatchState::NoMatch
                return matches;
            }
            // a limit order is first matched immediately if possible and if not it is placed into
            // the limit order book to be filled at a later time, when a matching market order is found
            OrderType::Limit => {
                if let Some(opp_order) = Self::get_opposite_order(order.side, orderbook) {
                    // first we do price check to ensure the price variant of the limit order is maintained
                    if Self::is_within_price_limit(order, *opp_order) {
                        Self::do_match(order, opp_order.clone(), orderbook, &mut matches);
                        // if there's a partial match we want to place the remnants on the orderbook
                        if MatchState::Partial == matches.get_state() {
                            let mut left_over = order.clone();
                            left_over.quantity = matches.get_qty_left();
                            let _ = orderbook.place(left_over);
                        }
                        return matches;
                    }
                }
                let _ = orderbook.place(order);
                // an early return with the state being MatchState::NoMatch
                return matches;
            }
            OrderType::Stop => todo!(),
        }
    }

    fn get_opposite_order(side: OrderSide, orderbook: &mut dyn OrderBook) -> Option<&Order> {
        match side {
            OrderSide::Bid => orderbook.peek_top_ask(),
            OrderSide::Ask => orderbook.peek_top_bid(),
        }
    }

    fn is_within_price_limit(order: Order, opp_order: Order) -> bool {
        match order.side {
            OrderSide::Bid => order.price >= opp_order.price,
            OrderSide::Ask => order.price <= opp_order.price,
        }
    }

    fn do_match(
        mut incoming_order: Order,
        opposite_order: Order,
        orderbook: &mut dyn OrderBook,
        matches: &mut Match<Trade>,
    ) {
        if incoming_order.quantity < opposite_order.quantity {
            matches.add_match(Trade {
                orderid: incoming_order.orderid,
                side: incoming_order.side,
                price: opposite_order.price,
                status: OrderStatus::Filled,
                quantity: incoming_order.quantity,
                timestamp: 0,
            });

            matches.add_match(Trade {
                orderid: opposite_order.orderid,
                side: opposite_order.side,
                price: opposite_order.price,
                status: OrderStatus::PartialFill,
                quantity: incoming_order.quantity,
                timestamp: 0,
            });

            orderbook.modify_quantity(
                opposite_order.orderid,
                opposite_order.quantity - incoming_order.quantity,
            );
            // the state is full because the engine was able to fully match the incoming order
            matches.update_state(MatchState::Full);
        } else if incoming_order.quantity > opposite_order.quantity {
            matches.add_match(Trade {
                orderid: incoming_order.orderid,
                side: incoming_order.side,
                price: opposite_order.price,
                status: OrderStatus::PartialFill,
                quantity: opposite_order.quantity,
                timestamp: 0,
            });

            matches.add_match(Trade {
                orderid: opposite_order.orderid,
                side: opposite_order.side,
                price: opposite_order.price,
                status: OrderStatus::Filled,
                quantity: opposite_order.quantity,
                timestamp: 0,
            });

            // update the quantity of the partially filled order
            incoming_order.quantity -= opposite_order.quantity;

            // we update the quantity left to match for the primary order
            matches.update_qty_left(incoming_order.quantity);

            // since the incoming order was partially filled, the state is updated accordingly
            matches.update_state(MatchState::Partial);

            let some_order = match incoming_order.side {
                OrderSide::Bid => {
                    // pop off the current top ask, since it has already been filled
                    orderbook.pop_top_ask();
                    // get the current top ask on the book
                    orderbook.peek_top_ask()
                }
                OrderSide::Ask => {
                    // pop the current top bid since it has been filled
                    orderbook.pop_top_bid();
                    // get the current top bid and attempt to fill
                    orderbook.peek_top_bid()
                }
            };

            // attempt to fill the rest of the partially filled order
            if let Some(opposite) = some_order {
                Self::do_match(incoming_order, opposite.clone(), orderbook, matches)
            }
        } else {
            matches.add_match(Trade {
                orderid: incoming_order.orderid,
                side: incoming_order.side,
                price: opposite_order.price,
                status: OrderStatus::Filled,
                quantity: incoming_order.quantity,
                timestamp: 0,
            });

            matches.add_match(Trade {
                orderid: opposite_order.orderid,
                side: opposite_order.side,
                price: opposite_order.price,
                status: OrderStatus::Filled,
                quantity: opposite_order.quantity,
                timestamp: 0,
            });

            matches.update_state(MatchState::Full);

            match incoming_order.side {
                OrderSide::Bid => orderbook.pop_top_ask(),
                OrderSide::Ask => orderbook.pop_top_bid(),
            };
        }
    }
}

#[cfg(test)]
mod test {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use crate::core::{
        model::TradingPair,
        orderbook::LimitOrderBook,
        types::{Asset, Long},
        utils::Util,
    };

    use super::*;

    #[test]
    fn an_empty_orderbook_should_have_no_executed_trades() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::ETH, Asset::USDC));

        let matcher = Matcher {};
        let order = create_order(OrderSide::Ask, dec!(2.22), OrderType::Market, 100);
        let matches = matcher.match_order(order, &mut orderbook);
        assert_eq!(matches.get_state(), MatchState::NoMatch);
    }

    #[test]
    fn a_market_bid_is_fully_matched_in_a_non_empty_orderbook_with_matching_asks() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::ETH, Asset::USDC));

        let asks = create_orders(OrderSide::Ask);
        for ask in &asks {
            let _ = orderbook.place(ask.clone());
        }

        let matcher = Matcher {};
        let bid = create_order(OrderSide::Bid, dec!(100.00), OrderType::Market, 100);
        let matches = matcher.match_order(bid, &mut orderbook);

        let trades = matches.get_matches();
        assert!(!trades.is_empty());
        assert_eq!(trades.len(), 4);
        assert_eq!(matches.get_state(), MatchState::Full);
        assert_eq!(matches.get_qty_left(), 0);

        let trade1 = &trades[0];
        assert_eq!(trade1.orderid, bid.orderid);
        assert_eq!(trade1.quantity, 50);
        assert_eq!(trade1.price, dec!(40.00));
        assert_eq!(trade1.status, OrderStatus::PartialFill);

        let trade2 = &trades[1];
        let ask1 = &asks[1];
        assert_eq!(trade2.orderid, ask1.orderid);
        assert_eq!(trade2.quantity, ask1.quantity);
        assert_eq!(trade2.price, ask1.price);
        assert_eq!(trade2.status, OrderStatus::Filled);

        let trade3 = &trades[2];
        assert_eq!(trade3.orderid, bid.orderid);
        assert_eq!(trade3.quantity, bid.quantity - trade1.quantity);
        assert_eq!(trade3.price, bid.price);
        assert_eq!(trade3.status, OrderStatus::Filled);

        let trade4 = &trades[3];
        let ask2 = &asks[0];
        assert_eq!(trade4.orderid, ask2.orderid);
        assert_eq!(trade4.quantity, 50);
        assert_eq!(trade4.price, ask2.price);
        assert_eq!(trade4.status, OrderStatus::PartialFill);
    }

    #[test]
    fn a_limit_order_is_partially_matched_if_price_limits_are_met_with_low_volume() {
        let mut orderbook = LimitOrderBook::init(TradingPair::from(Asset::ETH, Asset::USDC));

        let bids = create_orders(OrderSide::Bid);
        for bid in &bids {
            let _ = orderbook.place(bid.clone());
        }

        let matcher = Matcher {};
        let order = create_order(OrderSide::Ask, dec!(5.00), OrderType::Limit, 1000);
        let matches = matcher.match_order(order, &mut orderbook);
        assert_eq!(matches.get_state(), MatchState::Partial);
        assert_eq!(matches.get_qty_left(), 800);

        let trades = matches.get_matches();
        assert_eq!(trades.len(), 6);

        assert_eq!(order.orderid, trades[0].orderid);
        assert_eq!(trades[0].quantity, 50);

        assert!(orderbook.peek_top_bid().is_none());
        let top_ask = orderbook.peek_top_ask();
        assert!(top_ask.is_some());

        let ask = top_ask.unwrap();
        assert_eq!(ask.orderid, order.orderid);
        assert_eq!(ask.quantity, matches.get_qty_left());
    }

    fn create_order(
        side: OrderSide,
        price: Decimal,
        order_type: OrderType,
        quantity: Long,
    ) -> Order {
        Order {
            orderid: Uuid::new_v4(),
            price,
            side,
            quantity,
            order_type,
            timestamp: Util::current_time_millis(),
            trading_pair: TradingPair::from(Asset::ETH, Asset::USDC),
        }
    }

    fn create_orders(side: OrderSide) -> Vec<Order> {
        vec![
            create_order(side, dec!(100.00), OrderType::Limit, 100),
            create_order(side, dec!(40.00), OrderType::Limit, 50),
            create_order(side, dec!(550.00), OrderType::Limit, 50),
        ]
    }
}
