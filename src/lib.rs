// FIXME: Write better top-level docs
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
mod macros;
mod collections;
mod core_impls;
mod human_bytes;
mod pointers;
mod std_impls;
mod tests;

pub use human_bytes::HumanBytes;
#[cfg(feature = "derive")]
pub use size_of_derive::SizeOf;

use alloc::{collections::BTreeSet, rc::Rc, sync::Arc};
use core::{
    iter::Sum,
    mem::{replace, size_of_val},
    ops::{Add, AddAssign, Sub, SubAssign},
};

// TODO: There's some things we could do with allocator-specific size queries
// which would allow us to get the "real" size of everything we interact with.
// Allocators often round up allocation sizes which means that the true size of
// an allocation can differ from the "declared" size, like an allocator giving
// back 4096 bytes if a `Box<[u8; 4000]>` was allocated. Ideally we'd report
// that 4000 bytes were used, but 4096 bytes were allocated in total. This does
// heavily complicate things since we'd need to add support for as many
// different allocators as possible such as various malloc implementations,
// VirtualAlloc(), mimalloc, jemalloc, etc.

/// Get the total size of all given values
///
/// ```rust
/// use core::mem::size_of;
/// use size_of::SizeOf;
///
/// let vector: Vec<u8> = vec![1, 2, 3, 4];
/// let array: [u8; 10] = [255; 10];
///
/// let size = size_of::size_of_values([&vector as &dyn SizeOf, &array as &dyn SizeOf]);
/// assert_eq!(
///     size.total_bytes(),
///     size_of::<Vec<u8>>() + (size_of::<u8>() * 4) + size_of::<[u8; 10]>(),
/// );
/// ```
#[inline]
pub fn size_of_values<'a, I>(values: I) -> TotalSize
where
    I: IntoIterator<Item = &'a dyn SizeOf> + 'a,
{
    let mut context = Context::new();
    values
        .into_iter()
        .for_each(|value| value.size_of_with_context(&mut context));
    context.total_size()
}

/// Types with a size that can be queried at runtime
pub trait SizeOf {
    /// Gets the total size of the current value
    #[inline]
    fn size_of(&self) -> TotalSize {
        let mut context = Context::new();
        self.size_of_with_context(&mut context);
        context.total_size()
    }

    /// Adds the size of the current value to the given [`Context`],
    /// including both the size of the value itself and all of its children
    #[inline]
    fn size_of_with_context(&self, context: &mut Context) {
        context.add(size_of_val(self));
        self.size_of_children(context);
    }

    /// Gets the size of all "children" owned by this value, not including the
    /// size of the value itself.
    ///
    /// This should add all heap allocations owned by the current value to the
    /// given context
    fn size_of_children(&self, context: &mut Context);
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "derive", derive(SizeOf), size_of(crate = "crate"))]
pub struct Context {
    /// The total bytes used
    total_bytes: usize,
    /// The total excess bytes, e.g. the excess capacity allocated by a `Vec`
    excess_bytes: usize,
    /// The total bytes shared between `Rc` and `Arc`-type collections
    shared_bytes: usize,
    /// The total number of distinct allocations made
    distinct_allocations: usize,
    /// Whether all added bytes should be marked as shared
    is_shared: bool,
    /// Keeps track of all pointers (`&T`, `Rc` and `Arc`) we've seen
    pointers: BTreeSet<usize>,
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub const fn is_shared(&self) -> bool {
        self.is_shared
    }

    /// Run the given closure and mark all added allocations as shared
    #[inline]
    pub fn shared<F>(&mut self, with_shared: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let prev = replace(&mut self.is_shared, true);
        with_shared(self);
        self.is_shared = prev;
        self
    }

    #[inline]
    pub fn add_distinct_allocation(&mut self) -> &mut Self {
        self.add_distinct_allocations(1)
    }

    #[inline]
    pub fn add_distinct_allocations(&mut self, allocations: usize) -> &mut Self {
        self.distinct_allocations += allocations;
        self
    }

    #[inline]
    pub fn add(&mut self, size: usize) -> &mut Self {
        self.total_bytes += size;
        if self.is_shared {
            self.shared_bytes += size;
        }

        self
    }

    #[inline]
    pub fn add_excess(&mut self, size: usize) -> &mut Self {
        self.total_bytes += size;
        self.excess_bytes += size;
        if self.is_shared {
            self.shared_bytes += size;
        }

        self
    }

