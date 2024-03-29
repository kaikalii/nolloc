//! A growable key-value map where all items exist on the stack

use core::{borrow::Borrow, fmt, ops::Index, ptr};

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
    /// Get the most recently inserted key-value pair in the map
    ///
    /// # Example
    /// ```
    /// use nolloc::Map;
    ///
    /// Map::collect([1, 2, 3, 4].iter().map(|&i| (i, i)), |map| {
    ///     assert_eq!(map.head(), Some((&4, &4)));
    /// });
    /// ```
    pub fn head(&self) -> Option<(&K, &V)> {
        let head = self.head?;
        Some((&head.key, &head.value))
    }
    /// Get all entries inserterd after the most recent one
    ///
    /// # Example
    /// ```
    /// use nolloc::Map;
    ///
    /// Map::collect([1, 2, 3, 4].iter().map(|&i| (i, i)), |map| {
    ///     assert!(!map.rest().contains_key(&4));
    /// });
    /// ```
    pub fn rest(&self) -> Self {
        let head = if let Some(head) = self.head {
            head
        } else {
            return Map::new();
        };
        match (head.left, head.right) {
            (None, None) => Map::new(),
            (None, Some(node)) | (Some(node), None) => Map {
                head: Some(node),
                len: self.len - 1,
            },
            (Some(left), Some(right)) => {
                let node = if left.contains_child(right) {
                    left
                } else {
                    right
                };
                Map {
                    head: Some(node),
                    len: self.len - 1,
                }
            }
        }
    }
    /// Get the key-value pair with the minimum key in the map
    ///
    /// This is an **O(logn)** operation.
    pub fn min(&self) -> Option<(&K, &V)> {
        let mut curr = self.head?;
        while let Some(left) = curr.left {
            curr = left;
        }
        Some((&curr.key, &curr.value))
    }
    /// Get the key-value pair with the maximum key in the map
    ///
    /// This is an **O(logn)** operation.
    pub fn max(&self) -> Option<(&K, &V)> {
        let mut curr = self.head?;
        while let Some(right) = curr.right {
            curr = right;
        }
        Some((&curr.key, &curr.value))
    }
}

impl<'a, K, V> Map<'a, K, V> {
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
        Q: PartialOrd + ?Sized,
    {
        Some(&self.get_node(key)?.value)
    }
    fn get_node<Q>(&self, key: &Q) -> Option<&'a MapNode<'a, K, V>>
    where
        K: Borrow<Q>,
        Q: PartialOrd + ?Sized,
    {
        let mut curr = self.head?;
        loop {
            let curr_key = curr.key.borrow();
            if key == curr_key {
                return Some(curr);
            } else if key < curr_key {
                curr = curr.left?;
            } else {
                curr = curr.right?;
            }
        }
    }
}

impl<'a, K, V> Map<'a, K, V>
where
    K: PartialOrd,
{
    /// Insert a key-value pair into the map if it does not already exist and
    /// call a continuation on the new (or old) map
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
    /// Insert a key-value pair into the map and call a continuation on the
    /// new map
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
    /// Get an iterator over the key-value pairs of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn iter(&self) -> Iter<'a, K, V> {
        Iter { node: self.head }
    }
    /// Get an iterator over the keys of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn keys(&self) -> Keys<'a, K, V> {
        Keys { iter: self.iter() }
    }
    /// Get an iterator over the values of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn values(&self) -> Values<'a, K, V> {
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
    /// Get a view into the entry at the given key
    pub fn entry(&'a self, key: K) -> Entry<'a, K, V> {
        Entry { key, map: self }
    }
}

/// An iterator over the key-value pairs of a [`Map`]
pub struct Iter<'a, K, V> {
    node: Option<&'a MapNode<'a, K, V>>,
}

