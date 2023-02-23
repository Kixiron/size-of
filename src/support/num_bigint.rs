#![cfg(feature = "num-bigint")]

use crate::{Context, SizeOf};
use core::mem::size_of;
use num_bigint::{
    BigInt, BigUint, ParseBigIntError, Sign, TryFromBigIntError, U32Digits, U64Digits,
};

impl_total_size_childless! {
    Sign,
    ParseBigIntError,
    TryFromBigIntError<T>,
}

impl SizeOf for BigUint {
    fn size_of_children(&self, context: &mut Context) {
        // TODO: There's no way to access allocated capacity
        context
            .add_arraylike(self.iter_u64_digits().len(), size_of::<u64>())
            .add_distinct_allocation();
    }
}

impl SizeOf for BigInt {
    fn size_of_children(&self, context: &mut Context) {
        // TODO: There's no way to access allocated capacity
        context
            .add_arraylike(self.iter_u64_digits().len(), size_of::<u64>())
            .add_distinct_allocation();
    }
}

impl SizeOf for U32Digits<'_> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}

impl SizeOf for U64Digits<'_> {
    #[inline]
    fn size_of_children(&self, _context: &mut Context) {}
}
