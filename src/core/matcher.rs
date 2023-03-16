use super::{
    model::Order,
    orderbook::OrderBook,
    types::{OrderSide, OrderStatus, OrderType, Trade},
};

pub struct Matcher {}

impl Matcher {
    pub fn match_order(&self, order: Order, orderbook: &mut dyn OrderBook) -> Vec<Trade> {
        let mut trades: Vec<Trade> = Vec::with_capacity(4);
        match order.order_type {
            OrderType::Market => match order.side {
                OrderSide::Bid => match orderbook.peek_top_ask() {
                    Some(ask) => Self::do_match(order, ask.clone(), orderbook, &mut trades),
                    None => {}
                },
                OrderSide::Ask => match orderbook.peek_top_bid() {
                    Some(bid) => Self::do_match(order, bid.clone(), orderbook, &mut trades),
                    None => {}
                },
            },
            OrderType::Limit => match orderbook.place(order) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            },
            OrderType::Stop => todo!(),
        }
        trades
    }

    fn do_match(
        mut left: Order,
        right: Order,
        orderbook: &mut dyn OrderBook,
        trades: &mut Vec<Trade>,
    ) {
        if left.quantity < right.quantity {
            trades.push(Trade {
                orderid: left.orderid,
                side: left.side,
                price: left.price,
                status: OrderStatus::Filled,
                quantity: left.quantity,
                timestamp: 0,
            });

            trades.push(Trade {
                orderid: right.orderid,
                side: right.side,
                price: right.price,
                status: OrderStatus::PartialFill,
                quantity: left.quantity,
                timestamp: 0,
            });
            orderbook.modify_quantity(right.orderid, right.quantity - left.quantity);
        } else if left.quantity > right.quantity {
            trades.push(Trade {
                orderid: left.orderid,
                side: left.side,
                price: left.price,
                status: OrderStatus::PartialFill,
                quantity: left.quantity - right.quantity,
                timestamp: 0,
            });

            trades.push(Trade {
                orderid: right.orderid,
                side: right.side,
                price: right.price,
                status: OrderStatus::Filled,
                quantity: right.quantity,
                timestamp: 0,
            });

            // update the quantity of the partially filled order
            left.quantity -= right.quantity;

            // attempt to fill the rest of the partially filled order
            let right = match left.side {
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
            }
            .unwrap()
            .clone();
            Self::do_match(left, right, orderbook, trades)
        } else {
            // both orders can be fully filled
            trades.push(Trade {
                orderid: left.orderid,
                side: left.side,
                price: right.price,
                status: OrderStatus::Filled,
                quantity: left.quantity,
                timestamp: 0,
            });

            trades.push(Trade {
                orderid: right.orderid,
                side: right.side,
                price: right.price,
                status: OrderStatus::Filled,
                quantity: right.quantity,
                timestamp: 0,
            });

            match left.side {
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

    use crate::core::{
        model::TradingPair,
        orderbook::LimitOrderBook,
        types::{Asset, Long, OrderId},
    };

    use super::*;

    #[test]
    fn an_empty_orderbook_should_have_no_executed_trades() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::ETH,
            price_asset: Asset::USDC,
        });

        let matcher = Matcher {};
        let order = create_order(11, OrderSide::Ask, dec!(2.22), OrderType::Limit, 100);
        let matches = matcher.match_order(order, &mut orderbook);
        assert!(matches.is_empty());
    }

    #[test]
    fn a_market_bid_is_fully_matched_in_a_non_empty_orderbook_with_matching_asks() {
        let mut orderbook = LimitOrderBook::init(TradingPair {
            order_asset: Asset::ETH,
            price_asset: Asset::USDC,
        });

        let asks = vec![
            create_order(12, OrderSide::Ask, dec!(100.00), OrderType::Limit, 100),
            create_order(13, OrderSide::Ask, dec!(40.00), OrderType::Limit, 50),
            create_order(15, OrderSide::Ask, dec!(550.00), OrderType::Limit, 50),
        ];

        for ask in &asks {
            let _ = orderbook.place(ask.clone());
        }

        let matcher = Matcher {};
        let bid = create_order(14, OrderSide::Bid, dec!(100.00), OrderType::Market, 100);
        let matches = matcher.match_order(bid, &mut orderbook);
        assert!(!matches.is_empty());
        assert_eq!(matches.len(), 4);

        dbg!(&matches);

        let trade1 = &matches[0];
        assert_eq!(trade1.orderid, bid.orderid);
        assert_eq!(trade1.quantity, 50);
        assert_eq!(trade1.price, bid.price);
        assert_eq!(trade1.status, OrderStatus::PartialFill);

        let trade2 = &matches[1];
        let ask1 = &asks[1];
        assert_eq!(trade2.orderid, ask1.orderid);
        assert_eq!(trade2.quantity, ask1.quantity);
        assert_eq!(trade2.price, ask1.price);
        assert_eq!(trade2.status, OrderStatus::Filled);

        let trade3 = &matches[2];
        assert_eq!(trade3.orderid, bid.orderid);
        assert_eq!(trade3.quantity, bid.quantity - trade1.quantity);
        assert_eq!(trade3.price, bid.price);
        assert_eq!(trade3.status, OrderStatus::Filled);

        let trade4 = &matches[3];
        let ask2 = &asks[0];
        assert_eq!(trade4.orderid, ask2.orderid);
        assert_eq!(trade4.quantity, 50);
        assert_eq!(trade4.price, ask2.price);
        assert_eq!(trade4.status, OrderStatus::PartialFill);
    }

    fn create_order(
        orderid: OrderId,
        side: OrderSide,
        price: Decimal,
        order_type: OrderType,
        quantity: Long,
    ) -> Order {
        Order {
            orderid,
            price,
            side,
            quantity,
            order_type,
            timestamp: 1678170180000,
            trading_pair: TradingPair {
                order_asset: Asset::ETH,
                price_asset: Asset::USDC,
            },
        }
    }
}
