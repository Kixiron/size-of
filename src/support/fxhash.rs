#![cfg(feature = "fxhash")]

use fxhash::{FxHasher, FxHasher32, FxHasher64};

impl_total_size_childless! {
    FxHasher,
    FxHasher32,
    FxHasher64,
}
