use crate::{event::Event, level::Level};

/// Naive implementation of an orderbook to use as a benchmark
/// This has no use other than benchmarking.
#[derive(Debug, Default, Clone)]
pub struct Orderbook {
    best_bid: Option<Level>,
    best_ask: Option<Level>,
    bids: Vec<Level>,
    asks: Vec<Level>,
    last_updated: u64,
    last_sequence: u64,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            best_bid: None,
            best_ask: None,
            bids: Vec::new(),
            asks: Vec::new(),
            last_updated: 0,
            last_sequence: 0,
        }
    }

    pub fn process(&mut self, event: Event) {
        if event.timestamp < self.last_updated || event.seq < self.last_sequence {
            return;
        }

        match event.is_trade {
            true => self.process_trade(event),
            false => self.process_lvl2(event),
        };

        self.last_updated = event.timestamp;
        self.last_sequence = event.seq;
    }

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

    fn process_lvl2(&mut self, event: Event) {
        match event.is_buy {
            true => {
                if event.size == 0.0 {
                    if let Some(index) = self.bids.iter().position(|x| x.price == event.price) {
                        self.bids.remove(index);
                    };
                } else {
                    if let Some(level) = self.bids.iter_mut().find(|x| x.price == event.price) {
                        *level = Level::from(event);
                    } else {
                        self.bids.push(Level::from(event));
                    }

                    let Some(best_bid) = self.best_bid else {
                        self.best_bid = Some(Level::from(event));
                        return;
                    };

                    if event.price >= best_bid.price {
                        self.best_bid = Some(Level::from(event));
                    }

                    self.bids
                        .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                }
            }
            false => {
                if event.size == 0.0 {
                    if let Some(index) = self.asks.iter().position(|x| x.price == event.price) {
                        self.asks.remove(index);
                    };
                } else {
                    if let Some(level) = self.asks.iter_mut().find(|x| x.price == event.price) {
                        *level = Level::from(event);
                    } else {
                        self.asks.push(Level::from(event));
                    }

                    let Some(best_ask) = self.best_ask else {
                        self.best_ask = Some(Level::from(event));
                        return;
                    };

                    if event.price <= best_ask.price {
                        self.best_ask = Some(Level::from(event));
                    }

                    self.asks
                        .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                }
            }
        }
    }

    fn process_trade(&mut self, event: Event) {
        let buf = match event.is_buy {
            true => &mut self.bids,
            false => &mut self.asks,
        };

        if let Some(index) = buf.iter().position(|x| x.price == event.price) {
            let level = buf.get_mut(index).unwrap();
            if event.size >= level.size {
                buf.remove(index);
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

    pub fn top_n_bids(&self, n: usize) -> Vec<Level> {
        self.bids.iter().rev().take(n).cloned().collect()
    }

    pub fn top_n_asks(&self, n: usize) -> Vec<Level> {
        self.asks.iter().take(n).cloned().collect()
    }

    pub fn midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
            return Some((best_bid.price + best_ask.price) / 2.0);
        }

        None
    }

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
        let mut ob = Orderbook::new();

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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
            ob.top_n_bids(5),
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
        let mut ob = Orderbook::new();

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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
            ob.top_n_asks(5),
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
        let mut ob = Orderbook::new();

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
            ob.top_n_asks(3),
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
            ob.top_n_asks(3),
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
        let mut ob = Orderbook::new();

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
        let mut ob = Orderbook::new();

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
        let mut ob = Orderbook::new();

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
        let mut ob = Orderbook::new();

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
        let mut ob = Orderbook::new();

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
