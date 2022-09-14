#![cfg(feature = "rust_decimal")]

use rust_decimal::Decimal;

impl_total_size_childless! {
    Decimal,
}

#[cfg(test)]
mod tests {
    use crate::{SizeOf, TotalSize};
    use core::mem::size_of;
    use rust_decimal::Decimal;

    #[test]
    fn rust_decimal_is_u128() {
        let decimal = Decimal::MIN;
        assert_eq!(decimal.size_of(), TotalSize::total(size_of::<u128>()));
    }
}
