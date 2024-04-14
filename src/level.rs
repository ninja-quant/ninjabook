use crate::event::Event;
use std::cmp::Ordering;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
pub struct Level {
    pub price: f64,
    pub size: f64,
}

impl Level {
    pub fn new(price: f64, size: f64) -> Self {
        Self { price, size }
    }

    pub fn minimum() -> Self {
        Self {
            price: f64::MIN,
            size: 0.0,
        }
    }

    pub fn maximum() -> Self {
        Self {
            price: f64::MAX,
            size: 0.0,
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price.partial_cmp(&other.price) {
            Some(Ordering::Equal) => self.size.partial_cmp(&other.size),
            other_order => other_order,
        }
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.size == other.size
    }
}

impl Eq for Level {}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} : {})", self.price, self.size)
    }
}

impl From<Event> for Level {
    fn from(value: Event) -> Self {
        Self {
            price: value.price,
            size: value.size,
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn display_level() {
        let level = Level::new(1.1, 2.1);

        assert_eq!(format!("{}", level), "(1.1 : 2.1)")
    }

    #[test]
    fn event_to_level() {
        let event = Event {
            timestamp: 0,
            seq: 0,
            is_trade: false,
            is_buy: false,
            price: 10.0,
            size: 1.0,
        };

        let level = Level::from(event);

        assert_eq!(level.price, 10.0);
        assert_eq!(level.size, 1.0);
    }
}
