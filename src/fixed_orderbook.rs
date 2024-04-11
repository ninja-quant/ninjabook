use std::{hint::unreachable_unchecked, mem::replace};

use crate::{event::Event, level::Level};

/// Implementation of an orderbook with fixed to use as a benchmark
/// This has no use other than benchmarking.
#[derive(Debug, Clone)]
pub struct Orderbook {
    best_bid: Option<Level>,
    best_ask: Option<Level>,
    bids: Buffer,
    asks: Buffer,
    last_updated: u64,
    last_sequence: u64,
}

impl Default for Orderbook {
    fn default() -> Self {
        Self::new()
    }
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            best_bid: None,
            best_ask: None,
            bids: Buffer::new(),
            asks: Buffer::new(),
            last_updated: 0,
            last_sequence: 0,
        }
    }

    #[inline]
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
        match event.is_buy {
            true => {
                if event.size == 0.0 {
                    if let Ok(to_remove) = self.bids.find_index_bids(event.price) {
                        let removed = self.bids.remove(to_remove);
                        if let Some(best_bid) = self.best_bid {
                            if removed == best_bid.price {
                                self.best_bid = self.bids.first();
                            }
                        }
                    }
                } else {
                    match self.bids.find_index_bids(event.price) {
                        Ok(to_modify) => self.bids.modify(to_modify, event.size),
                        Err(to_insert) => self.bids.insert(to_insert, Level::from(event)),
                    }

                    self.best_bid = self.bids.first();
                }
            }
            false => {
                if event.size == 0.0 {
                    if let Ok(to_remove) = self.asks.find_index_asks(event.price) {
                        let removed = self.asks.remove(to_remove);
                        if let Some(best_ask) = self.best_ask {
                            if removed == best_ask.price {
                                self.best_ask = self.asks.first();
                            }
                        }
                    }
                } else {
                    match self.asks.find_index_asks(event.price) {
                        Ok(to_modify) => self.asks.modify(to_modify, event.size),
                        Err(to_insert) => self.asks.insert(to_insert, Level::from(event)),
                    }
                    self.best_ask = self.asks.first();
                }
            }
        }
    }

    #[inline]
    fn process_trade(&mut self, event: Event) {
        if event.is_buy {
            if let Ok(index) = self.bids.find_index_bids(event.price) {
                let level = self.bids.get_mut(index);
                if event.size >= level.size {
                    self.bids.remove(index);
                } else {
                    level.size -= event.size;
                }
            };
        } else if let Ok(index) = self.asks.find_index_asks(event.price) {
            let level = self.asks.get_mut(index);
            if event.size >= level.size {
                self.asks.remove(index);
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
        let mut result = Vec::with_capacity(n);

        for level in self.bids.buf.iter() {
            if !level.price.is_nan() {
                result.push(*level);
            }
            if result.len() == n {
                break;
            }
        }

        result
    }

    #[inline]
    pub fn top_asks(&self, n: usize) -> Vec<Level> {
        let mut result = Vec::with_capacity(n);

        for level in self.asks.buf.iter() {
            if !level.price.is_nan() {
                result.push(*level);
            }
            if result.len() == n {
                break;
            }
        }

        result
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

#[derive(Debug, Clone)]
pub struct Buffer {
    buf: [Level; 100],
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            buf: [Level::empty(); 100],
        }
    }

    #[inline]
    pub fn find_index_bids(&self, target: f64) -> Result<usize, usize> {
        let mut size = self.buf.len();
        let mut left = 0;
        let mut right = size;

        while left < right {
            let mid = left + size / 4;
            let cur = self.get(mid);

            if target > cur.price || cur.price.is_nan() {
                right = mid;
            } else if target < cur.price {
                left = mid + 1;
            } else {
                if mid >= self.buf.len() {
                    unsafe { unreachable_unchecked() }
                }
                return Ok(mid);
            }

            size = right - left;
        }
        if left > self.buf.len() {
            unsafe { unreachable_unchecked() }
        }
        Err(left)
    }

    #[inline]
    pub fn find_index_asks(&self, target: f64) -> Result<usize, usize> {
        let mut size = self.buf.len();
        let mut left = 0;
        let mut right = size;

        while left < right {
            let mid = left + size / 4;
            let cur = self.get(mid);

            if target < cur.price || cur.price.is_nan() {
                right = mid;
            } else if target > cur.price {
                left = mid + 1;
            } else {
                if mid >= self.buf.len() {
                    unsafe { unreachable_unchecked() }
                }
                return Ok(mid);
            }

            size = right - left;
        }
        if left > self.buf.len() {
            unsafe { unreachable_unchecked() }
        }
        Err(left)
    }

    #[inline]
    fn move_back(&mut self, start: usize) {
        let mut next = start + 1;
        while next < self.buf.len() {
            let lvl = self.get(next);
            if lvl.price.is_nan() {
                break;
            }
            self.buf.swap(next - 1, next);

            next += 1;
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> f64 {
        let level = self.get_mut(index);
        let removed = level.price;
        level.price = f64::NAN;

        self.move_back(index);
        removed
    }

    pub fn first(&self) -> Option<Level> {
        let level = self.get(0);
        if level.price != 0.0 {
            return Some(*level);
        }
        None
    }

    pub fn get(&self, index: usize) -> &Level {
        unsafe { self.buf.get_unchecked(index) }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Level {
        unsafe { self.buf.get_unchecked_mut(index) }
    }

    pub fn insert(&mut self, index: usize, level: Level) {
        let to_replace = self.get_mut(index);
        let mut replaced = replace(to_replace, level);

        let mut next = index + 1;

        while next < self.buf.len() {
            if replaced.price.is_nan() {
                break;
            }

            replaced = replace(self.get_mut(next), replaced);

            next += 1;
        }
    }

    pub fn modify(&mut self, index: usize, size: f64) {
        let level = self.get_mut(index);
        level.size = size;
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
