#![cfg(all(
    any(target_arch = "powerpc", target_arch = "powerpc64"),
    feature = "stdsimd",
))]

#[cfg(target_arch = "powerpc")]
use core::arch::powerpc;
#[cfg(target_arch = "powerpc64")]
use core::arch::powerpc64 as powerpc;

use powerpc::{
    vector_bool_char, vector_bool_int, vector_bool_long, vector_bool_short, vector_double,
    vector_float, vector_signed_char, vector_signed_int, vector_signed_long, vector_signed_short,
    vector_unsigned_char, vector_unsigned_int, vector_unsigned_long, vector_unsigned_short,
};

impl_total_size_childless! {
    vector_bool_char,
    vector_bool_int,
    vector_bool_long,
    vector_bool_short,
    vector_double,
    vector_float,
    vector_signed_char,
    vector_signed_int,
    vector_signed_long,
    vector_signed_short,
    vector_unsigned_char,
    vector_unsigned_int,
    vector_unsigned_long,
    vector_unsigned_short,
}
