# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.1.5] - 2023-02-23

## Added

- Implemented `SizeOf` for `std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6}`
- Added support for the [`bigdecimal`](https://crates.io/crates/bigdecimal) crate under the `bigdecimal` feature
- Added support for the [`num-bigint`](https://crates.io/crates/num-bigint) crate under the `num-bigint` feature

## Changed

- Implemented `SizeOf` for `NonNull<T>` where `T: ?Sized`
- Updated the `hashbrown` crate

## Fixed

- Made `Box`, `Vec`, `VecDeque` and `BinaryHeap` not log allocations when they contain ZSTs

## [0.1.4] - 2022-11-04

## Added

- Implemented `SizeOf` for `core::any::TypeId`

## Changed

- Made `SizeOf` impls for `Context`, `TotalSize` and `HumanBytes` unconditional
  (were previously dependent on the `derive` feature)
- `derive(SizeOf)` now partially normalizes types and doesn't emit bounds for `fn` types

## [0.1.3] - 2022-10-21

## Added

- Support for the [`xxhash-rust`](https://docs.rs/xxhash-rust) crate with the `xxhash-xxh32`, `xxhash-xxh64`
  and `xxhash-xxh3` features for the associated `xxh32`, `xxh64` and `xxh3` features within `xxhash-rust` 

## [0.1.2] - 2022-10-12

### Added

- Implemented `SizeOf` for `std::hash::BuildHasherDefault`
- Support for the [`arcstr`](https://docs.rs/arcstr) crate under the `arcstr` feature
- Support for the [`hashbrown`](https://docs.rs/hashbrown)  crate under the `hashbrown` feature
- Support for the [`fxhash`](https://docs.rs/fxhash/latest/fxhash) crate under the `fxhash` feature
- Support for the [`rust_decimal`](https://docs.rs/rust_decimal) crate under the `rust_decimal` feature
- Support for the [`ordered-float`](https://docs.rs/ordered-float) crate under the `ordered-float` feature
- Support for the [`ahash`](https://docs.rs/ahash) crate under the `ahash` feature (along with
  the `ahash-std` feature for when `ahash` has its `std` feature enabled)
- Support for the [`time`](https://docs.rs/time) crate under the `time` feature along with the `time-std`
  feature for enabling support for `time`'s `std` feature
- Support for the [`chrono`](https://docs.rs/chrono) crate under the `chrono` feature

<!-- next-url -->
[Unreleased]: https://github.com/Kixiron/size-of/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/Kixiron/size-of/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/Kixiron/size-of/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/Kixiron/size-of/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/Kixiron/size-of/compare/...v0.1.2
