// Implement SizeOf for types without meaningful children
macro_rules! impl_total_size_childless {
    ($($(#[$meta:meta])* $($ident:ident)::+$(<$($generic:ident),* $(,)?>)?),* $(,)?) => {
        $(
            $(#[$meta])*
            impl $(<$($generic),*>)? $crate::SizeOf for $($ident)::+ $(<$($generic),*>)? {
                #[inline]
                fn size_of_children(&self, _context: &mut $crate::Context) {}
            }
        )*
    };
}

// Implement SizeOf for tuples
macro_rules! impl_tuple {
    ($(($($elem:ident),* $(,)?)),* $(,)?) => {
        $(
            #[allow(non_snake_case)]
            impl<$($elem,)*> $crate::SizeOf for ($($elem,)*)
            where
                $($elem: $crate::SizeOf,)*
            {
                #[inline]
                #[allow(unused_variables)]
                fn size_of_children(&self, context: &mut $crate::Context) {
                    let ($($elem,)*) = self;
                    $($elem.size_of_children(context);)*
                }
            }
        )*
    };
}

// Implement SizeOf for a handful of function pointers (up to 16 args)
macro_rules! impl_function_ptrs {
    ($($cconv:literal),* $(,)?) => {
        $(
            impl_function_ptrs! {
                @inner $cconv
                (),
                (T1),
                (T1, T2),
                (T1, T2, T3),
                (T1, T2, T3, T4),
                (T1, T2, T3, T4, T5),
                (T1, T2, T3, T4, T5, T6),
                (T1, T2, T3, T4, T5, T6, T7),
                (T1, T2, T3, T4, T5, T6, T7, T8),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15),
                (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16),
            }
        )*
    };

    // Function pointers have no children, so their body is empty. *Technically* we could try
    // estimating the size of the literal code pointed to by the function but oh god, that's
    // a whole 'nother can of worms I ain't dealing with
    (@inner $cconv:literal $(($($ty:ident),* $(,)?)),* $(,)?) => {
        $(
            impl<$($ty,)* U> $crate::SizeOf for extern $cconv fn($($ty),*) -> U {
                #[inline]
                fn size_of_children(&self, _context: &mut $crate::Context) {}
            }
        )*
    };
}
