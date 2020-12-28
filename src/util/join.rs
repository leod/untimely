use std::{cmp::Ordering, iter::Peekable};

/// Produce an iterator over the full join of two sorted iterators over
/// key-value pairs.
///
/// Full join here means that elements from both iterators are returned,
/// whether they match by key or not.
///
/// The iterators are assumed to be sorted by the key in ascending order.
///
/// # Arguments
///
/// * `left` - The left iterator, producing values of type `(K, T)`
/// * `right` - The right iterator, producing values of type `(K, U)`
///
/// # Returns
///
/// An iterator over the full join with items of type [`Item<K, T, U>`](Item).
/// The returned items are sorted by the key. There are three cases:
///
/// 1. If a key is contained only in the left iterator, [`Item::Left`] is
///    returned.
/// 2. If a key is contained only in the right iterator, [`Item::Right`] is
///    returned.
/// 3. If a key is contained in both iterators, [`Item::Both`] is returned.
///
/// # Examples
///
/// ```
/// use untimely::util::join::{full_join, Item};
///
/// let left = vec![(1, "hello"), (3, "foo"), (5, "rust")];
/// let right = vec![(1, "world"), (4, "bar"), (5, "rocks")];
/// let result: Vec<_> = full_join(left, right).collect();
///
/// let expected = vec![
///     Item::Both(1, "hello", "world"),
///     Item::Left(3, "foo"),
///     Item::Right(4, "bar"),
///     Item::Both(5, "rust", "rocks"),
/// ];
/// assert_eq!(result, expected);
/// ```
pub fn full_join<Left, Right, K, T, U>(
    left: Left,
    right: Right,
) -> FullJoinIter<Left::IntoIter, Right::IntoIter>
where
    Left: IntoIterator<Item = (K, T)>,
    Right: IntoIterator<Item = (K, U)>,
    K: Ord,
{
    FullJoinIter {
        left: left.into_iter().peekable(),
        right: right.into_iter().peekable(),
    }
}

/// Element of a [full join](full_join) between two sorted iterators over
/// key-value pairs ("left" and "right" iterator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item<K, T, U> {
    /// The key `K` is only contained in the left iterator.
    ///
    /// The first element contains the key, and the second element contains the
    /// corresponding element `T` in the left iterator.
    Left(K, T),

    /// The key `K` is only contained in the right iterator.
    ///
    /// The first element contains the key, and the second element contains the
    /// corresponding element `U` in the right iterator.
    Right(K, U),

    /// The key `K` is contained in both iterators.
    ///
    /// The first element contains the key, and the second and third elements
    /// contain the corresponding elements `T` and `U` in the left and right
    /// iterators.
    Both(K, T, U),
}

/// Iterator over the [full join](full_join) of two sorted iterators of
/// key-value pairs.
///
/// The iterators are assumed to be sorted by the key in ascending order.
pub struct FullJoinIter<Left, Right>
where
    Left: Iterator,
    Right: Iterator,
{
    left: Peekable<Left>,
    right: Peekable<Right>,
}

impl<Left, Right, K, T, U> Iterator for FullJoinIter<Left, Right>
where
    Left: Iterator<Item = (K, T)>,
    Right: Iterator<Item = (K, U)>,
    K: Ord,
{
    type Item = Item<K, T, U>;

    fn next(&mut self) -> Option<Self::Item> {
        // Advance the iterator which has the element with the smaller key.
        match (self.left.peek(), self.right.peek()) {
            (Some(_), None) => {
                // The right iterator is finished.
                let (left_k, left_v) = self.left.next().unwrap();
                Some(Item::Left(left_k, left_v))
            }
            (None, Some(_)) => {
                // The left iterator is finished.
                let (right_k, right_v) = self.right.next().unwrap();
                Some(Item::Right(right_k, right_v))
            }
            (Some((left_k, _)), Some((right_k, _))) => Some(match left_k.cmp(&right_k) {
                Ordering::Less => {
                    let (left_k, left_v) = self.left.next().unwrap();
                    Item::Left(left_k, left_v)
                }
                Ordering::Greater => {
                    let (right_k, right_v) = self.right.next().unwrap();
                    Item::Right(right_k, right_v)
                }
                Ordering::Equal => {
                    let (left_k, left_v) = self.left.next().unwrap();
                    let (right_k, right_v) = self.right.next().unwrap();
                    assert!(left_k == right_k);
                    Item::Both(left_k, left_v, right_v)
                }
            }),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{full_join, Item};

    #[test]
    fn test_both_empty() {
        let left: Vec<(usize, ())> = Vec::new();
        let right: Vec<(usize, String)> = Vec::new();
        let result: Vec<_> = full_join(left, right).collect();
        assert!(result.is_empty());
    }

    #[test]
    fn test_left_empty() {
        let left: Vec<(usize, ())> = Vec::new();
        let right: Vec<(usize, &str)> = vec![(1, "foo"), (400, "bar")];
        let result: Vec<_> = full_join(left, right).collect();
        let expected = vec![Item::Right(1, "foo"), Item::Right(400, "bar")];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_right_empty() {
        let left: Vec<(usize, &str)> = vec![(1, "foo"), (400, "bar")];
        let right: Vec<(usize, ())> = Vec::new();
        let result: Vec<_> = full_join(left, right).collect();
        let expected = vec![Item::Left(1, "foo"), Item::Left(400, "bar")];
        assert_eq!(result, expected);
    }
}
