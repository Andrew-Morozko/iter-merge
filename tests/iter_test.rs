#![cfg(feature = "vec_storage")]

use iter_merge::Merged;

mod helpers;
use helpers::test_all_merges;

#[test]
fn test_all_merge_configurations() {
    [
        // empty inputs
        vec![],
        // single empty iterator
        vec![vec![]],
        // multiple empty iterators
        vec![vec![], vec![], vec![]],
        // single element with empty iterators
        vec![vec![], vec![], vec![1]],
        // multiple single elements
        vec![vec![], vec![1], vec![2], vec![3], vec![]],
        // single element
        vec![vec![1]],
        // basic merging
        vec![vec![1, 3, 5], vec![2, 4, 6]],
        // unsorted single iterator
        vec![vec![1, 0, -1]],
        // unsorted multiple iterators
        vec![vec![1, 0, -1], vec![1, 0, 1], vec![0, 1, 0]],
        // duplicate values
        vec![vec![0, 0, -1], vec![0, 0, 1], vec![0, 0, 0]],
        // mixed lengths
        vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], vec![0], vec![11], vec![2, 3]],
        // negative numbers
        vec![vec![-5, -3, -1], vec![-4, -2, 0]],
        // tie breaking tests
        vec![vec![0], vec![0], vec![-1, 0]],
        vec![vec![1, 1, 2], vec![1, 3]],
        vec![vec![2, 1, 2], vec![1], vec![1, 2, 1], vec![2], vec![1, 2]],
        // identical iterators
        vec![vec![1, 2, 3], vec![1, 2, 3], vec![1, 2, 3]],
    ].iter().for_each(test_all_merges);
}

#[test]
fn test_peek_basic() {
    let mut merged = Merged::new([[1, 3, 5], [2, 4, 6]]).build();

    assert_eq!(merged.peek(), Some(&1));
    assert_eq!(merged.next(), Some(1));

    assert_eq!(merged.peek(), Some(&2));
    assert_eq!(merged.next(), Some(2));

    assert_eq!(merged.peek(), Some(&3));
    assert_eq!(merged.next(), Some(3));
}

#[test]
fn test_peek_empty() {
    let mut merged = Merged::new(Vec::<Vec<i32>>::new()).build();
    assert_eq!(merged.peek(), None);
    assert_eq!(merged.next(), None);
}

#[test]
fn test_peek_after_consumption() {
    let mut merged = Merged::new([[1, 2], [3, 4]]).build();

    // Consume all elements
    assert_eq!(merged.next(), Some(1));
    assert_eq!(merged.next(), Some(2));
    assert_eq!(merged.next(), Some(3));
    assert_eq!(merged.next(), Some(4));

    // Should be empty now
    assert_eq!(merged.peek(), None);
    assert_eq!(merged.next(), None);
}

#[test]
fn test_peek_with_duplicates() {
    let mut merged = Merged::new([
        [1, 1, 2],
        [1, 3, 3]
    ]).build();

    assert_eq!(merged.peek(), Some(&1));
    assert_eq!(merged.next(), Some(1));

    assert_eq!(merged.peek(), Some(&1));
    assert_eq!(merged.next(), Some(1));

    assert_eq!(merged.peek(), Some(&1));
    assert_eq!(merged.next(), Some(1));

    assert_eq!(merged.peek(), Some(&2));
    assert_eq!(merged.next(), Some(2));
}

#[test]
fn test_next_if_basic() {
    let mut merged = Merged::new(vec![
        vec![1, 1, 2, 3],
        vec![1, 4],
    ]).build();

    // Consume all 1s
    assert_eq!(merged.next_if(|&x| x == 1), Some(1));
    assert_eq!(merged.next_if(|&x| x == 1), Some(1));
    assert_eq!(merged.next_if(|&x| x == 1), Some(1));

    // Should not consume 2
    assert_eq!(merged.next_if(|&x| x == 1), None);
    assert_eq!(merged.next(), Some(2));
}

#[test]
fn test_next_if_predicate() {
    let mut merged = Merged::new(vec![
        vec![1, 2, 3, 4],
        vec![2, 5],
    ]).build();

    // The first element is 1 (from first iterator), which is odd
    assert_eq!(merged.next_if(|&x| x % 2 == 0), None);
    assert_eq!(merged.next(), Some(1));

    // Now the next element is 2 (from second iterator), which is even
    assert_eq!(merged.next_if(|&x| x % 2 == 0), Some(2));
}

#[test]
fn test_next_if_empty() {
    let mut merged = Merged::new(Vec::<Vec<i32>>::new()).build();
    assert_eq!(merged.next_if(|&x| x == 1), None);
}

#[test]
fn test_next_if_after_consumption() {
    let mut merged = Merged::new(vec![
        vec![1, 2],
        vec![3],
    ]).build();

    // Consume all elements
    assert_eq!(merged.next(), Some(1));
    assert_eq!(merged.next(), Some(2));
    assert_eq!(merged.next(), Some(3));

    // Should be empty
    assert_eq!(merged.next_if(|&x| x == 1), None);
}