impl<'a, K, V> MapNode<'a, K, V> {
    fn contains_child(&self, child: &Self) -> bool {
        self.left.map_or(false, |node| ptr::eq(node, child))
            || self.right.map_or(false, |node| ptr::eq(node, child))
            || self.left.map_or(false, |node| node.contains_child(child))
            || self.right.map_or(false, |node| node.contains_child(child))
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: PartialOrd,
{
    type Item = (&'a K, &'a V);
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
pub struct Keys<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V>
where
    K: PartialOrd,
{
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.0)
    }
}

/// An iterator over the values of a [`Map`]
pub struct Values<'a, K, V> {
    iter: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V>
where
    K: PartialOrd,
{
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.1)
    }
}

impl<'a, K, V> IntoIterator for &'a Map<'a, K, V>
where
    K: PartialOrd,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for Map<'a, K, V>
where
    K: PartialOrd,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
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

/// A view into a single entry in a [`Map`]
#[derive(Debug)]
pub struct Entry<'a, K, V>
where
    K: PartialOrd,
{
    key: K,
    map: &'a Map<'a, K, V>,
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: PartialOrd,
{
    /// Get the key associated with the entry
    pub fn key(&self) -> &K {
        if let Some(node) = self.map.get_node(&self.key) {
            &node.key
        } else {
            &self.key
        }
    }
    /// Insert a value if the entry does not already exist in the map
    /// and call a continuation
    ///
    /// # Example
    /// ```
    /// use nolloc::Map;
    ///
    /// Map::new().entry("poneyland").or_insert(3, |map, _| {
    ///     assert_eq!(map["poneyland"], 3);
    ///     let x = map.entry("poneyland").or_insert(10, |_, v| *v * 2);
    ///     assert_eq!(x, 6);
    /// });
    /// ```
    pub fn or_insert<F, R>(self, value: V, then: F) -> R
    where
        F: FnOnce(&Map<K, V>, &V) -> R,
    {
        if let Some(value) = self.map.get(&self.key) {
            then(self.map, value)
        } else {
            self.map
                .insert(self.key, value, |map| then(map, &map.head.unwrap().value))
        }
    }
    /// Insert a value if the entry does not already exist in the map
    /// and call a continuation
    pub fn or_insert_with<F, R, G>(self, get_value: G, then: F) -> R
    where
        F: FnOnce(&Map<K, V>, &V) -> R,
        G: FnOnce() -> V,
    {
        if let Some(value) = self.map.get(&self.key) {
            then(self.map, value)
        } else {
            self.map.insert(self.key, get_value(), |map| {
                then(map, &map.head.unwrap().value)
            })
        }
    }
    /// Insert a value if the entry does not already exist in the map
    /// and call a continuation
    pub fn or_insert_with_key<F, R, G>(self, get_value: G, then: F) -> R
    where
        F: FnOnce(&Map<K, V>, &V) -> R,
        G: FnOnce(&K) -> V,
    {
        if let Some(value) = self.map.get(&self.key) {
            then(self.map, value)
        } else {
            let value = get_value(&self.key);
            self.map
                .insert(self.key, value, |map| then(map, &map.head.unwrap().value))
        }
    }
    /// Insert the default value if the entry does not already exist in the map
    /// and call a continuation
    pub fn of_default<F, R, G>(self, then: F) -> R
    where
        F: FnOnce(&Map<K, V>, &V) -> R,
        V: Default,
    {
        self.or_insert_with(Default::default, then)
    }
    /// Insert a value even if the entry already exists and call a continuation
    pub fn insert<F, R>(self, value: V, then: F) -> R
    where
        F: FnOnce(&Map<K, V>, &V) -> R,
    {
        self.map
            .insert(self.key, value, |map| then(map, &map.head.unwrap().value))
    }
}

/// Map indexing is an **O(logn)** operation
impl<'a, K, V, Q> Index<&Q> for Map<'a, K, V>
where
    K: Borrow<Q>,
    Q: PartialOrd + ?Sized,
{
    type Output = V;
    #[track_caller]
    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}
