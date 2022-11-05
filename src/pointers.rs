use crate::{Context, SizeOf};
use alloc::{
    boxed::Box,
    rc::{Rc, Weak as RcWeak},
    sync::{Arc, Weak as ArcWeak},
};
use core::{mem::size_of_val, ptr::NonNull, sync::atomic::AtomicPtr};

// TODO: Do we want to traverse all *accessible* memory or all *owned* memory?
impl<T> SizeOf for &T
where
    T: SizeOf + ?Sized,
{
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {
        // // Only record the data behind the ref we've never seen it before
        // if context.insert_ref(self) {
        //     context.add(size_of_val(*self));
        //     T::total_size_of_children(self, context);
        // }
    }
}

impl<T> SizeOf for &mut T
where
    T: SizeOf + ?Sized,
{
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {
        // Mutable references are exclusive so there should only ever be one of
        // them
        //
        // context.add(size_of_val(*self));
        // T::total_size_of_children(self, context);
    }
}

impl<T: ?Sized> SizeOf for *const T {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl<T: ?Sized> SizeOf for *mut T {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl<T> SizeOf for Box<T>
where
    T: SizeOf + ?Sized,
{
    fn size_of_children(&self, context: &mut Context) {
        let size = size_of_val(self.as_ref());
        if size != 0 {
            context.add(size).add_distinct_allocation();
        }

        T::size_of_children(self, context);
    }
}

impl<T> SizeOf for Rc<T>
where
    T: SizeOf + ?Sized,
{
    fn size_of_children(&self, context: &mut Context) {
        if context.insert_rc(self) {
            context
                .shared(|ctx| {
                    ctx.add(size_of_val(self.as_ref()));
                    T::size_of_children(self, ctx);
                })
                .add_distinct_allocation();
        }
    }
}

// Weak refs aren't owned
// TODO: Should we record the data pointed to by weak refs as shared?
impl<T: ?Sized> SizeOf for RcWeak<T> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl<T> SizeOf for Arc<T>
where
    T: SizeOf + ?Sized,
{
    fn size_of_children(&self, context: &mut Context) {
        if context.insert_arc(self) {
            context
                .shared(|ctx| {
                    ctx.add(size_of_val(self.as_ref()));
                    T::size_of_children(self, ctx);
                })
                .add_distinct_allocation();
        }
    }
}

// Weak refs aren't owned
// TODO: Should we record the data pointed to by weak refs as shared?
impl<T: ?Sized> SizeOf for ArcWeak<T> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl<T: ?Sized> SizeOf for NonNull<T> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl_total_size_childless! {
    AtomicPtr<T>,
}
