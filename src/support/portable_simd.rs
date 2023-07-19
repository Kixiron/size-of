#![cfg(feature = "portable-simd")]

use crate::{Context, SizeOf};
use core::simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount, Which};

impl_total_size_childless! {
    Which,
}

impl<T, const N: usize> SizeOf for Simd<T, N>
where
    T: SimdElement + SizeOf,
    LaneCount<N>: SupportedLaneCount,
{
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        for elem in self.as_array() {
            elem.size_of_children(context);
        }
    }
}

impl<T, const N: usize> SizeOf for Mask<T, N>
where
    T: MaskElement + SizeOf,
    LaneCount<N>: SupportedLaneCount,
{
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {
        // A mask's elements are booleans
    }
}

impl<const LANES: usize> SizeOf for LaneCount<LANES> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}
