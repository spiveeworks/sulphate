use std::ops;
use std::time;

use server;
use Time;

#[derive(Clone)]
pub struct Simple<T=Time> {
    start_instant: Option<time::Instant>,
    last_time: T,
}

type Diff<T> = <T as ops::Sub>::Output;

impl<T> Simple<T>
    where T: Clone + ops::Sub + ops::Add<Diff<T>, Output=T>,
          Diff<T>: From<time::Duration>,
{
    pub fn new(start_time: T) -> Self {
        Simple {
            start_instant: None,
            last_time: start_time,
        }
    }

    pub fn elapsed_as_of(self: &Self, now: time::Instant) -> time::Duration {
        if let Some(start) = self.start_instant {
            now.duration_since(start)
        } else {
            // time only passes if the clock has started
            time::Duration::new(0,0)
        }
    }

    pub fn time(self: &Self, now: time::Instant) -> T {
        let elapsed: Diff<T> = self.elapsed_as_of(now).into();
        self.last_time.clone() + elapsed
    }

    pub fn stop(self: &mut Self, now: time::Instant) {
        self.last_time = self.time(now);
        self.start_instant = None;
    }

    pub fn start(self: &mut Self, now: time::Instant) {
        self.stop(now);
        self.start_instant = Some(now);
    }
}

impl<T> server::Clock<T> for Simple<T>
    where T: Clone + Ord + ops::Sub + ops::Add<Diff<T>, Output=T>,
          time::Duration: From<Diff<T>>,
          Diff<T>: From<time::Duration>,
{
    fn in_game(self: &mut Self, now: time::Instant) -> T {
        self.time(now)
    }
    fn minimum_wait(
        self: &mut Self,
        now: T,
        until: T,
    ) -> time::Duration {
        (until - now).into()
    }
    fn finished_cycle(
        self: &mut Self,
        _now: time::Instant,
        _in_game: T,
    ) {}
    fn end_cycles(self: &mut Self) {}
}
