macro_rules! gen_unchecked_ops {
    ($([$unchecked_name:ident, $checked_name:ident]),*) => {
        $(
            #[rustversion::since(1.79)]
            #[inline]
            /// # Safety
            #[doc = concat!("See [usize::", stringify!($unchecked_name), "]")]
            pub(crate) const unsafe fn $unchecked_name(a: usize, b: usize) -> usize {
                // SAFETY: the safety contract must be upheld by the caller.
                unsafe { a.$unchecked_name(b) }
            }

            #[rustversion::before(1.79)]
            #[inline]
            /// # Safety
            #[doc = concat!("See [usize::", stringify!($unchecked_name), "]")]
            pub(crate) const unsafe fn $unchecked_name(a: usize, b: usize) -> usize {
                match a.$checked_name(b) {
                    Some(val) => val,
                    // SAFETY: the safety contract must be upheld by the caller.
                    None => unsafe { core::hint::unreachable_unchecked() },
                }
            }
        )*
    };
}

gen_unchecked_ops!(
    [unchecked_add, checked_add],
    [unchecked_sub, checked_sub],
    [unchecked_mul, checked_mul]
);
