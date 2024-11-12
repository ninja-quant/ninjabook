use crate::{event::Event, level::Level};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct Orderbook {
    best_bid: Option<Level>,
    best_ask: Option<Level>,
    bids: BTreeMap<u64, Level>,
    asks: BTreeMap<u64, Level>,
    last_updated: u64,
    last_sequence: u64,
    inv_tick_size: f64,
}

impl Orderbook {
    pub fn new(tick_size: f64) -> Self {
        Self {
            best_bid: None,
            best_ask: None,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_updated: 0,
            last_sequence: 0,
            inv_tick_size: 1.0 / tick_size,
        }
    }

    #[inline]
    pub fn process_raw(
        &mut self,
        timestamp: u64,
        seq: u64,
        is_trade: bool,
        is_buy: bool,
        price: f64,
        size: f64,
    ) {
        let event = Event {
            timestamp,
            seq,
            is_trade,
            is_buy,
            price,
            size,
        };

        self.process(event);
    }

    #[inline]
    pub fn process_stream_bbo_raw(
        &mut self,
        timestamp: u64,
        seq: u64,
        is_trade: bool,
        is_buy: bool,
        price: f64,
        size: f64,
    ) -> Option<(Option<Level>, Option<Level>)> {
        let event = Event {
            timestamp,
            seq,
            is_trade,
            is_buy,
            price,
            size,
        };

        self.process_stream_bbo(event)
    }

    #[inline]
    pub fn process(&mut self, event: Event) {
        if event.timestamp < self.last_updated && event.seq < self.last_sequence {
            return;
        }

        match event.is_trade {
            true => self.process_trade(event),
            false => self.process_lvl2(event),
        };

        self.last_updated = event.timestamp;
        self.last_sequence = event.seq;
    }

    #[inline]
    pub fn process_stream_bbo(&mut self, event: Event) -> Option<(Option<Level>, Option<Level>)> {
        let old_bid = self.best_bid;
        let old_ask = self.best_ask;

        self.process(event);

        let new_bid = self.best_bid;
        let new_ask = self.best_ask;

        if old_bid != new_bid || old_ask != new_ask {
            Some((new_bid, new_ask))
        } else {
            None
        }
    }

    #[inline]
    fn process_lvl2(&mut self, event: Event) {
        let price_ticks = event.price_ticks(self.inv_tick_size);
        match event.is_buy {
            true => {
                if event.size == 0.0 {
                    if let Some(removed) = self.bids.remove(&price_ticks) {
                        if let Some(best_bid) = self.best_bid {
                            if removed.price == best_bid.price {
                                self.best_bid = self.bids.values().next_back().cloned();
                            }
                        };
                    }
                } else {
                    self.bids
                        .entry(price_ticks)
                        .and_modify(|e| e.size = event.size)
                        .or_insert(Level::from(event));

                    let Some(best_bid) = self.best_bid else {
                        self.best_bid = Some(Level::from(event));
                        return;
                    };

                    if event.price >= best_bid.price {
                        self.best_bid = Some(Level::from(event));
                    }
                }
            }
            false => {
                if event.size == 0.0 {
                    if let Some(removed) = self.asks.remove(&price_ticks) {
                        if let Some(best_ask) = self.best_ask {
                            if removed.price == best_ask.price {
                                self.best_ask = self.asks.values().next().cloned();
                            }
                        };
                    }
                } else {
                    self.asks
                        .entry(price_ticks)
                        .and_modify(|e| e.size = event.size)
                        .or_insert(Level::from(event));

                    let Some(best_ask) = self.best_ask else {
                        self.best_ask = Some(Level::from(event));
                        return;
                    };

                    if event.price <= best_ask.price {
                        self.best_ask = Some(Level::from(event));
                    }
                }
            }
        }
    }

    #[inline]
    fn process_trade(&mut self, event: Event) {
        let buf = match event.is_buy {
            true => &mut self.bids,
            false => &mut self.asks,
        };

        let price_ticks = event.price_ticks(self.inv_tick_size);

        if let Some(level) = buf.get_mut(&price_ticks) {
            if event.size >= level.size {
                buf.remove(&price_ticks);
            } else {
                level.size -= event.size;
            }
        };
    }

    pub fn best_bid(&self) -> Option<Level> {
        self.best_bid
    }

    pub fn best_ask(&self) -> Option<Level> {
        self.best_ask
    }

    #[inline]
    pub fn top_bids(&self, n: usize) -> Vec<Level> {
        self.bids.values().rev().take(n).cloned().collect()
    }

    #[inline]
    pub fn top_asks(&self, n: usize) -> Vec<Level> {
        self.asks.values().take(n).cloned().collect()
    }

