#![cfg(feature = "std")]

use crate::{Context, SizeOf};
use core::{
    cell::{Cell, UnsafeCell},
    mem::size_of,
    ptr::NonNull,
    sync::atomic::AtomicBool,
};
use std::{
    collections::hash_map::RandomState,
    ffi::{OsStr, OsString},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::{Path, PathBuf},
    sync::{Barrier, Condvar, Mutex, Once, RwLock},
    thread::{Thread, ThreadId},
    time::{Instant, SystemTime},
};

/// The size of elements within `PathBuf` and `OsString`
const PATH_ELEM_SIZE: usize = if cfg!(windows) {
    // Windows uses u16 elements
    size_of::<u16>()
} else {
    // Assume everything else uses u8 elements
    size_of::<u8>()
};

impl_total_size_childless! {
    Path,
    OsStr,
    Barrier,
    Condvar,
    Instant,
    ThreadId,
    SystemTime,
    RandomState,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
}

impl SizeOf for OsString {
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            context
                .add_vectorlike(self.len(), self.capacity(), PATH_ELEM_SIZE)
                .add_distinct_allocation();
        }
    }
}

impl SizeOf for PathBuf {
    fn size_of_children(&self, context: &mut Context) {
        if self.capacity() != 0 {
            context
                .add_vectorlike(self.as_os_str().len(), self.capacity(), PATH_ELEM_SIZE)
                .add_distinct_allocation();
        }
    }
}

impl<T> SizeOf for Mutex<T>
where
    T: SizeOf,
{
    // TODO: More target-specific size estimation
    fn size_of_children(&self, context: &mut Context) {
        if cfg!(target_env = "sgx") {
            context
                .add(estimate_mutex_size_sgx::<T>())
                .add_distinct_allocation();
        }

        // TODO: hermit does allocate a priority queue but we have no way to know how
        // big it is https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/hermit/mutex.rs#L95-L98

        // Ignore any errors that occur while trying to lock a Mutex
        if let Ok(contents) = self.lock() {
            contents.size_of_children(context);
        }
    }
}

// SGX allocates its mutexes so we have to do some legwork to estimate its real
// size. Note that this is still only a best-effort thing, its waiter uses a
// linked list queue internally that we have no way to access or query for size,
// so we just optimistically assume that it never has any waiters.
//
// https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/mutex.rs#L9
const fn estimate_mutex_size_sgx<T>() -> usize {
    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/mutex.rs#L4-L9
    #[allow(dead_code)]
    struct FakeSgxMutex<T> {
        inner: FakeSpinMutex<FakeWaitVariable<bool>>,
        value: UnsafeCell<T>,
    }

    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/waitqueue/spin_mutex.rs#L13-L16
    #[allow(dead_code)]
    struct FakeSpinMutex<T> {
        value: UnsafeCell<T>,
        lock: AtomicBool,
    }

    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/waitqueue/mod.rs#L44-L47
    #[allow(dead_code)]
    struct FakeWaitVariable<T> {
        queue: FakeWaitQueue,
        lock: T,
    }

    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/waitqueue/mod.rs#L87-L91
    #[allow(dead_code)]
    struct FakeWaitQueue {
        inner: FakeUnsafeList<FakeSpinMutex<FakeWaitEntry>>,
    }

    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/waitqueue/mod.rs#L31-L36
    #[allow(dead_code)]
    struct FakeWaitEntry {
        // https://docs.rs/fortanix-sgx-abi/0.5.0/fortanix_sgx_abi/type.Tcs.html
        tcs: NonNull<u8>,
        wake: bool,
    }

    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/waitqueue/unsafe_list.rs#L27-L30
    #[allow(dead_code)]
    struct FakeUnsafeList<T> {
        head_tail: NonNull<FakeUnsafeListEntry<T>>,
        head_tail_entry: Option<FakeUnsafeListEntry<T>>,
    }

    // https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sys/sgx/waitqueue/unsafe_list.rs#L10-L14
    #[allow(dead_code)]
    struct FakeUnsafeListEntry<T> {
        next: NonNull<FakeUnsafeListEntry<T>>,
        prev: NonNull<FakeUnsafeListEntry<T>>,
        value: Option<T>,
    }

    size_of::<FakeSgxMutex<T>>()
}

