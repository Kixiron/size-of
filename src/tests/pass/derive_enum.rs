use size_of_derive::SizeOf;

#[derive(SizeOf)]
enum Uninhabited {}

#[derive(SizeOf)]
enum ZstVariants {
    Foo,
    Bar,
    #[size_of(skip)]
    Baz,
}

#[derive(SizeOf)]
enum BracketVariants {
    Foo {
        foo: usize,
    },
    Bar {
        #[size_of(skip)]
        bar: u8,
        bing: ZstVariants,
    },
    #[size_of(skip)]
    Baz {
        baz: Vec<u8>,
    },
}

#[derive(SizeOf)]
enum TupleVariants {
    Foo(usize),
    Bar(#[size_of(skip)] u8, ZstVariants),
    #[size_of(skip)]
    Baz(Vec<u8>),
}

#[derive(SizeOf)]
enum MixedVariants {
    Foo,
    Bar {
        #[size_of(skip)]
        bar: u8,
        bing: ZstVariants,
    },
    #[size_of(skip)]
    Baz(Vec<u8>),
}

#[derive(SizeOf)]
enum AllSkipped {
    #[size_of(skip)]
    Foo(usize),
    Bar(#[size_of(skip)] u8, #[size_of(skip)] ZstVariants),
    #[size_of(skip)]
    Baz(Vec<u8>),
}

fn main() {
    // TODO: Check that sizes are correct
}
