use std::sync::mpsc;
use std::thread;
use std::time;

use event_queue;

pub struct Server<C, I, G> {
    clock: C,
    external: mpsc::Receiver<I>,
    game: G,
}

// note this returns the result of the update, not of .progress_time()
fn apply_update<I, G, E, T>(
    game: &mut G,
    upd: I,
    in_game: T
) -> bool

    where I: Interruption<G>,
          G: event_queue::Simulation<E, T>,
          E: event_queue::GeneralEvent<G>,
          T: Ord + Clone,
{
    game.as_mut().progress_time(in_game);
    upd.update(game)
}

impl<C, I, G> Server<C, I, G>
    where I: Interruption<G>
{
    pub fn new(
        game: G,
        external: mpsc::Receiver<I>,
        clock: C,
    ) -> Self {
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
            let now = time::Instant::now();
            if sleep_until > now {
                thread::sleep(sleep_until - now);
            }
        }
        result.ok()
    }

    /// runs until told to stop externally
    pub fn run<E, T>(self: &mut Self)
        where C: Clock<T>,
              G: event_queue::Simulation<E, T>,
              E: event_queue::GeneralEvent<G>,
              T: Ord + Clone,
    {
        let mut should_exit = false;
        let mut upd = None;
        while !should_exit {
            let now = time::Instant::now();
            let in_game = self.clock.in_game(now);
            upd = upd.or_else(|| self.external.try_recv().ok());
            if let Some(upd) = upd.take() {
                self.clock.finished_cycle(now, in_game.clone());
                should_exit = apply_update(&mut self.game, upd, in_game);
            } else if let Some(et) = self.game.as_mut().soonest() {
                if et <= in_game {
                    self.clock.finished_cycle(now, et);
                    // then execute any internal instructions
                    self.game.invoke_next();
                } else {
                    self.clock.end_cycles();
                    // then wait for more instructions
                    let sleep_for = self.clock
                                        .minimum_wait(in_game, et.clone());
                    upd = self.recv_timeout_or_sleep(sleep_for, now);
                }
            } else {
                self.clock.end_cycles();
                // if necessary wait forever
                upd = self.external.recv().ok();
                // if the channel closes, and we have nothing to do, exit
                should_exit = upd.is_none();
            }
        }
        self.clock.end_cycles();
    }
}

pub trait Interruption<G> {
    /// returns true if the server should stop
    fn update(
        self: Self,
        game: &mut G,
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

