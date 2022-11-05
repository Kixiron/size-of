use crate::{Context, SizeOf};
use alloc::{
    collections::{BinaryHeap, LinkedList, VecDeque},
    ffi::CString,
    string::String,
    vec::Vec,
};
use core::mem::size_of;

impl SizeOf for String {
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            context
                .add_vectorlike(self.len(), self.capacity(), size_of::<u8>())
                .add_distinct_allocation();
        }
    }
}

impl SizeOf for CString {
    fn size_of_children(&self, context: &mut Context) {
        let length = self.to_bytes_with_nul().len();
        if length != 0 {
            context
                .add_arraylike(length, size_of::<u8>())
                .add_distinct_allocation();
        }
    }
}

impl<T> SizeOf for Vec<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            if size_of::<T>() != 0 {
                context
                    .add_vectorlike(self.len(), self.capacity(), size_of::<T>())
                    .add_distinct_allocation();
            }

            self.as_slice().size_of_children(context);
        }
    }
}

impl<T> SizeOf for VecDeque<T>
where
    T: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            if size_of::<T>() != 0 {
                context
                    .add_vectorlike(self.len(), self.capacity(), size_of::<T>())
                    .add_distinct_allocation();
            }

            let (left, right) = self.as_slices();
            left.size_of_children(context);
            right.size_of_children(context);
        }
    }
}

impl<T> SizeOf for BinaryHeap<T>
where
    T: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            if size_of::<T>() != 0 {
                context
                    .add_vectorlike(self.len(), self.capacity(), size_of::<T>())
                    .add_distinct_allocation();
            }

            self.iter()
                .for_each(|element| element.size_of_children(context));
        }
    }
}

impl<T> SizeOf for LinkedList<T>
where
    T: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        let length = self.len();

        if length != 0 {
            // Record each node as a `{ T, *const (), *const () }`
            context
                .add_arraylike(length, size_of::<T>() + (size_of::<*const ()>() * 2))
                .add_distinct_allocations(length);

            self.iter()
                .for_each(|element| element.size_of_children(context));
        }
    }
}

// A btree node has 2*B - 1 (K,V) pairs and (usize, u16, u16)
// overhead, and an internal btree node additionally has 2*B
// `usize` overhead.
// A node can contain between B - 1 and 2*B - 1 elements, so
// we assume it has the midpoint 3/2*B - 1.
pub(crate) mod btree {
    use crate::{Context, SizeOf};
    use alloc::collections::{BTreeMap, BTreeSet};
    use core::mem::size_of;

    // Constants from rust's source:
    // https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html#43-45
    const B: usize = 6;
    const BTREE_MAX: usize = 2 * B - 1;
    const BTREE_MIN: usize = B - 1;

    // A fake btree node, this isn't 100% accurate since the real btree node can
    // have a different layout, but it should be close enough
    #[allow(dead_code)]
    struct FakeNode<K, V> {
        parent: *const (),
        parent_idx: u16,
        len: u16,
        keys: [K; BTREE_MAX],
        values: [V; BTREE_MAX],
    }

    // TODO: Figure out the unused capacity as well
    // TODO: Estimate the number of allocated buckets each btree makes
    pub(crate) const fn estimate_btree_size<K, V>(length: usize) -> usize {
        length * size_of::<FakeNode<K, V>>() * 2 / (BTREE_MAX + BTREE_MIN)
    }

    impl<K> SizeOf for BTreeSet<K>
    where
        K: SizeOf,
    {
        fn size_of_children(&self, context: &mut Context) {
            if !self.is_empty() {
                context
                    .add(estimate_btree_size::<K, ()>(self.len()))
                    // FIXME: Estimate the number of allocated buckets
                    .add_distinct_allocation();

                self.iter().for_each(|key| key.size_of_children(context));
            }
        }
    }

    impl<K, V> SizeOf for BTreeMap<K, V>
    where
        K: SizeOf,
        V: SizeOf,
    {
        fn size_of_children(&self, context: &mut Context) {
            if !self.is_empty() {
                context
                    .add(estimate_btree_size::<K, V>(self.len()))
                    // FIXME: Estimate the number of allocated buckets
                    .add_distinct_allocation();

                self.iter().for_each(|(key, value)| {
                    key.size_of_children(context);
                    value.size_of_children(context);
                });
            }
        }
    }
}
