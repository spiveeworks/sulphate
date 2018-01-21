use std::cmp;

// totally ordered floats, a nice default for the EventQueue
#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Time(f64);

impl Eq for Time {}  // since we prevent NaN

impl Ord for Time {
    fn cmp(self: &Self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect("comparing NaN valued Time")
    }
}

