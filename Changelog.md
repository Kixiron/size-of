# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Added

- Implemented `SizeOf` for `std::hash::BuildHasherDefault`
- Support for the [`arcstr`](https://docs.rs/arcstr) crate under the `arcstr` feature
- Support for the [`hashbrown`](https://docs.rs/hashbrown)  crate under the `hashbrown` feature
- Support for the [`fxhash`](https://docs.rs/fxhash/latest/fxhash) crate under the `fxhash` feature
- Support for the [`rust_decimal`](https://docs.rs/rust_decimal) crate under the `rust_decimal` feature
- Support for the [`ordered-float`](https://docs.rs/ordered-float) crate under the `ordered-float` feature
- Support for the [`ahash`](https://docs.rs/ahash) crate under the `ahash` feature (along with
  the `ahash-std` feature for when `ahash` has its `std` feature enabled)

<!-- next-url -->
[Unreleased]: https://github.com/Kixiron/size-of/compare/...HEAD
