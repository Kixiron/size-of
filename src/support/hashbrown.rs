#![cfg(feature = "hashbrown")]

use crate::{std_impls::hashmap::estimate_hashmap_size, Context, SizeOf};
use hashbrown::{HashMap, HashSet};

impl<K, S> SizeOf for HashSet<K, S>
where
    K: SizeOf,
    S: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            let (total_bytes, used_bytes) =
                estimate_hashmap_size::<K, ()>(self.len(), self.capacity());

            context
                .add(used_bytes)
                .add_excess(total_bytes - used_bytes)
                .add_distinct_allocation();

            self.iter().for_each(|key| key.size_of_children(context));
        }

        self.hasher().size_of_children(context);
    }
}

impl<K, V, S> SizeOf for HashMap<K, V, S>
where
    K: SizeOf,
    V: SizeOf,
    S: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            let (total_bytes, used_bytes) =
                estimate_hashmap_size::<K, V>(self.len(), self.capacity());

            context
                .add(used_bytes)
                .add_excess(total_bytes - used_bytes)
                .add_distinct_allocation();

            self.iter().for_each(|(key, value)| {
                key.size_of_children(context);
                value.size_of_children(context);
            });
        }

        self.hasher().size_of_children(context);
    }
}