// TODO: Target-specific size estimation
impl<T> SizeOf for RwLock<T>
where
    T: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        // Ignore any errors that occur while trying to lock an RwLock
        if let Ok(contents) = self.read() {
            contents.size_of_children(context);
        }
    }
}

// https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sync/once.rs#L116-L121
// https://github.com/rust-lang/rust/blob/98f3001eecbe4cbd091c10ffab45b4c164bb507b/library/std/src/sync/once.rs#L180-L184
//
// Technically this is incorrect, `Once` points to a `Waiter` which is part of a
// linked list of `Waiter`s, but we have no way to figure out how long that
// linked list is. In leu of that, we just assume there's only ever one `Waiter`
// in the list. The waiter also potentially holds a `Thread` which itself can
// have heap allocations, but we have no way to access it so there's not much we
// can do there
impl SizeOf for Once {
    fn size_of_children(&self, context: &mut Context) {
        #[allow(dead_code)]
        struct FakeWaiter {
            thread: Cell<Option<Thread>>,
            signaled: AtomicBool,
            next: *const FakeWaiter,
        }

        // We assume the `Once` only points to a single `Waiter`
        context
            .add(size_of::<FakeWaiter>())
            .add_distinct_allocation();
    }
}

pub(crate) mod hashmap {
    use crate::{Context, SizeOf};
    use core::mem::{align_of, size_of};
    use std::collections::{HashMap, HashSet};

    /// Calculates the number of buckets required to hold `capacity` hashmap
    /// elements, based on [hashbrown's innards](https://docs.rs/hashbrown/0.12.3/src/hashbrown/raw/mod.rs.html#185-207)
    #[inline]
    const fn capacity_to_buckets(capacity: usize) -> usize {
        if capacity == 0 {
            0
        } else if capacity < 4 {
            4
        } else if capacity < 8 {
            8
        } else {
            (capacity * 8 / 7).next_power_of_two()
        }
    }

    // https://github.com/rust-lang/hashbrown/blob/master/src/raw/generic.rs#L8-L21
    const GROUP_WIDTH: usize = if cfg!(any(
        target_pointer_width = "64",
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "wasm32",
    )) {
        size_of::<u64>()
    } else {
        size_of::<u32>()
    };

    // https://github.com/rust-lang/hashbrown/blob/2a7c32287247e13680bf874c9a6278aad01fac91/src/raw/mod.rs#L242-L255
    // https://github.com/rust-lang/hashbrown/blob/2a7c32287247e13680bf874c9a6278aad01fac91/src/raw/mod.rs#L1067-L1103
    #[inline]
    pub(crate) const fn calculate_layout_for<T>(buckets: usize) -> usize {
        // FIXME: `max()` isn't a const fn yet
        let align = if align_of::<T>() > GROUP_WIDTH {
            align_of::<T>()
        } else {
            GROUP_WIDTH
        };
        let ctrl_offset = ((size_of::<T>() * buckets) + (align - 1)) & !(align - 1);
        ctrl_offset + buckets + GROUP_WIDTH
    }

    /// Estimates a hashmap's size, returns a tuple containing the total memory
    /// allocated and the portion of that memory that's used
    // TODO: Is this really correct?
    #[inline]
    pub(crate) const fn estimate_hashmap_size<K, V>(
        length: usize,
        capacity: usize,
    ) -> (usize, usize) {
        if capacity == 0 {
            (0, 0)
        } else {
            // Estimate the number of buckets the map contains
            let buckets = capacity_to_buckets(capacity);

            // Estimate the layout of the entire table
            let table_layout = calculate_layout_for::<(K, V)>(buckets);

            // Estimate the memory used by `length` elements
            let used_layout = calculate_layout_for::<(K, V)>(length);

            (table_layout, used_layout)
        }
    }

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
}
