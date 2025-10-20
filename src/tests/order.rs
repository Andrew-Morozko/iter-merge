use alloc::vec::Vec;
use core::fmt::Debug;

use crate::{VecStorage, comparators::Comparator, internal::PeekIter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TaggedItem<T> {
    item: T,
    iter_idx: usize,
}

#[derive(Clone)]
struct TaggedItemComparator<B>(B);
impl<T, B> Comparator<TaggedItem<T>> for TaggedItemComparator<B>
where
    B: Comparator<T>,
{
    fn compare<'a>(&self, a: &'a TaggedItem<T>, b: &'a TaggedItem<T>) -> core::cmp::Ordering {
        self.0.compare(&a.item, &b.item)
    }
}
use super::data::TestData;
pub fn assert_correct_order<T: Clone + Ord + Debug>(
    data: &impl TestData<Item = T>, cmp: impl Comparator<T>, tb: impl Comparator<T>,
) {
    let mut merge = VecStorage::from_iter(
        data.as_iters()
            .enumerate()
            .map(|(iter_idx, items)| items.map(move |item| TaggedItem { item, iter_idx })),
    )
    .into_builder()
    .min_by(TaggedItemComparator(&cmp))
    .tie_breaker(TaggedItemComparator(&tb))
    .build();
    let result = merge.clone().into_vec();
    let mut result_it = result.iter();
    for (iter_it, res_it) in merge.by_ref().zip(&mut result_it) {
        assert_eq!(&iter_it, res_it);
    }
    assert!(result_it.next().is_none() && merge.next().is_none());
    // Both vec and merge iterator agree on the order of items; let's check vec's order for correctness
    let mut iters = data
        .as_iters()
        .map(PeekIter::new_from_iter)
        .collect::<Vec<_>>();
    for choice in result {
        let iter_idx = choice.iter_idx;
        let chosen_item_ref = &(iters[iter_idx].as_ref().expect("empty_iter").item);

        for iter in iters.iter().filter_map(|iter| iter.as_ref()) {
            assert!(
                cmp.compare(chosen_item_ref, &iter.item)
                    .then_with(|| tb.compare(chosen_item_ref, &iter.item))
                    .is_le()
            );
        }
        // Chosen iter was correct, make sure that the same item was yielded
        assert_eq!(&choice.item, chosen_item_ref);

        let chosen_iter = iters[iter_idx].as_mut().expect("empty iter");

        if chosen_iter.advance().is_none() {
            // iterator exhausted, replace with none
            iters[iter_idx] = None;
        }
    }
    assert!(iters.iter().all(Option::is_none));
}