    #[inline]
    pub fn midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
            return Some((best_bid.price + best_ask.price) / 2.0);
        }

        None
    }

    #[inline]
    pub fn weighted_midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
            let num = best_bid.size * best_ask.price + best_bid.price * best_ask.size;
            let den = best_bid.size + best_ask.size;
            return Some(num / den);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_lvl2_bids() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 16.0,
            size: 1.0,
        };

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level {
                price: 16.0,
                size: 1.0
            },]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 7.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 10.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 8.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 8.0,
            size: 2.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 2.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 12.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 12.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 2.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 21.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 21.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 12.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 2.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 8.0,
            size: 0.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 21.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 12.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 8.0,
            size: 10.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 21.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 12.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 10.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 21.0,
            size: 0.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 12.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 10.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
            ]
        );
    }

    #[test]
    fn process_lvl2_asks() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 16.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [Level {
                price: 16.0,
                size: 1.0
            },]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 7.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 10.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 6.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 6.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 8.0,
            size: 2.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 6.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 2.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 50.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 6.0,
                    size: 1.0
                },
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 2.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 6.0,
            size: 0.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 8.0,
                    size: 2.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 50.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 8.0,
            size: 0.0,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(5),
            [
                Level {
                    price: 7.0,
                    size: 1.0
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 16.0,
                    size: 1.0
                },
                Level {
                    price: 50.0,
                    size: 1.0
                },
            ]
        );
    }

    #[test]
    fn process_all_asks() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 11.0,
            size: 1.0,
        };
        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 10.0,
            size: 1.0,
        };
        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 9.0,
            size: 1.0,
        };
        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: true,
            is_buy: false,
            price: 9.0,
            size: 0.5,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(3),
            [
                Level {
                    price: 9.0,
                    size: 0.5
                },
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 11.0,
                    size: 1.0
                },
            ]
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: true,
            is_buy: false,
            price: 9.0,
            size: 0.9,
        };
        ob.process(event);

        assert_eq!(
            ob.top_asks(3),
            [
                Level {
                    price: 10.0,
                    size: 1.0
                },
                Level {
                    price: 11.0,
                    size: 1.0
                },
            ]
        );
    }

    #[test]
    fn old_event() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 11.0,
            size: 1.0,
        };
        ob.process(event);

        let event = Event {
            timestamp: 1,
            seq: 1,
            is_trade: false,
            is_buy: false,
            price: 10.0,
            size: 1.0,
        };
        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 9.0,
            size: 1.0,
        };
        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: true,
            is_buy: false,
            price: 8.0,
            size: 1.0,
        };
        ob.process(event);

        assert_eq!(
            ob.best_ask.unwrap(),
            Level {
                price: 10.0,
                size: 1.0
            }
        )
    }

    #[test]
    fn process_stream_bbo() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 16.0,
            size: 1.0,
        };

        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 20.0,
            size: 1.0,
        };

        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 22.0,
            size: 1.0,
        };

        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 21.0,
            size: 1.0,
        };

        let (best_bid, best_ask) = ob.process_stream_bbo(event).unwrap();

        assert_eq!(
            best_bid.unwrap(),
            Level {
                price: 20.0,
                size: 1.0
            }
        );

        assert_eq!(
            best_ask.unwrap(),
            Level {
                price: 21.0,
                size: 1.0
            }
        );

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 23.0,
            size: 1.0,
        };

        assert_eq!(ob.process_stream_bbo(event), None);
    }

    #[test]
    fn remove_non_existing_level_with_trade() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 16.0,
            size: 1.0,
        };

        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 20.0,
            size: 1.0,
        };

        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: true,
            is_buy: true,
            price: 12.0,
            size: 1.0,
        };

        ob.process(event);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: true,
            is_buy: false,
            price: 22.0,
            size: 1.0,
        };

        ob.process(event);

        assert_eq!(
            ob.best_bid().unwrap(),
            Level {
                price: 16.0,
                size: 1.0
            }
        );

        assert_eq!(
            ob.best_ask().unwrap(),
            Level {
                price: 20.0,
                size: 1.0
            }
        );
    }

    #[test]
    fn midprice() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 16.0,
            size: 1.0,
        };

        ob.process(event);

        assert_eq!(ob.midprice(), None);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 20.0,
            size: 1.0,
        };

        ob.process(event);

        let midprice = ob.midprice().unwrap();

        assert_eq!(midprice, 18.0)
    }

    #[test]
    fn weighted_midprice() {
        let mut ob = Orderbook::new(0.01);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: true,
            price: 16.0,
            size: 1.0,
        };

        ob.process(event);

        assert_eq!(ob.weighted_midprice(), None);

        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 20.0,
            size: 4.0,
        };

        ob.process(event);

        let weighted_midprice = ob.weighted_midprice().unwrap();

        assert_eq!(weighted_midprice, 16.8)
    }
}
