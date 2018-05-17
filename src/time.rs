use std::cmp;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Time(f64);

impl Time {
    pub fn try_from(val: f64) -> Option<Self> {
        if val.is_finite() {
            Some(Time(val))
        } else {
            None
        }
    }

    pub fn as_float(self: Self) -> f64 {
        self.0
    }
}

impl Eq for Time {
}

impl Ord for Time {
    fn cmp(self: &Self, other: &Self) -> cmp::Ordering {
        // TODO use unsafe unreachable
        self.partial_cmp(other).expect("Unexpected NaN")
    }
}

