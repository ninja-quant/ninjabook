use std::fmt::Display;

use crate::event::Event;

#[derive(Debug, Clone, Copy, Default)]
pub struct Level {
    pub price: f64,
    pub size: f64,
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
        let level = Level {
            price: 1.1,
            size: 2.1,
        };

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