    #[inline]
    pub fn add_arraylike(&mut self, len: usize, element_size: usize) -> &mut Self {
        let bytes = len * element_size;
        self.total_bytes += bytes;
        if self.is_shared {
            self.shared_bytes += bytes;
        }

        self
    }

    #[inline]
    pub fn add_vectorlike(
        &mut self,
        len: usize,
        capacity: usize,
        element_size: usize,
    ) -> &mut Self {
        let used = len * element_size;
        let allocated = capacity * element_size;
        self.total_bytes += allocated;
        self.excess_bytes += allocated - used;
        if self.is_shared {
            self.shared_bytes += allocated;
        }

        self
    }

    #[inline]
    pub fn insert_ptr<T: ?Sized>(&mut self, ptr: *const T) -> bool {
        // TODO: Use `pointer::addr()` whenever strict provenance stabilizes
        self.pointers.insert(ptr as *const T as *const u8 as usize)
    }

    #[inline]
    pub fn add_ptr<T: ?Sized>(&mut self, ptr: *const T) -> &mut Self {
        self.insert_ptr(ptr);
        self
    }

    #[inline]
    pub fn contains_ptr<T: ?Sized>(&self, ptr: *const T) -> bool {
        // TODO: Use `pointer::addr()` whenever strict provenance stabilizes
        self.pointers
            .contains(&(ptr as *const T as *const u8 as usize))
    }

    // fn insert_ref<T: ?Sized>(&mut self, reference: &T) -> bool {
    //     self.insert_ptr(reference)
    // }

    #[inline]
    fn insert_rc<T: ?Sized>(&mut self, rc: &Rc<T>) -> bool {
        self.insert_ptr(Rc::as_ptr(rc))
    }

    #[inline]
    fn insert_arc<T: ?Sized>(&mut self, arc: &Arc<T>) -> bool {
        self.insert_ptr(Arc::as_ptr(arc))
    }

    #[inline]
    pub const fn total_size(&self) -> TotalSize {
        TotalSize::new(
            self.total_bytes,
            self.excess_bytes,
            self.shared_bytes,
            self.distinct_allocations,
        )
    }
}

/// Represents the total space taken up by an instance of a variable, including
/// heap allocations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "derive", derive(SizeOf), size_of(crate = "crate"))]
pub struct TotalSize {
    /// The total bytes used
    total_bytes: usize,
    /// The total excess bytes, e.g. the excess capacity allocated by a `Vec`
    excess_bytes: usize,
    /// The total bytes shared between `Rc` and `Arc`-type collections
    shared_bytes: usize,
    /// The total number of distinct allocations made
    distinct_allocations: usize,
}

impl TotalSize {
    #[inline]
    pub const fn new(
        total_bytes: usize,
        excess_bytes: usize,
        shared_bytes: usize,
        distinct_allocations: usize,
    ) -> Self {
        Self {
            total_bytes,
            excess_bytes,
            shared_bytes,
            distinct_allocations,
        }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self::new(0, 0, 0, 0)
    }

    #[inline]
    pub const fn total(total: usize) -> Self {
        Self::new(total, 0, 0, 0)
    }

    #[inline]
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    #[inline]
    pub const fn excess_bytes(&self) -> usize {
        self.excess_bytes
    }

    #[inline]
    pub const fn shared_bytes(&self) -> usize {
        self.shared_bytes
    }

    #[inline]
    pub const fn distinct_allocations(&self) -> usize {
        self.distinct_allocations
    }

    /// Return the total used bytes, calculated by `total_bytes - excess_bytes`
    #[inline]
    pub const fn used_bytes(&self) -> usize {
        self.total_bytes - self.excess_bytes
    }
}

impl Add for TotalSize {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            total_bytes: self.total_bytes + rhs.total_bytes,
            excess_bytes: self.excess_bytes + rhs.excess_bytes,
            shared_bytes: self.shared_bytes + rhs.shared_bytes,
            distinct_allocations: self.distinct_allocations + rhs.distinct_allocations,
        }
    }
}

impl AddAssign for TotalSize {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for TotalSize {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            total_bytes: self.total_bytes - rhs.total_bytes,
            excess_bytes: self.excess_bytes - rhs.excess_bytes,
            shared_bytes: self.shared_bytes - rhs.shared_bytes,
            distinct_allocations: self.distinct_allocations - rhs.distinct_allocations,
        }
    }
}

impl SubAssign for TotalSize {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Sum for TotalSize {
    #[inline]
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::zero(), |acc, size| acc + size)
    }
}
