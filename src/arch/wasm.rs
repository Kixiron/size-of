#![cfg(target_arch = "wasm")]

use core::arch::wasm::v128;

impl_total_size_childless! {
    v128,
}
