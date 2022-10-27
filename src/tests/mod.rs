#![cfg(test)]

use crate::{collections::btree::estimate_btree_size, SizeOf, TotalSize};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec,
    vec::Vec,
};
use core::mem::size_of;

#[test]
fn primitives() {
    assert_eq!(0u8.size_of(), TotalSize::total(1));
    assert_eq!(0u16.size_of(), TotalSize::total(2));
    assert_eq!(0u32.size_of(), TotalSize::total(4));
    assert_eq!(0u64.size_of(), TotalSize::total(8));
    assert_eq!(0usize.size_of(), TotalSize::total(size_of::<usize>()));

    assert_eq!(0i8.size_of(), TotalSize::total(1));
    assert_eq!(0i16.size_of(), TotalSize::total(2));
    assert_eq!(0i32.size_of(), TotalSize::total(4));
    assert_eq!(0i64.size_of(), TotalSize::total(8));
    assert_eq!(0isize.size_of(), TotalSize::total(size_of::<isize>()));

    assert_eq!(0f32.size_of(), TotalSize::total(4));
    assert_eq!(0f64.size_of(), TotalSize::total(8));

    assert_eq!('f'.size_of(), TotalSize::total(4));
    assert_eq!("Hello World!".size_of(), TotalSize::total(12));
    assert_eq!(
        (&"Hello World!").size_of(),
        TotalSize::total(12 + size_of::<usize>()),
    );
    assert_eq!(true.size_of(), TotalSize::total(1));
}

#[test]
fn boxed() {
    assert_eq!(
        Box::new(0u32).size_of(),
        TotalSize::new(4 + size_of::<usize>(), 0, 0, 1),
    );
}

#[test]
fn slices() {
    let array: Box<[u32]> = vec![0; 64].into_boxed_slice();
    assert_eq!(array[5..10].size_of(), TotalSize::total(4 * 5));
    assert_eq!(array[..32].size_of(), TotalSize::total(4 * 32));
    assert_eq!(
        <Box<_> as SizeOf>::size_of(&array),
        TotalSize::new(size_of::<Box<[u32]>>() + size_of::<[u32; 64]>(), 0, 0, 1),
    );

    let array: Box<[u32; 1000]> = vec![0; 1000].into_boxed_slice().try_into().unwrap();
    assert_eq!(
        <Box<_> as SizeOf>::size_of(&array),
        TotalSize::new(
            size_of::<Box<[u32; 1000]>>() + size_of::<[u32; 1000]>(),
            0,
            0,
            1,
        ),
    );
}

#[test]
fn vec() {
    let vec: Vec<u32> = vec![0; 64];
    assert_eq!(vec[5..10].size_of(), TotalSize::total(4 * 5));
    assert_eq!(vec[..32].size_of(), TotalSize::total(4 * 32));
    assert_eq!(
        vec.size_of(),
        TotalSize::new(size_of::<Vec<u32>>() + size_of::<[u32; 64]>(), 0, 0, 1),
    );

    let mut overallocated = Vec::with_capacity(1000);
    assert_eq!(
        overallocated.size_of(),
        TotalSize::new(size_of::<Vec<u8>>() + 1000, 1000, 0, 1),
    );

    overallocated.extend(0u8..100);
    assert_eq!(
        overallocated.size_of(),
        TotalSize::new(size_of::<Vec<u8>>() + 1000, 900, 0, 1),
    );

    let mut vec_o_vecs = Vec::new();
    assert_eq!(
        vec_o_vecs.size_of(),
        TotalSize::total(size_of::<Vec<Vec<u8>>>()),
    );

    vec_o_vecs.reserve_exact(1);
    vec_o_vecs.push(Vec::new());
    assert_eq!(
        vec_o_vecs.size_of(),
        TotalSize::new(size_of::<Vec<Vec<u8>>>() + size_of::<Vec<u8>>(), 0, 0, 1),
    );

    vec_o_vecs[0].reserve_exact(1000);
    vec_o_vecs[0].extend(0u8..100);
    assert_eq!(
        vec_o_vecs.size_of(),
        TotalSize::new(
            size_of::<Vec<Vec<u8>>>() + size_of::<Vec<u8>>() + 1000,
            900,
            0,
            2,
        ),
    );
}

