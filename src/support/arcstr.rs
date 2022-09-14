#![cfg(feature = "arcstr")]

use crate::{Context, SizeOf};
use arcstr::{ArcStr, Substr};
use core::mem::size_of;

// https://docs.rs/arcstr/1.1.4/src/arcstr/arc_str.rs.html#751-757
#[repr(C, align(8))]
struct FakeThinInner {
    len_flags: usize,
    strong: usize,
    data: [u8; 0],
}

impl SizeOf for ArcStr {
    fn size_of_children(&self, context: &mut Context) {
        // FIXME: I'd rather use some sort of `ArcStr::as_ptr()` if possible
        if context.insert_ptr(self.as_ptr()) {
            let bytes = size_of::<FakeThinInner>() + self.len();
            context.add(bytes).add_shared(bytes);

            // Static arcstrs don't create any allocations
            if !ArcStr::is_static(self) {
                context.add_distinct_allocation();
            }
        }
    }
}

impl SizeOf for Substr {
    #[inline]
    fn size_of_children(&self, context: &mut Context) {
        self.parent().size_of_children(context);
    }
}

#[cfg(test)]
mod tests {
    use super::FakeThinInner;
    use crate::{SizeOf, TotalSize};
    use arcstr::{ArcStr, Substr};
    use core::mem::size_of;

    #[test]
    fn arcstr_smoke() {
        let empty = ArcStr::new();
        assert_eq!(
            empty.size_of(),
            TotalSize::new(
                size_of::<ArcStr>() + size_of::<FakeThinInner>(),
                0,
                size_of::<FakeThinInner>(),
                0,
            ),
        );

        let shared = size_of::<FakeThinInner>() + 4;
        let total = size_of::<ArcStr>() + shared;

        let static_string = arcstr::literal!("whee");
        assert_eq!(static_string.size_of(), TotalSize::new(total, 0, shared, 0));

        let allocated_string = ArcStr::from("whee");
        assert_eq!(
            allocated_string.size_of(),
            TotalSize::new(total, 0, shared, 1),
        );
    }

    #[test]
    fn shared_arcstr() {
        let string1 = ArcStr::from("whee");
        let string2 = string1.clone();
        let string3 = string1.clone();
        let string4 = string1.clone();

        let size =
            crate::size_of_values([&string1 as _, &string2 as _, &string3 as _, &string4 as _]);

        let shared = size_of::<FakeThinInner>() + 4;
        let total = size_of::<ArcStr>() * 4 + shared;
        assert_eq!(size, TotalSize::new(total, 0, shared, 1));
    }

    #[test]
    fn substr_smoke() {
        let empty = Substr::new();
        assert_eq!(
            empty.size_of(),
            TotalSize::new(
                size_of::<Substr>() + size_of::<FakeThinInner>(),
                0,
                size_of::<FakeThinInner>(),
                0,
            ),
        );

        let shared = size_of::<FakeThinInner>() + 4;
        let total = size_of::<Substr>() + shared;

        let static_substr = Substr::full(arcstr::literal!("whee"));
        assert_eq!(static_substr.size_of(), TotalSize::new(total, 0, shared, 0));

        let allocated_substr = Substr::full(ArcStr::from("whee"));
        assert_eq!(
            allocated_substr.size_of(),
            TotalSize::new(total, 0, shared, 1),
        );

        let sliced_allocated_substr = ArcStr::from("whee").substr(..2);
        assert_eq!(
            sliced_allocated_substr.size_of(),
            TotalSize::new(total, 0, shared, 1),
        );
    }
}
