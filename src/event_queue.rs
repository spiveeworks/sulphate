use std::collections;
use std::ops;

/// The general event trait.
///
/// This trait is for enums that contain multiple kinds of
///   [Event](../trait.Event.html),
///
/// It is also implemented for the [EventBox](../struct.EventBox.html) struct,
/// which contains an Event trait object.
pub trait GeneralEvent<G> {
    fn invoke(self: Self, game: &mut G);
}

/// The main event trait.
///
/// By default event-queues will polymorphically box up implementors of this
/// trait, allowing for simple extension to your event system.
///
/// Alternatively you can make your own enum type, that sums up all of your
/// Event implementors, and avoid the virtual method calls.
///
/// Note that the supertrait Into<EventBox<G>> is automatically implemented.
pub trait Event<G, E = EventBox<G>>
    where E: GeneralEvent<G>,
{
    fn invoke(self: Self, game: &mut G);
}

// polymorphise Event monomorphisms,
//   since by-value makes more sense to write,
//   and potentially allows implementors to use their own code
//   outside of event contexts
trait PolyEvent<G, E = EventBox<G>>: Event<G, E>
    where E: GeneralEvent<G>,
{
    fn invoke_box(
        self: Box<Self>,
        game: &mut G,
    );
}

// note the resemlence to FnBox
impl<G, E, Es> PolyEvent<G, E> for Es
    where Es: Event<G, E>,
          E: GeneralEvent<G>,
{
    fn invoke_box(
        self: Box<Self>,
        game: &mut G,
    ) {
        self.invoke(game);
    }
}

pub struct EventBox<G>(Box<PolyEvent<G>>);

impl<G> GeneralEvent<G> for EventBox<G> {
    fn invoke(
        self: Self,
        game: &mut G,
    ) {
        self.0.invoke_box(game)
    }
}

pub struct EventQueue<E, T>
    where T: Ord + Clone
{
    now: T,
    // TODO small vec
    events: collections::BTreeMap<T, Vec<Option<E>>>,
}

pub type PolyEventQueue<G, T> = EventQueue<EventBox<G>, T>;

impl<E, T> EventQueue<E, T>
    where T: Ord + Clone  // doesn't necessarily need Clone but that's silly
{
    pub fn new(initial_time: T) -> Self {
        EventQueue {
            now: initial_time,
            events: collections::BTreeMap::new(),
        }
    }


    pub fn now(self: &Self) -> T {
        self.now.clone()
    }

    pub fn soonest(&self) -> Option<T> {
        self.events
            .keys()
            .next()
            .map(Clone::clone)
    }

    fn has_event_by(self: &Self, time: &T) -> bool {
        if let Some(next_time) = self.soonest() {
            next_time <= *time
        } else {
            false
        }
    }

    pub fn is_empty(self: &Self) -> bool {
        self.events.is_empty()
    }

    pub fn enqueue_absolute<Es>(self: &mut Self, event: Es, execute_time: T)
        -> usize
        where Es: Into<E>
    {
        let call_back = event.into();
        let events = self
            .events
            .entry(execute_time)
            .or_insert_with(|| Vec::new());
        let result = events.len();
        events.push(Some(call_back));
        result
    }

    pub fn enqueue_relative<Es, D>(
        self: &mut Self,
        event: Es,
        execute_delay: D,
    ) -> usize
        where Es: Into<E>,
              T: ops::Add<D, Output=T>,
    {
        let execute_time = self.now() + execute_delay;
        self.enqueue_absolute(event, execute_time)
    }

    pub fn cancel_event(self: &mut Self, execute_time: &T, id: usize) {
        if let Some(events) = self.events.get_mut(execute_time) {
            if let Some(event) = events.get_mut(id) {
                event.take();
            }
        }
    }

    /// progresses in-game time to the next event,
    /// or to the specified time if that is sooner
    /// returns the result of self.has_event_by(&until)
    pub fn progress_time(self: &mut Self, until: T) -> bool
        where T: Clone
    {
        let has_event_by = self.has_event_by(&until);
        self.now = {
            if has_event_by {
                self.soonest().unwrap()
            } else {
                until
            }
        };
        has_event_by
    }
}

impl<G, T> PolyEventQueue<G, T>
    where T: Ord + Clone
{
    pub fn enqueue_box_absolute<Es>(
        self: &mut Self,
        event: Es,
        execute_time: T,
    )
        where Es: 'static + Event<G>,
    {
        self.enqueue_absolute(EventBox(Box::new(event)), execute_time);
    }

    pub fn enqueue_box_relative<Es, D>(
        self: &mut Self,
        event: Es,
        execute_delay: D,
    )
        where Es: 'static + Event<G>,
              T: ops::Add<D, Output=T> + Clone,
    {
        self.enqueue_relative(EventBox(Box::new(event)), execute_delay);
    }
}

pub trait Simulation<E, T>
    where Self: Sized + AsMut<EventQueue<E, T>>,
          T: Ord + Clone,
          E: GeneralEvent<Self>,
{
    fn invoke_next(self: &mut Self);
    fn simulate(self: &mut Self, until: T);
}

impl<G, E, T> Simulation<E, T> for G
    where G: AsMut<EventQueue<E, T>>,
          T: Ord + Clone,
          E: GeneralEvent<G>,
{
    fn invoke_next(self: &mut Self) {
        let next_events = {
            let time = self.as_mut();
            let soonest = time.soonest();
            if soonest.is_none() {
                return
            }
            let soonest = soonest.unwrap();

            // second unwrap should be justified unless `soonest()` misbehaves
            let events = time.events.remove(&soonest).unwrap();

            if time.now < soonest {
                time.now = soonest;
            }

            events
        };

        for event in next_events.into_iter().flat_map(|x| x) {
            event.invoke(self);
        }
    }

    fn simulate(self: &mut Self, until: T) {
        while self.as_mut().has_event_by(&until) {
            self.invoke_next();
        }
        self.as_mut().now = until;
    }
}

