use std::cmp::Ordering;

use iter_merge::Merged;

/// Wrapper for an item and its iterator index
/// Has the same ordering as the item
#[derive(Debug)]
struct LabeledItem<T> {
    item: T,
    iter_idx: usize,
}

impl<T: Ord> Ord for LabeledItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item.cmp(&other.item)
    }
}

impl<T: PartialOrd> PartialOrd for LabeledItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.item.partial_cmp(&other.item)
    }
}

impl<T: PartialEq> PartialEq for LabeledItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T> Eq for LabeledItem<T> where T: Eq {}

struct MergeChecker<'a, T> {
    items: Vec<&'a [T]>,
    orig: &'a Vec<Vec<T>>,
}

impl<'a, T> MergeChecker<'a, T>
where
    T: core::fmt::Debug + Ord,
{
    fn new(items: &'a Vec<Vec<T>>) -> Self {
        Self {
            orig: items,
            items: Vec::new(),
        }
    }

    fn check_merge(&mut self, merge: impl IntoIterator<Item = LabeledItem<T>>, stable: bool) {
        // reset:
        self.items.clear();
        self.items
            .extend(self.orig.iter().map(AsRef::<[T]>::as_ref));
        merge
            .into_iter()
            .for_each(|choice| self.check_choice(&choice, stable));

        assert!(
            self.items.iter().all(|it| it.is_empty()),
            "Some items are not consumed"
        );
    }

    fn check_choice(&mut self, choice: &LabeledItem<T>, stable: bool) {
        for (iter_idx, item) in self
            .items
            .iter()
            .enumerate()
            .filter_map(|(iter_idx, items)| items.first().map(|item| (iter_idx, item)))
        {
            match item.cmp(&choice.item) {
                Ordering::Less => {
                    panic!(
                        "chosen item {choice:?} is less than item {item:?} from iterator {iter_idx}",
                    )
                }
                Ordering::Equal if stable => {
                    assert!(
                        iter_idx >= choice.iter_idx,
                        "item from earlier iterator {iter_idx} should've been chosen instead of {choice:?}"
                    );
                }
                _ => {}
            }
        }
        let Some((item, rest)) = self.items[choice.iter_idx].split_first() else {
            panic!("item was consumed from empty iterator {}", choice.iter_idx);
        };

        assert_eq!(item, &choice.item);
        self.items[choice.iter_idx] = rest;
    }
}

pub(crate) fn test_all_merges<T>(input: &Vec<Vec<T>>)
where
    T: Ord + core::fmt::Debug + Copy,
{
    let mkiter = || {
        input.iter().enumerate().map(|(iter_idx, items)| {
            items
                .iter()
                .copied()
                .map(move |item| LabeledItem { item, iter_idx })
        })
    };

    let mut checker = MergeChecker::new(input);

    checker.check_merge(Merged::new(mkiter()).build(), true);
    checker.check_merge(
        Merged::new(mkiter()).arbitrary_tie_breaking().build(),
        false,
    );
    checker.check_merge(Merged::new(mkiter()).build().into_vec(), true);
    checker.check_merge(
        Merged::new(mkiter())
            .arbitrary_tie_breaking()
            .build()
            .into_vec(),
        false,
    );
}
