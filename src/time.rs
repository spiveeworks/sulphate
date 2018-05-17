use std::cmp;
use std::ops;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Time(f64);

impl Time {
    pub fn try_from(val: f64) -> Option<Self> {
        if !val.is_nan() {
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

impl ops::Add for Time {
    type Output = Time;
    fn add(self: Time, other: Time) -> Time {
        Time(self.0 + other.0)
    }
}

impl ops::Sub for Time {
    type Output = Time;
    fn sub(self: Time, other: Time) -> Time {
        Time(self.0 - other.0)
    }
}

impl ops::AddAssign for Time {
    fn add_assign(self: &mut Time, other: Time) {
        self.0 += other.0;
    }
}

impl ops::SubAssign for Time {
    fn sub_assign(self: &mut Time, other: Time) {
        self.0 -= other.0;
    }
}
