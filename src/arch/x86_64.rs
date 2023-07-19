//! Arch impls for both x86 and x86_64
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

#[cfg(target_arch = "x86")]
use core::arch::x86;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as x86;

use x86::{CpuidResult, __m128, __m128d, __m128i, __m256, __m256d, __m256i};

impl_total_size_childless! {
    CpuidResult,
    __m128,
    __m128d,
    __m128i,
    __m256,
    __m256d,
    __m256i,
}

#[cfg(feature = "stdsimd")]
mod stdsimd {
    use super::x86::{__m128bh, __m256bh, __m512, __m512bh, __m512d, __m512i};

    impl_total_size_childless! {
        __m128bh,
        __m256bh,
        __m512,
        __m512bh,
        __m512d,
        __m512i,
    }
}
