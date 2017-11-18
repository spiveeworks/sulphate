use std::any;
use std::any::Any;
use std::collections;

use rand;

pub type UID = u64;

type Key = (any::TypeId, UID);

// heap in the memory sense not the queue sense
pub struct EntityHeap {
    content: collections::HashMap<Key, Box<Any>>,
    key_seed: rand::XorShiftRng,
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
        let key_seed = rand::weak_rng();
        EntityHeap { content, key_seed }
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

    fn new_uid(self: &mut Self, ty: any::TypeId) -> UID {
        use rand::Rng;
        loop {
            let id = self.key_seed.next_u64();
            if !self.content.contains_key(&(ty, id)) {
                return id;
            }
        }
    }

    pub fn add<T: Any>(self: &mut Self, v: T) -> UID {
        let ty = any::TypeId::of::<T>();
        let val = Box::new(v);
        let id = self.new_uid(ty);
        let overflow = self.content
                           .insert((ty, id), val);
        // this is fine, it will just drop the value,
        // but when debugging I'd want to know what happened
        debug_assert!(overflow.is_none(), "reused key");
        id
    }

    pub fn remove<T: Any>(self: &mut Self, k: UID) -> Option<T> {
        let ty = any::TypeId::of::<T>();
        self.content
            .remove(&(ty, k))
            .map(unwrap_box)
    }
}


