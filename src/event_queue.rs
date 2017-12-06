use std::cmp;
use std::collections;
use std::ops;

pub trait Event<T, W> where T: Ord {
    fn invoke(
        self: Self,
        time: &mut EventQueue<T, W>,
        world: &mut W,
    );
}

// polymorphise Event monomorphisms,
//   since by-value makes more sense to write,
//   and potentially allows implementors to use their own code
//   outside of event contexts
trait PolyEvent<T, W>: Event<T, W> where T: Ord {
    fn invoke_box(
        self: Box<Self>,
        time: &mut EventQueue<T, W>,
        world: &mut W,
    );
}

// note the resemlence to FnBox
impl<E, T, W> PolyEvent<T, W> for E
    where E: Event<T, W>,
          T: Ord,
{
    fn invoke_box(
        self: Box<Self>,
        time: &mut EventQueue<T, W>,
        world: &mut W,
    ) {
        self.invoke(time, world);
    }
}


struct QueueElement<T, W> where T: Ord {
    execute_time: T,
    call_back: Box<PolyEvent<T, W>>,
}

impl<T, W> PartialEq for QueueElement<T, W> where T: Ord {
    fn eq(
        self: &Self,
        other: &Self,
    ) -> bool {
        self.execute_time == other.execute_time
    }
}

impl<T, W> Eq for QueueElement<T, W> where T: Ord {}

impl<T, W> PartialOrd for QueueElement<T, W> where T: Ord {
    fn partial_cmp(
        self: &Self,
        other: &Self
    ) -> Option<cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<T, W> Ord for QueueElement<T, W> where T: Ord {
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

pub struct EventQueue<T, W> where T: Ord {
    current_time: T,
    queue: collections::BinaryHeap<QueueElement<T, W>>,
}

impl<T, W> EventQueue<T, W> where T: Ord {
    pub fn new(initial_time: T) -> Self {
        EventQueue {
            current_time: initial_time,
            queue: collections::BinaryHeap::new(),
        }
    }

    pub fn now(self: &Self) -> T
        where T: Clone
    {
        self.current_time.clone()
    }

    pub fn current_time(self: &Self) -> &T {
        &self.current_time
    }

    pub fn next_ref(&self) -> Option<&T> {
        self.queue
            .peek()
            .map(|qe| &qe.execute_time)
    }

    pub fn next(&self) -> Option<T> where T: Clone {
        self.next_ref().map(Clone::clone)
    }

    pub fn invoke_next(self: &mut Self, world: &mut W) {
        if let Some(element) = self.queue.pop() {
            let QueueElement { execute_time, call_back } = element;
            if self.current_time < execute_time {
                self.current_time = execute_time;
            }
            call_back.invoke_box(self, world);
        }
    }

    fn has_event_by(self: &Self, time: &T) -> bool {
        if let Some(next_time) = self.next_ref() {
            next_time <= time
        } else {
            false
        }
    }

    pub fn is_empty(self: &Self) -> bool {
        self.queue.is_empty()
    }

    /// progresses in-game time to the next event,
    /// or to the specified time if that is sooner
    /// returns the result of self.has_event_by(&until)
    pub fn progress_time(self: &mut Self, until: T) -> bool
        where T: Clone
    {
        let has_event_by = self.has_event_by(&until);
        self.current_time = {
            if has_event_by {
                self.next().unwrap()
            } else {
                until
            }
        };
        has_event_by
    }



    pub fn simulate(
        self: &mut Self,
        world: &mut W,
        until: T,
    ) {
        while self.has_event_by(&until) {
            self.invoke_next(world);
        }
        self.current_time = until;
    }

    pub fn enqueue_absolute<E>(self: &mut Self, event: E, execute_time: T)
        where E: 'static + Event<T, W>
    {
        let call_back = Box::new(event);
        let element = QueueElement { call_back, execute_time };
        self.queue.push(element);
    }

    pub fn enqueue_relative<E, D>(self: &mut Self, event: E, execute_delay: D)
        where E: 'static + Event<T, W>,
              T: ops::Add<D, Output=T> + Clone,
    {
        let execute_time = self.current_time.clone() + execute_delay;
        self.enqueue_absolute(event, execute_time);
    }
}

