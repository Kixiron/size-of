use core::mem::size_of;
use size_of::{SizeOf, TotalSize};
#[cfg(not(feature = "derive"))]
use size_of_derive::SizeOf;

#[derive(SizeOf)]
struct Zst;

#[derive(SizeOf)]
struct TupleStruct(usize, usize, bool, #[size_of(skip)] Vec<u8>);

#[derive(SizeOf)]
struct NestedTupleStruct(usize, usize, bool, Vec<u8>);

#[derive(SizeOf)]
struct BracketStruct {
    foo: usize,
    bar: usize,
    tuple: NestedTupleStruct,
    #[size_of(skip)]
    nothing: Vec<u8>,
}

fn main() {
    assert_eq!(Zst.size_of(), TotalSize::zero());

    let tuple = TupleStruct(10, 10, false, Vec::with_capacity(200));
    assert_eq!(tuple.size_of(), TotalSize::total(size_of::<TupleStruct>()));

    let bracket = BracketStruct {
        foo: 10,
        bar: 200,
        tuple: NestedTupleStruct(10, 10, false, Vec::with_capacity(200)),
        nothing: Vec::with_capacity(1000),
    };
    assert_eq!(
        bracket.size_of(),
        TotalSize::new(size_of::<BracketStruct>() + 200, 200, 0, 1),
    );
}
