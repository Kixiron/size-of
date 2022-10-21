# Size Of

A crate for measuring the total memory usage of an object at runtime

## Features

`size-of` has built-in support for many 3rd party crates that can be enabled with feature flags

- `std`: Enables support for the rust standard library (enabled by default, when disabled `size-of` is `#![no_std]` compatible)
- `derive`: Enables support for `#[derive(SizeOf)]` (enabled by default)
- `time`: Enables support for the [`time`](https://docs.rs/time) crate
  - `time-std`: Enables support for `time`'s `std` feature
- `chrono`: Enables support for the [`chrono`](https://docs.rs/chrono) crate
- `hashbrown`: Enables support for the [`hashbrown`](https://docs.rs/hashbrown) crate
- `fxhash`: Enables support for the [`fxhash`](https://docs.rs/fxhash/latest/fxhash) crate
- `rust_decimal`: Enables support for the [`rust_decimal`](https://docs.rs/rust_decimal) crate
- `ordered-float`: Enables support for the [`ordered-float`](https://docs.rs/ordered-float) crate
- `ahash`: Enables support for the [`ahash`](https://docs.rs/ahash) crate
  - `ahash-std`: Enables support for `ahash`'s `std` feature
- `xxhash-rust`: Enables support for the [`xxhash-rust`](https://docs.rs/xxhash-rust) crate
  - `xxhash-xxh32`: Enables support for `xxhhash-rust`'s `xxh32` feature
  - `xxhash-xxh64`: Enables support for `xxhhash-rust`'s `xxh64` feature
  - `xxhash-xxh3`: Enables support for `xxhhash-rust`'s `xxh3` feature
