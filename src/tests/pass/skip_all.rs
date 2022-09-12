use core::mem::{size_of, ManuallyDrop};
use size_of::{SizeOf, TotalSize};
#[cfg(not(feature = "derive"))]
use size_of_derive::SizeOf;

struct NonImplementer;

#[derive(SizeOf)]
#[size_of(skip_all)]
struct ContainsNonImplementer {
    foo: NonImplementer,
    vektor: Vec<u8>,
}

#[derive(SizeOf)]
#[size_of(skip_all)]
union Union {
    foo: ManuallyDrop<Vec<u8>>,
    bar: u8,
}

fn main() {
    let container = ContainsNonImplementer {
        foo: NonImplementer,
        vektor: Vec::with_capacity(1000),
    };
    assert_eq!(
        container.size_of(),
        TotalSize::total(size_of::<ContainsNonImplementer>()),
    );

    let union = Union {
        foo: ManuallyDrop::new(Vec::with_capacity(1000)),
    };
    assert_eq!(union.size_of(), TotalSize::total(size_of::<Union>()));
}
