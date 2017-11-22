use std::sync::mpsc;
use std::thread;
use std::time;

use entity_heap;
use event_queue;

pub struct Server<C, I, T>
    where C: Clock<T>,
          I: Interruption<T>,
          W: Write<T>,
          T: Ord,  // time
{
    game: Game<T>,
    external: mpsc::Receiver<I>,
    clock: C,
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

    fn recv_timeout_or_sleep(
        self: &Self,
        sleep_for: time::Duration,
        now: time::Instant,
    ) -> Option<I> {
        let sleep_until = now + sleep_for;
        let result = self.external.recv_timeout(sleep_for);
        if result.is_none() {
            let sleep_for = sleep_until - time::Instant::now();
            thread::sleep(sleep_for);
        }
        result
    }

    /// runs until told to stop externally
    pub fn run(self: &mut Self) {
        let mut should_exit = false;
        while !should_exit {
            let now = time::Instant::now();
            let in_game = self.clock.in_game(now);
            let 
            if let Ok(upd) = self.external.try_recv() {
                // first execute any external instructions
                should_exit = self.game.apply_update(upd);
            } else {
                // then execute any internal instructions
                self.time.simulate_until(in_game);
                self.clock.finished_cycle(now);
                if let Some(et) = self.time.next() {
                    // then wait for more instructions
                    let sleep_for = self.clock.minimum_wait(now, et);
                    let ext = self.recv_timeout_or_sleep(sleep_for, now);
                    if let Some(upd) = ext {
                        should_exit = self.game.apply_update(upd);
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
    /// convert a real time to an in-game time
    fn in_game(
        self: &mut Self,
        now: time::Instant,
    ) -> T;
    /// convert an in-game time to a possible real time
    /// (it is not an error for the clock to run slower than it claims)
    fn minimum_wait(
        self: &mut Self,
        in_game: T,
    ) -> time::Duration;
    /// used to report that a device (e.g. the event queue) has finished
    /// a cycle.
    /// Use this to slow the clock when threads are lagging.
    /// (e.g. don't let clock exceed any threads by one 16th of a second)
    fn finished_cycle(
        self: &mut Self,
        now: time::Instant,
    );
}