#[test]
fn strings() {
    let string_a = String::from("01234567");
    assert_eq!(
        string_a.size_of(),
        TotalSize::new(size_of::<String>() + 8, 0, 0, 1),
    );

    let string_b = String::from("0123456789012345");
    assert_eq!(
        string_b.size_of(),
        TotalSize::new(size_of::<String>() + 16, 0, 0, 1),
    );

    let mut overallocated = String::with_capacity(1000);
    assert_eq!(
        overallocated.size_of(),
        TotalSize::new(size_of::<String>() + 1000, 1000, 0, 1),
    );

    overallocated.push_str("0123456789012345");
    assert_eq!(
        overallocated.size_of(),
        TotalSize::new(size_of::<String>() + 1000, 1000 - 16, 0, 1),
    );
}

#[test]
fn btree() {
    let empty_set = BTreeSet::<u32>::new();
    assert_eq!(
        empty_set.size_of(),
        TotalSize::total(size_of::<BTreeSet<u32>>()),
    );

    let empty_map = BTreeMap::<u32, u32>::new();
    assert_eq!(
        empty_map.size_of(),
        TotalSize::total(size_of::<BTreeMap<u32, u32>>()),
    );

    let mut set = BTreeSet::<u32>::new();
    set.extend(0..10);
    assert_eq!(
        set.size_of(),
        TotalSize::new(
            size_of::<BTreeSet<u32>>() + estimate_btree_size::<u32, ()>(10),
            0,
            0,
            1,
        ),
    );

    let mut map = BTreeMap::<u32, u32>::new();
    map.extend((0..10).map(|x| (x, 0)));
    assert_eq!(
        map.size_of(),
        TotalSize::new(
            size_of::<BTreeMap<u32, u32>>() + estimate_btree_size::<u32, u32>(10),
            0,
            0,
            1,
        ),
    );
}

// TODO: Test shared pointers

#[cfg(feature = "std")]
mod std {
    use crate::{std_impls::hashmap::estimate_hashmap_size, SizeOf, TotalSize};
    use core::mem::size_of;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn ui() {
        let tests = trybuild::TestCases::new();
        tests.pass("src/tests/pass/*.rs");
        tests.compile_fail("src/tests/fail/*.rs");
    }

    #[test]
    fn hashset() {
        let empty = HashSet::<u32>::new();
        assert_eq!(empty.size_of(), TotalSize::total(size_of::<HashSet<u32>>()),);

        let allocated = HashSet::<u32>::with_capacity(1024);
        let (total_bytes, used_bytes) = estimate_hashmap_size::<u32, ()>(0, 1024);
        assert_eq!(
            allocated.size_of(),
            TotalSize::new(
                size_of::<HashSet<u32>>() + total_bytes,
                total_bytes - used_bytes,
                0,
                1,
            ),
        );

        // TODO: Set containing elements
    }

    #[test]
    fn hashmap() {
        let empty = HashMap::<u32, u32>::new();
        assert_eq!(
            empty.size_of(),
            TotalSize::total(size_of::<HashMap<u32, u32>>()),
        );

        let allocated = HashMap::<u32, u32>::with_capacity(1024);
        let (total_bytes, used_bytes) = estimate_hashmap_size::<u32, u32>(0, 1024);
        assert_eq!(
            allocated.size_of(),
            TotalSize::new(
                size_of::<HashMap<u32, u32>>() + total_bytes,
                total_bytes - used_bytes,
                0,
                1,
            ),
        );

        // TODO: Map containing elements
    }
}
