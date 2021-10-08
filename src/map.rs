//! A growable key-value map where all items exist on the stack

use core::{borrow::Borrow, fmt, ptr};

/// A growable key-value map where all items exist on the stack
pub struct Map<'a, K, V> {
    head: Option<&'a MapNode<'a, K, V>>,
    len: usize,
}

struct MapNode<'a, K, V> {
    key: K,
    value: V,
    left: Option<&'a Self>,
    right: Option<&'a Self>,
}

impl<'a, K, V> Map<'a, K, V>
where
    K: PartialOrd,
{
    /// Create a new map
    pub fn new() -> Self {
        Map::default()
    }
    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Get the map's length
    ///
    /// This is an **O(1)** operation.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Check if the map contains a key
    ///
    /// This is an **O(logn)** operation.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: PartialOrd,
    {
        self.get(key).is_some()
    }
    /// Get the value corresponding to the key
    ///
    /// This is an **O(logn)** operation.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: PartialOrd,
    {
        let mut curr = self.head?;
        loop {
            let curr_key = curr.key.borrow();
            if key == curr_key {
                return Some(&curr.value);
            } else if key < curr_key {
                curr = curr.left?;
            } else {
                curr = curr.right?;
            }
        }
    }
    /// Insert a key/value pair into the map if it does not already exist,
    /// call a continuation on the new (or old) map, and return its result
    ///
    /// This is an **O(logn)** operation.
    pub fn try_insert<F, R>(&self, key: K, value: V, then: F) -> R
    where
        F: FnOnce(&Map<K, V>) -> R,
    {
        if self.contains_key(&key) {
            then(self)
        } else {
            self.insert(key, value, then)
        }
    }
    /// Insert a key/value pair into the map, call a continuation on the
    /// new map, and return its result
    ///
    /// If an entry with the key already exists in the map, it is not removed,
    /// but the new entry is still inserted. All lookups on the new map
    /// will find the most recently inserted entry for a key.
    ///
    /// This is an **O(logn)** operation.
    pub fn insert<F, R>(&self, key: K, value: V, then: F) -> R
    where
        F: FnOnce(&Map<K, V>) -> R,
    {
        let mut node = MapNode {
            key,
            value,
            left: None,
            right: None,
        };
        if let Some(head) = self.head {
            if node.key < head.key {
                node.right = Some(head);
                let mut curr = head;
                while node.key < curr.key {
                    curr = if let Some(next) = curr.left.or(curr.right) {
                        next
                    } else {
                        break;
                    }
                }
                if node.key > curr.key {
                    node.left = Some(curr);
                }
            } else {
                node.left = Some(head);
                let mut curr = head;
                while node.key >= curr.key {
                    curr = if let Some(next) = curr.right.or(curr.left) {
                        next
                    } else {
                        break;
                    }
                }
                if node.key < curr.key {
                    node.right = Some(curr);
                }
            }
        }
        then(&Map {
            head: Some(&node),
            len: self.len + 1,
        })
    }
    /// Get an iterator over the key/value pairs of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn iter<'m>(&'m self) -> Iter<'a, 'm, K, V> {
        Iter { node: self.head }
    }
    /// Get an iterator over the keys of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn keys<'m>(&'m self) -> Keys<'a, 'm, K, V> {
        Keys { iter: self.iter() }
    }
    /// Get an iterator over the values of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn values<'m>(&'m self) -> Values<'a, 'm, K, V> {
        Values { iter: self.iter() }
    }
    /// Collect an iterator into a map and call a continuation function on it
    ///
    /// # Example
    /// ```
    /// use nolloc::Map;
    ///
    /// let nums = [(1, 2), (3, 4), (5, 6)];
    ///
    /// let sum = Map::collect(nums, |map| {
    ///     let mut sum = 0;
    ///     for (k, _) in &nums {
    ///         sum += *map.get(k).unwrap();
    ///     }
    ///     sum
    /// });
    ///
    /// assert_eq!(sum, 12);
    /// ```
    pub fn collect<I, F, R>(iter: I, then: F) -> R
    where
        K: PartialOrd,
        I: IntoIterator<Item = (K, V)>,
        F: FnOnce(&Map<K, V>) -> R,
    {
        Map::default().extend(iter, then)
    }
    /// Extend the map with an iterator and call a continuation function on it
    pub fn extend<I, F, R>(&self, iter: I, then: F) -> R
    where
        K: PartialOrd,
        I: IntoIterator<Item = (K, V)>,
        F: FnOnce(&Map<K, V>) -> R,
    {
        let mut iter = iter.into_iter();
        if let Some((k, v)) = iter.next() {
            self.insert(k, v, |list| list.extend(iter, then))
        } else {
            then(self)
        }
    }
}

/// An iterator over the key/value pairs of a [`Map`]
pub struct Iter<'a, 'm, K, V> {
    node: Option<&'m MapNode<'a, K, V>>,
}

impl<'a, K, V> MapNode<'a, K, V> {
    fn contains_child(&self, child: &Self) -> bool {
        self.left.map_or(false, |node| ptr::eq(node, child))
            || self.right.map_or(false, |node| ptr::eq(node, child))
            || self.left.map_or(false, |node| node.contains_child(child))
            || self.right.map_or(false, |node| node.contains_child(child))
    }
}

impl<'a, 'm, K, V> Iterator for Iter<'a, 'm, K, V>
where
    K: PartialOrd,
{
    type Item = (&'m K, &'m V);
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node?;
        let res = (&node.key, &node.value);
        self.node = match (node.left, node.right) {
            (None, None) => None,
            (None, Some(right)) => Some(right),
            (Some(left), None) => Some(left),
            (Some(left), Some(right)) => Some(if left.contains_child(right) {
                left
            } else {
                right
            }),
        };
        Some(res)
    }
}

/// An iterator over the keys of a [`Map`]
pub struct Keys<'a, 'm, K, V> {
    iter: Iter<'a, 'm, K, V>,
}

impl<'a, 'm, K, V> Iterator for Keys<'a, 'm, K, V>
where
    K: PartialOrd,
{
    type Item = &'m K;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.0)
    }
}

/// An iterator over the values of a [`Map`]
pub struct Values<'a, 'm, K, V> {
    iter: Iter<'a, 'm, K, V>,
}

impl<'a, 'm, K, V> Iterator for Values<'a, 'm, K, V>
where
    K: PartialOrd,
{
    type Item = &'m V;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.1)
    }
}

impl<'a, 'm, K, V> IntoIterator for &'m Map<'a, K, V>
where
    K: PartialOrd,
{
    type Item = (&'m K, &'m V);
    type IntoIter = Iter<'a, 'm, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> Default for Map<'a, K, V> {
    fn default() -> Self {
        Map { head: None, len: 0 }
    }
}

impl<'a, K, V> Clone for Map<'a, K, V> {
    fn clone(&self) -> Self {
        Map {
            head: self.head,
            len: self.len,
        }
    }
}

impl<'a, K, V> Copy for Map<'a, K, V> {}

impl<'a, K, V> PartialEq for Map<'a, K, V>
where
    K: PartialOrd,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        for (key, value) in self {
            if !other.get(key).map_or(false, |other_val| value == other_val) {
                return false;
            }
        }
        for (key, value) in other {
            if !self.get(key).map_or(false, |other_val| value == other_val) {
                return false;
            }
        }
        true
    }
}

impl<'a, K, V> Eq for Map<'a, K, V>
where
    K: PartialOrd + Eq,
    V: Eq,
{
}

impl<'a, K, V> fmt::Debug for Map<'a, K, V>
where
    K: PartialOrd + fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
