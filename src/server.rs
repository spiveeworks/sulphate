use std::sync::mpsc;
use std::thread;
use std::time;

use entity_heap;
use event_queue;

pub struct Server<C, I, T>
    where C: Clock<T>,
          I: Interruption<T>,
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
    // note this returns the result of the update, not of .progress_time()
    fn apply_update<I: Interruption<T>>(
        self: &mut Self,
        upd: I,
        in_game: T
    ) -> bool
        where T: Clone
    {
        self.time.progress_time(in_game);
        upd.update(&mut self.space, &mut self.time)
    }

    fn invoke_next(self: &mut Self) {
        self.time.invoke_next(&mut self.space);
    }
}

impl<C, I, T> Server<C, I, T>
    where C: Clock<T>,
          I: Interruption<T>,
          T: Ord + Clone,  // time
{
    pub fn new(
        space: entity_heap::EntityHeap,
        time: event_queue::EventQueue<T>,
        external: mpsc::Receiver<I>,
        clock: C,
    ) -> Self {
        let game = Game { space, time };
        Server { game, external, clock }
    }

    fn recv_timeout_or_sleep(
        self: &Self,
        sleep_for: time::Duration,
        now: time::Instant,
    ) -> Option<I> {
        let sleep_until = now + sleep_for;
        let result = self.external.recv_timeout(sleep_for);
        if result.is_err() {
            let sleep_for = sleep_until - time::Instant::now();
            thread::sleep(sleep_for);
        }
        result.ok()
    }

    /// runs until told to stop externally
    pub fn run(self: &mut Self) {
        let mut should_exit = false;
        while !should_exit {
            let now = time::Instant::now();
            let in_game = self.clock.in_game(now);
            if let Ok(upd) = self.external.try_recv() {
                self.clock.finished_cycle(now, in_game.clone());
                // first execute any external instructions
                should_exit = self.game.apply_update(upd, in_game);
            } else {
                if let Some(et) = self.game.time.next() {
                    if et <= in_game {
                        self.clock.finished_cycle(now, et);
                        // then execute any internal instructions
                        self.game.invoke_next();
                    } else {
                        self.clock.end_cycles();
                        // then wait for more instructions
                        let sleep_for = self.clock
                                            .minimum_wait(in_game, et.clone());
                        let ext = self.recv_timeout_or_sleep(sleep_for, now);
                        if let Some(upd) = ext {
                            should_exit = self.game.apply_update(upd, et);
                        }
                    }
                } else if let Ok(upd) = self.external.recv() {
                    self.clock.end_cycles();
                    // if necessary wait forever
                    let now = time::Instant::now();
                    let in_game = self.clock.in_game(now);
                    should_exit = self.game.apply_update(upd, in_game);
                } else {
                    // if the channel closes, and we have nothing to do, exit
                    should_exit = true;
                }
            }
        }
        self.clock.end_cycles();
    }
}

pub trait Interruption<T> where T: Ord {
    /// returns true if the server should stop
    fn update(
        self: Self,
        space: &mut entity_heap::EntityHeap,
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
        until: T,
    ) -> time::Duration;
    /// used to report that a device (e.g. the event queue) has finished
    /// a cycle.
    /// Use this to slow the clock when threads are lagging.
    /// (e.g. don't let clock exceed any threads by one 16th of a second)
    fn finished_cycle(
        self: &mut Self,
        now: time::Instant,
        in_game: T,
    );
    /// used to report that a device has no work to do and does not need
    /// the clock to stutter for it
    fn end_cycles(
        self: &mut Self
    );
}

