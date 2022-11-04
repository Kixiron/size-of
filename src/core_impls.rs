use crate::{Context, SizeOf};
use alloc::{
    alloc::Layout,
    borrow::{Cow, ToOwned},
};
use core::{
    any::TypeId,
    cell::{Cell, RefCell},
    cmp::{self, Reverse},
    convert::Infallible,
    ffi::CStr,
    fmt::Arguments,
    future::Pending,
    hash::BuildHasherDefault,
    marker::{PhantomData, PhantomPinned},
    mem::{ManuallyDrop, MaybeUninit},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
    },
    ops::{ControlFlow, Deref},
    panic::{AssertUnwindSafe, Location},
    pin::Pin,
    sync::atomic::{
        self, AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16,
        AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
    },
    task::Poll,
    time::Duration,
};

impl<const N: usize, T> SizeOf for [T; N]
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.as_slice().size_of_children(context);
    }
}

impl<T> SizeOf for [T]
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.iter()
            .for_each(|element| element.size_of_children(context));
    }
}

impl<T> SizeOf for ManuallyDrop<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.deref().size_of_children(context);
    }
}

impl<T> SizeOf for Option<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        if let Some(inner) = self {
            inner.size_of_children(context);
        }
    }
}

impl<T, E> SizeOf for Result<T, E>
where
    T: SizeOf,
    E: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        match self {
            Ok(ok) => ok.size_of_children(context),
            Err(err) => err.size_of_children(context),
        }
    }
}

impl<T> SizeOf for Reverse<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.0.size_of_children(context);
    }
}

// TODO: Is there a better impl for this?
impl<T> SizeOf for Pin<T>
where
    T: Deref,
    T::Target: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.deref().size_of_children(context);
    }
}

impl<T> SizeOf for Cell<T>
where
    T: Copy + SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.get().size_of_children(context);
    }
}

impl<T> SizeOf for RefCell<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        // Ignore any errors that occur while trying to borrow a RefCell
        if let Ok(cell) = self.try_borrow() {
            cell.deref().size_of_children(context);
        }
    }
}

impl SizeOf for Location<'_> {
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.file().size_of_children(context);
    }
}

// TODO: Saturating<T> once it's stable
impl<T> SizeOf for Wrapping<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.0.size_of_children(context);
    }
}

impl<C, B> SizeOf for ControlFlow<C, B>
where
    C: SizeOf,
    B: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        match self {
            Self::Continue(cont) => cont.size_of_children(context),
            Self::Break(brk) => brk.size_of_children(context),
        }
    }
}

impl<'a, T> SizeOf for Cow<'a, T>
where
    T: ToOwned + ?Sized + 'a,
    T::Owned: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        if let Self::Owned(owned) = self {
            owned.size_of_children(context);
        }
    }
}

// TODO: `core::future::Ready<T>`, the problem is that currently there's no way
// to access the inner value

impl<T> SizeOf for Poll<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        if let Self::Ready(ready) = self {
            ready.size_of_children(context);
        }
    }
}

impl<T> SizeOf for AssertUnwindSafe<T>
where
    T: SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.0.size_of_children(context);
    }
}

impl SizeOf for Arguments<'_> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl_total_size_childless! {
    str,
    bool,
    char,

    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    usize,
    isize,

    f32,
    f64,

    AtomicBool,
    AtomicU8,
    AtomicI8,
    AtomicU16,
    AtomicI16,
    AtomicU32,
    AtomicI32,
    AtomicU64,
    AtomicI64,
    AtomicUsize,
    AtomicIsize,

    NonZeroU8,
    NonZeroI8,
    NonZeroU16,
    NonZeroI16,
    NonZeroU32,
    NonZeroI32,
    NonZeroU64,
    NonZeroI64,
    NonZeroU128,
    NonZeroI128,
    NonZeroUsize,
    NonZeroIsize,

    CStr,
    Layout,
    TypeId,
    Duration,
    // `Pending<T>` is a zst
    Pending<T>,
    Infallible,
    cmp::Ordering,
    PhantomPinned,
    MaybeUninit<T>,
    PhantomData<T>,
    atomic::Ordering,
    // BuildHasherDefault is a zst
    BuildHasherDefault<T>,
}

// Implement SizeOf for up to 16-tuples
impl_tuple! {
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P),
}

// Implement SizeOf for different calling conventions for functions with up to
// 16 arguments
impl_function_ptrs! {
    "C",
    "Rust",
    "aapcs",
    "cdecl",
    "win64",
    "sysv64",
    "system",
    "stdcall",
    "fastcall",
}
