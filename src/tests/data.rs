pub(crate) type TestItemType = i8;

const ALL_TEST_VECTORS: &[&[&[TestItemType]]] = &[
    &[&[]],
    &[&[], &[], &[], &[], &[]],
    &[&[], &[], &[42], &[]],
    &[&[], &[1], &[], &[2]],
    &[&[5], &[4], &[3], &[2], &[1]],
    &[&[5, 4, 3], &[4], &[], &[3, 2], &[2, 1, 0], &[1, 0, -1]],
    &[
        &[1, 1, 1],
        &[3, 2, 1],
        &[1, 2, 3],
        &[2, 2],
        &[4, 2, 1],
        &[2, 1, 3],
        &[4, 1, 1],
        &[2, 3, 4],
        &[4, 2, 1],
        &[4, 4, 4],
    ],
    &[], // This (inclusive) represents a cut-off for miri tests
    &[&[1, 2, 3], &[1, 2, 3]],
    &[&[1, 3, 5], &[2, 4, 6]],
    &[&[1, 1, 1, 1], &[2, 2, 2, 2], &[1, 2, 1, 2]],
    &[&[1, 2], &[1, 2, 3, 4]],
    &[&[1], &[2], &[3], &[4]],
    &[&[1, 2, 3, 4, 5, 6, 7, 8], &[0], &[9]],
    &[&[10, 8, 6, 4, 2], &[1, 3, 5, 7, 9]],
    &[&[-3, -1, 0, 2], &[1, 2, 3], &[-2, 4]],
    &[&[1, 2, 2, 3], &[2, 2, 3, 4], &[2, 5]],
    &[&[1, 3, 5, 7], &[2, 4, 6, 8], &[0, 9]],
    &[&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]],
    &[&[1, 4, 7], &[2, 5, 8], &[3, 6, 9]],
    &[&[5, 5, 5], &[5, 5], &[5]],
    &[&[1, 100, 127], &[50, 127], &[75, 127]],
    &[
        &[1, 2, 3, 4, 5],
        &[6, 7, 8, 9, 10],
        &[11, 12, 13, 14, 15],
        &[16, 17, 18, 19, 20],
        &[21, 22, 23, 24, 25],
        &[26, 27, 28, 29, 30],
        &[31, 32, 33, 34, 35],
        &[36, 37, 38, 39, 40],
        &[41, 42, 43, 44, 45],
        &[46, 47, 48, 49, 50],
    ],
    &[
        &[1, 3, 5, 7, 9],
        &[2, 4, 6, 8, 10],
        &[11, 13, 15, 17, 19],
        &[12, 14, 16, 18, 20],
        &[21, 23, 25, 27, 29],
        &[22, 24, 26, 28, 30],
        &[31, 33, 35, 37, 39],
        &[32, 34, 36, 38, 40],
        &[41, 43, 45, 47, 49],
        &[42, 44, 46, 48, 50],
    ],
    &[
        &[10, 20, 30],
        &[15, 25, 35],
        &[12, 22, 32],
        &[17, 27, 37],
        &[14, 24, 34],
        &[19, 29, 39],
        &[16, 26, 36],
        &[11, 21, 31],
        &[18, 28, 38],
        &[13, 23, 33],
    ],
    &[&[1], &[2], &[3], &[4], &[5], &[6], &[7], &[8], &[9], &[10]],
    &[
        &[1, 2],
        &[3, 4],
        &[5, 6],
        &[7, 8],
        &[9, 10],
        &[11, 12],
        &[13, 14],
        &[15, 16],
        &[17, 18],
        &[19, 20],
    ],
    &[
        &[1, 1, 1],
        &[2, 2, 2],
        &[3, 3, 3],
        &[4, 4, 4],
        &[5, 5, 5],
        &[6, 6, 6],
        &[7, 7, 7],
        &[8, 8, 8],
        &[9, 9, 9],
        &[10, 10, 10],
    ],
    &[
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        &[2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
        &[1, 3, 5, 7, 9, 11, 13, 15, 17, 19],
        &[5, 10, 15, 20, 25, 30, 35, 40, 45, 50],
        &[50, 45, 40, 35, 30, 25, 20, 15, 10, 5],
        &[0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        &[1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
        &[3, 6, 9, 12, 15, 18, 21, 24, 27, 30],
        &[30, 27, 24, 21, 18, 15, 12, 9, 6, 3],
    ],
    &[
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        &[2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
        &[1, 3, 5, 7, 9, 11, 13, 15, 17, 19],
        &[5, 10, 15, 20, 25, 30, 35, 40, 45, 50],
        &[50, 45, 40, 35, 30, 25, 20, 15, 10, 5],
        &[0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        &[1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
        &[3, 6, 9, 12, 15, 18, 21, 24, 27, 30],
        &[30, 27, 24, 21, 18, 15, 12, 9, 6, 3],
    ],
];

pub(crate) const TEST_VECTORS: &[&[&[TestItemType]]] = {
    if cfg!(miri) {
        // Run miri on a smaller number of tests
        let mut i = 0;
        while !ALL_TEST_VECTORS[i].is_empty() {
            i += 1;
        }
        ALL_TEST_VECTORS.split_at(i + 1).0
    } else {
        ALL_TEST_VECTORS
    }
};

pub(crate) const MAX_TEST_VEC_LEN: usize = {
    let mut val = 0;
    let mut i = 0;
    while i < TEST_VECTORS.len() {
        let len = TEST_VECTORS[i].len();
        if val < len {
            val = len;
        }
        i += 1;
    }
    val
};

#[cfg(miri)]
pub(crate) type MaybeBoxed<T> = crate::alloc::boxed::Box<T>;
#[cfg(not(miri))]
pub(crate) type MaybeBoxed<T> = T;

#[inline(always)]
pub(crate) fn maybe_box<T>(item: T) -> MaybeBoxed<T> {
    // If we're running under miri - box our values to enable
    // better leak checks
    #[cfg(miri)]
    return crate::alloc::boxed::Box::new(item);
    #[cfg(not(miri))]
    return item;
}

pub trait TestData: UnwindSafe + RefUnwindSafe {
    type Item: Ord + Clone + UnwindSafe + RefUnwindSafe + Debug;
    fn as_iters(&self) -> impl Iterator<Item = impl Iterator<Item = Self::Item> + Clone>;
    fn length(&self) -> usize;
    fn item_count(&self) -> usize;
}

impl<'a, T> TestData for &'a [&'a [T]]
where
    T: Ord + Copy + UnwindSafe + RefUnwindSafe + Debug,
{
    type Item = MaybeBoxed<T>;

    fn as_iters(&self) -> impl Iterator<Item = impl Iterator<Item = Self::Item> + Clone> {
        self.iter()
            .map(|iter| maybe_box(iter.iter().map(|item| maybe_box(*item))))
    }
    fn length(&self) -> usize {
        self.len()
    }
    fn item_count(&self) -> usize {
        self.iter().map(|it| it.len()).sum()
    }
}

use core::{
    fmt::Debug,
    panic::{RefUnwindSafe, UnwindSafe},
};

use super::std::vec::Vec;
impl<T> TestData for Vec<Vec<T>>
where
    T: Ord + Copy + UnwindSafe + RefUnwindSafe + Debug,
{
    type Item = MaybeBoxed<T>;

    fn as_iters(&self) -> impl Iterator<Item = impl Iterator<Item = Self::Item> + Clone> {
        self.iter()
            .map(|iter| maybe_box(iter.iter().map(|item| maybe_box(*item))))
    }

    fn length(&self) -> usize {
        self.len()
    }
    fn item_count(&self) -> usize {
        self.iter().map(Vec::len).sum()
    }
}
