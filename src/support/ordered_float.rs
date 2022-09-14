#![cfg(feature = "ordered-float")]

use crate::{Context, SizeOf};
use ordered_float::{Float, FloatIsNan, NotNan, OrderedFloat};

impl_total_size_childless! {
    FloatIsNan,
}

impl<T> SizeOf for NotNan<T>
where
    // FIXME: Not a fan of the `Float` bound but I don't really have a
    // choice
    T: Float + SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        T::size_of_children(self, context);
    }
}

impl<T> SizeOf for OrderedFloat<T>
where
    // FIXME: Not a fan of the `Float` bound but I don't really have a
    // choice
    T: Float + SizeOf,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        T::size_of_children(self, context);
    }
}
