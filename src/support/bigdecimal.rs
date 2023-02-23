#![cfg(feature = "bigdecimal")]

use crate::{Context, SizeOf};
use bigdecimal::BigDecimal;
use core::mem::size_of;

#[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
type BigDigit = u64;
#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "128")))]
type BigDigit = u32;

impl SizeOf for BigDecimal {
    fn size_of_children(&self, context: &mut Context) {
        // TODO: There's no way to access allocated capacity
        context
            .add_arraylike(self.digits() as usize, size_of::<BigDigit>())
            .add_distinct_allocation();
    }
}
