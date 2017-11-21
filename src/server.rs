
pub struct Server<C, I, T>
    where C: Clock<T>,
          I: Interruption<T>,
          W: Write<T>,
          T: Ord,  // time
{
    game: Game<T>,
    external: mpsc::Receiver<I>,
    clock: C,
    current_time: W,
}

struct Game<T> where T: Ord {
    space: entity_heap::EntityHeap,
    time: event_queue::EventQueue<T>,
}

impl<T> Game<T> where T: Ord {
    fn apply_update<I: Interruption<T>>(self: &mut Self, upd: I) -> bool {
        upd.update(&mut self.space, &mut self.time)
    }
}

impl<C, I, T> Server<C, I, T>
    where C: Clock<T>,
          I: Interruption<T>,
          T: Ord + Clone,  // time
{
    pub fn new(
        space: entity_heap::EntityHeap<T>,
        time: event_queue::EventQueue<T>,
        external: mpsc::Receiver<I>,
        clock: C,
        current_time: W,
    ) {
        let game = Game { space, time };
        Server { game, external, clock, current_time }
    }

    /// runs until told to stop externally
    pub fn run(self: &mut Self) {
        let mut should_exit = false;
        while !should_exit {
            if let Ok(upd) = self.external.try_recv() {
            // first execute any external instructions
                should_exit = self.game.apply_update(upd);
            } else if let Some(next_event) = self.next() {
                let now = time::Instant::now();
                use ClockResult::*;
                match self.clock.now_what(now, next_event) {
                    Simulate { until } => {
            // then execute any internal instruction
                        self.time.simulate_until(until);
                    },
                    Sleep { sleep_for } => {
            // then wait for something to do (whether internal or not)
                        let ext = self.recv_timeout_or_sleep(sleep_for, now);
                        if let Some(upd) = ext {
                            should_exit = self.game.apply_update(upd);
                        }
                    },
                }
            } else if let Ok(upd) = self.external.recv() {
            // if necessary wait forever
                should_exit = self.game.apply_update(upd);
            } else {
            // if the channel closes, and we have nothing to do, exit
                should_exit = true;
            }
        }
    }
}

pub trait Interruption<T> where T: Ord {
    /// returns true if the server should stop
    fn update(
        self: Self,
        space: &mut entity_heap::EntityHeap<T>,
        time: &mut event_queue::EventQueue<T>,
    ) -> bool;
}

pub trait Clock<T> where T: Ord {
    fn now_what(
        self: &mut Self,
        next_event: Option<units::Time>,
    ) -> ClockResult;
}

pub enum ClockResult<T> where T: Ord {
    Simulate { until: T },
    Sleep { until: time::Duration },
    SleepIndefinite,
}

/// Clock for possible use in Server objects,
/// simply progresses to the next available event
pub struct InstantClock;


impl<T> Clock<T> for InstantClock where T: Ord {
    fn now(
        self: &mut Self,
        next_event: Option<T>,
    ) -> ClockResult<T> {
        next_event.map(|then| ClockResult::Simulate(then))
                  .unwrap_or(SleepIndefinite)
    }
}

