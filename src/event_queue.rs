use std::cmp;
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

struct QueueElement<E, T>
    where T: Ord
{
    execute_time: T,
    call_back: E,
}

impl<E, T> PartialEq for QueueElement<E, T>
    where T: Ord
{
    fn eq(
        self: &Self,
        other: &Self,
    ) -> bool {
        self.execute_time == other.execute_time
    }
}

impl<E, T> Eq for QueueElement<E, T>
    where T: Ord
{}

impl<E, T> PartialOrd for QueueElement<E, T>
    where T: Ord
{
    fn partial_cmp(
        self: &Self,
        other: &Self
    ) -> Option<cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<E, T> Ord for QueueElement<E, T>
    where T: Ord
{
    fn cmp(
        self: &Self,
        other: &Self
    ) -> cmp::Ordering {
        use std::cmp::Ordering::*;
        match Ord::cmp(&self.execute_time, &other.execute_time) {
            Less => Greater,  // lower time = higher priority
            Equal => Equal,
            Greater => Less,
        }
    }
}

pub struct EventQueue<E, T>
    where T: Ord
{
    now: T,
    queue: collections::BinaryHeap<QueueElement<E, T>>,
}

pub type PolyEventQueue<G, T> = EventQueue<EventBox<G>, T>;

impl<E, T> EventQueue<E, T>
    where T: Ord
{
    pub fn new(initial_time: T) -> Self {
        EventQueue {
            now: initial_time,
            queue: collections::BinaryHeap::new(),
        }
    }


    pub fn now(self: &Self) -> T
        where T: Clone
    {
        self.now.clone()
    }

    pub fn now_ref(self: &Self) -> &T {
        &self.now
    }

    pub fn soonest_ref(&self) -> Option<&T> {
        self.queue
            .peek()
            .map(|qe| &qe.execute_time)
    }

    pub fn soonest(&self) -> Option<T>
        where T: Clone
    {
        self.soonest_ref().map(Clone::clone)
    }

    fn has_event_by(self: &Self, time: &T) -> bool {
        if let Some(next_time) = self.soonest_ref() {
            next_time <= time
        } else {
            false
        }
    }

    pub fn is_empty(self: &Self) -> bool {
        self.queue.is_empty()
    }

    pub fn enqueue_absolute<Es>(self: &mut Self, event: Es, execute_time: T)
        where Es: Into<E>
    {
        let call_back = event.into();
        let element = QueueElement { call_back, execute_time };
        self.queue.push(element);
    }

    pub fn enqueue_relative<Es, D>(self: &mut Self, event: Es, execute_delay: D)
        where Es: Into<E>,
              T: ops::Add<D, Output=T> + Clone,
    {
        let execute_time = self.now() + execute_delay;
        self.enqueue_absolute(event, execute_time);
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
    where T: Ord
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
          T: Ord,
          E: GeneralEvent<Self>,
{
    fn invoke_next(self: &mut Self);
    fn simulate(self: &mut Self, until: T);
}

impl<G, E, T> Simulation<E, T> for G
    where G: AsMut<EventQueue<E, T>>,
          T: Ord,
          E: GeneralEvent<G>,
{
    fn invoke_next(self: &mut Self) {
        let next_event = {
            let time = self.as_mut();
            let element = time.queue.pop();
            if element.is_none() {
                return
            }
            let QueueElement { execute_time, call_back } = element.unwrap();

            if time.now < execute_time {
                time.now = execute_time;
            }
            call_back
        };

        next_event.invoke(self);
    }

    fn simulate(self: &mut Self, until: T) {
        while self.as_mut().has_event_by(&until) {
            self.invoke_next();
        }
        self.as_mut().now = until;
    }
}

