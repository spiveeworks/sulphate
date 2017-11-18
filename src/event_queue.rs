use std::cmp;
use std::collections;
use std::ops;

use entity_heap;

pub trait Event<T> where T: Ord {
    fn invoke(
        self: Self,
        space: &mut entity_heap::EntityHeap,
        time: &mut EventQueue<T>,
    );
}

// polymorphise Event monomorphisms,
//   since by-value makes more sense to write,
//   and potentially allows implementors to use their own code
//   outside of event contexts
trait PolyEvent<T>: Event<T> where T: Ord {
    fn invoke_box(
        self: Box<Self>,
        space: &mut entity_heap::EntityHeap,
        time: &mut EventQueue<T>,
    );
}

// note the resemlence to FnBox
impl<E, T> PolyEvent<T> for E
    where E: Event<T>,
          T: Ord,
{
    fn invoke_box(
        self: Box<Self>,
        space: &mut entity_heap::EntityHeap,
        time: &mut EventQueue<T>,
    ) {
        self.invoke(space, time);
    }
}


struct QueueElement<T> where T: Ord {
    execute_time: T,
    call_back: Box<PolyEvent<T>>,
}

impl<T> PartialEq for QueueElement<T> where T: Ord {
    fn eq(
        self: &Self,
        other: &Self,
    ) -> bool {
        self.execute_time == other.execute_time
    }
}

impl<T> Eq for QueueElement<T> where T: Ord {}

impl<T> PartialOrd for QueueElement<T> where T: Ord {
    fn partial_cmp(
        self: &Self,
        other: &Self
    ) -> Option<cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<T> Ord for QueueElement<T> where T: Ord {
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

pub struct EventQueue<T> where T: Ord {
    current_time: T,
    queue: collections::BinaryHeap<QueueElement<T>>,
}

impl<T> EventQueue<T> where T: Ord {
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

    pub fn next(&self) -> Option<&T> {
        self.queue
            .peek()
            .map(|qe| &qe.execute_time)
    }

    pub fn invoke_next(self: &mut Self, space: &mut entity_heap::EntityHeap) {
        if let Some(element) = self.queue.pop() {
            let QueueElement { execute_time, call_back } = element;
            if self.current_time < execute_time {
                self.current_time = execute_time;
            }
            call_back.invoke_box(space, self);
        }
    }

    fn has_event_by(self: &mut Self, time: &T) -> bool {
        if let Some(next_time) = self.next() {
            next_time <= time
        } else {
            false
        }
    }


    pub fn simulate(
        self: &mut Self,
        space: &mut entity_heap::EntityHeap,
        until: T,
    ) {
        while self.has_event_by(&until) {
            self.invoke_next(space);
        }
        self.current_time = until;
    }

    pub fn enqueue_absolute<E>(self: &mut Self, event: E, execute_time: T)
        where E: 'static + Event<T>
    {
        let call_back = Box::new(event);
        let element = QueueElement { call_back, execute_time };
        self.queue.push(element);
    }

    pub fn enqueue_relative<E, D>(self: &mut Self, event: E, execute_delay: D)
        where E: 'static + Event<T>,
              T: ops::Add<D, Output=T> + Clone,
    {
        let execute_time = self.current_time.clone() + execute_delay;
        self.enqueue_absolute(event, execute_time);
    }
}

