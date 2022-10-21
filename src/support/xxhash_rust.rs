#![cfg(feature = "xxhash-rust")]

impl_total_size_childless! {
    #[cfg(feature = "xxhash-xxh32")]
    xxhash_rust::xxh32::Xxh32,

    #[cfg(feature = "xxhash-xxh64")]
    xxhash_rust::xxh64::Xxh64,
    #[cfg(feature = "xxhash-xxh64")]
    xxhash_rust::xxh64::Xxh64Builder,

    #[cfg(feature = "xxhash-xxh3")]
    xxhash_rust::xxh3::Xxh3,
    #[cfg(feature = "xxhash-xxh3")]
    xxhash_rust::xxh3::Xxh3Builder,
}