#[test]
fn test_next_if_eq_basic() {
    let mut merged = Merged::new(vec![
        vec![1, 1, 2, 3],
        vec![1, 4],
    ]).build();

    // Consume all 1s
    assert_eq!(merged.next_if_eq(&1), Some(1));
    assert_eq!(merged.next_if_eq(&1), Some(1));
    assert_eq!(merged.next_if_eq(&1), Some(1));

    // Should not consume 2
    assert_eq!(merged.next_if_eq(&1), None);
    assert_eq!(merged.next(), Some(2));
}

#[test]
fn test_next_if_eq_different_types() {
    let mut merged = Merged::new(vec![
        vec![1, 2, 3],
        vec![2, 4],
    ]).build();

    // The first element is 1, not 2
    let target: i32 = 2;
    assert_eq!(merged.next_if_eq(&target), None);
    assert_eq!(merged.next(), Some(1));

    // Now the next element is 2
    assert_eq!(merged.next_if_eq(&target), Some(2));
}

#[test]
fn test_next_if_eq_empty() {
    let mut merged = Merged::new(Vec::<Vec<i32>>::new()).build();
    assert_eq!(merged.next_if_eq(&1), None);
}

#[test]
fn test_break_up_basic() {
    let iter1 = vec![1, 3];
    let iter2 = vec![2, 4];
    let mut merged = Merged::new(vec![iter1, iter2]).build();

    assert_eq!(merged.next(), Some(1));

    let mut storage = merged.break_up();

    // Check the remaining elements
    assert_eq!(storage[0].0, 3); // First iterator's peeked value
    assert_eq!(storage[0].1.next(), None); // First iterator is exhausted

    assert_eq!(storage[1].0, 2); // Second iterator's peeked value
    assert_eq!(storage[1].1.next(), Some(4)); // Second iterator has more elements
    assert_eq!(storage[1].1.next(), None);
}

#[test]
fn test_break_up_empty() {
    let merged = Merged::new(Vec::<Vec<i32>>::new()).build();
    let storage = merged.break_up();
    assert_eq!(storage.len(), 0);
}

#[test]
fn test_break_up_after_full_consumption() {
    let iter1 = vec![1];
    let iter2 = vec![2];
    let mut merged = Merged::new(vec![iter1, iter2]).build();

    // Consume all elements
    assert_eq!(merged.next(), Some(1));
    assert_eq!(merged.next(), Some(2));

    let storage = merged.break_up();
    assert_eq!(storage.len(), 0); // All iterators should be exhausted
}



#[test]
fn test_size_hint() {
    let merged = Merged::new(Vec::<Vec<i32>>::new()).build();
    assert_eq!(merged.size_hint(), (0, Some(0)));

    let merged = Merged::new(vec![
        vec![1, 2, 3],
        vec![4, 5],
    ]).build();

    assert_eq!(merged.size_hint(), (5, Some(5)));

    let iter = (0..3).filter(|&e| e < 3);
    let merged = Merged::new([iter]).build();
    assert_eq!(merged.size_hint(), (1, Some(3)));

    let iter = (0..).filter(|&e| e > 0);
    let merged = Merged::new([iter]).build();
    assert_eq!(merged.size_hint(), (1, None));

    let iter = 0..5;
    let merged = Merged::new([iter]).build();
    assert_eq!(merged.size_hint(), (5, Some(5)));


    let iter = (0..5).chain((0..).filter(|_| true));
    let merged = Merged::new([iter]).build();
    assert_eq!(merged.size_hint(), (5, None));

    // Size hint overflow
    let merged = Merged::new([0..usize::MAX, 0..usize::MAX]).build();
    assert_eq!(merged.size_hint(), (usize::MAX, None));
}

#[test]
fn test_add_iter() {
    let mut merged = Merged::new([vec![1, 3, 5]]).build();
    merged.add_iter(vec![2, 4, 6]);
    assert_eq!(merged.into_vec(), vec![1, 2, 3, 4, 5, 6]);
    merged.add_iter(vec![7, 8, 9]);
    assert_eq!(merged.into_vec(), vec![7, 8, 9]);
    merged.add_iter(vec![]);
    assert_eq!(merged.into_vec(), vec![]);
    merged.add_iter(vec![11, 14, 17]);
    merged.add_iter(vec![12, 15, 18]);
    merged.add_iter(vec![10, 13, 16]);
    assert_eq!(merged.into_vec(), vec![10, 11, 12, 13, 14, 15, 16, 17, 18]);
}

#[test]
fn test_add_iters() {
    let mut merged = Merged::new([vec![7, 8, 9]]).build();
    merged.add_iters([vec![2, 4, 6], vec![1, 3, 5]]);
    assert_eq!(merged.into_vec(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
}