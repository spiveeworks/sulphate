use std::any;
use std::any::Any;
use std::collections;

pub type UID = u64;

type Key = (any::TypeId, UID);

// heap in the memory sense not the queue sense
pub struct EntityHeap {
    content: collections::HashMap<Key, Box<Any>>
}

static DOWNCAST_ERROR: &'static str = "\
Value stored under incorrect type information. \
";

fn unwrap_box<T: Any>(box_val: Box<Any>) -> T {
    *box_val.downcast()
            .ok()
            .expect(DOWNCAST_ERROR)
}

fn unwrap_box_ref<T: Any>(box_ref: &Box<Any>) -> &T {
    box_ref.downcast_ref()
           .expect(DOWNCAST_ERROR)
}

fn unwrap_box_mut<T: Any>(box_mut: &mut Box<Any>) -> &mut T {
    box_mut.downcast_mut()
           .expect(DOWNCAST_ERROR)
}

impl EntityHeap {
    pub fn new() -> EntityHeap {
        let content = collections::HashMap::new();
        EntityHeap { content }
    }

    pub fn get<T: Any>(self: &Self, k: UID) -> Option<&T> {
        let ty = any::TypeId::of::<T>();
        self.content
            .get(&(ty, k))
            .map(unwrap_box_ref)
    }

    pub fn get_mut<T: Any>(self: &mut Self, k: UID) -> Option<&mut T> {
        let ty = any::TypeId::of::<T>();
        self.content
            .get_mut(&(ty, k))
            .map(unwrap_box_mut)
    }

    pub fn insert<T: Any>(self: &mut Self, k: UID, v: T) -> Option<T> {
        let ty = any::TypeId::of::<T>();
        let val = Box::new(v);
        self.content
            .insert((ty, k), val)
            .map(unwrap_box)
    }

    pub fn remove<T: Any>(self: &mut Self, k: UID) -> Option<T> {
        let ty = any::TypeId::of::<T>();
        self.content
            .remove(&(ty, k))
            .map(unwrap_box)
    }
}


