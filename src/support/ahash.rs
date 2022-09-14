#![cfg(feature = "ahash")]

use ahash::{AHasher, RandomState};

impl_total_size_childless! {
    AHasher,
    RandomState,
}

// `ahash::{AHashMap, AHashSet}` just deref into std maps & sets
#[cfg(feature = "ahash-std")]
mod ahash_std {
    use crate::{Context, SizeOf};
    use ahash::{AHashMap, AHashSet};
    use std::ops::Deref;

    impl<K, V, S> SizeOf for AHashMap<K, V, S>
    where
        K: SizeOf,
        V: SizeOf,
        S: SizeOf,
    {
        #[inline]
        fn size_of_children(&self, context: &mut Context) {
            self.deref().size_of_children(context);
        }
    }

    impl<K, S> SizeOf for AHashSet<K, S>
    where
        K: SizeOf,
        S: SizeOf,
    {
        #[inline]
        fn size_of_children(&self, context: &mut Context) {
            self.deref().size_of_children(context);
        }
    }
}
