//! A growable set where all items exist on the stack

use core::{borrow::Borrow, fmt, ptr};

/// A growable set where all items exist on the stack
pub struct Set<'a, T> {
    head: Option<&'a SetNode<'a, T>>,
    len: usize,
}

struct SetNode<'a, T> {
    item: T,
    left: Option<&'a Self>,
    right: Option<&'a Self>,
}

impl<'a, T> Set<'a, T>
where
    T: PartialOrd,
{
    /// Create a new set
    pub fn new() -> Self {
        Set::default()
    }
    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Get the set's length
    ///
    /// This is an **O(1)** operation.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Get the most recently inserted item in the set
    ///
    /// # Example
    /// ```
    /// use nolloc::Set;
    ///
    /// Set::collect([1, 2, 3, 4], |set| {
    ///     assert_eq!(set.head(), Some(&4));
    /// });
    /// ```
    pub fn head(&self) -> Option<&T> {
        Some(&self.head?.item)
    }
    /// Get all items inserterd after the most recent one
    ///
    /// # Example
    /// ```
    /// use nolloc::Set;
    ///
    /// Set::collect([1, 2, 3, 4], |set| {
    ///     assert!(!set.rest().contains(&4));
    /// });
    /// ```
    pub fn rest(&self) -> Self {
        let head = if let Some(head) = self.head {
            head
        } else {
            return Set::new();
        };
        match (head.left, head.right) {
            (None, None) => Set::new(),
            (None, Some(node)) | (Some(node), None) => Set {
                head: Some(node),
                len: self.len - 1,
            },
            (Some(left), Some(right)) => {
                let node = if left.contains_child(right) {
                    left
                } else {
                    right
                };
                Set {
                    head: Some(node),
                    len: self.len - 1,
                }
            }
        }
    }
    /// Get the item with the minimum value in the set
    ///
    /// This is an **O(logn)** operation.
    ///
    /// # Example
    /// ```
    /// use nolloc::Set;
    ///
    /// Set::collect([4, 1, 5, 9, 2, 6, 9, 8, 3], |set| {
    ///     assert_eq!(set.min(), Some(&1))
    /// });
    /// ```
    pub fn min(&self) -> Option<&T> {
        let mut curr = self.head?;
        while let Some(left) = curr.left {
            curr = left;
        }
        Some(&curr.item)
    }
    /// Get the item with the maximum value in the set
    ///
    /// This is an **O(logn)** operation.
    ///
    /// # Example
    /// ```
    /// use nolloc::Set;
    ///
    /// Set::collect([4, 1, 5, 9, 2, 6, 9, 8, 3], |set| {
    ///     assert_eq!(set.max(), Some(&9))
    /// });
    /// ```
    pub fn max(&self) -> Option<&T> {
        let mut curr = self.head?;
        while let Some(right) = curr.right {
            curr = right;
        }
        Some(&curr.item)
    }
}

impl<'a, T> Set<'a, T> {
    /// Check if the set contains an item
    ///
    /// This is an **O(logn)** operation.
    pub fn contains<Q>(&self, item: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: PartialOrd,
    {
        self.get(item).is_some()
    }
    /// Get an item in that set that is equals to to the item
    ///
    /// This is an **O(logn)** operation.
    pub fn get<Q>(&self, item: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: PartialOrd,
    {
        let mut curr = self.head?;
        loop {
            let curr_item = curr.item.borrow();
            if item == curr_item {
                return Some(&curr.item);
            } else if item < curr_item {
                curr = curr.left?;
            } else {
                curr = curr.right?;
            }
        }
    }
}

impl<'a, T> Set<'a, T>
where
    T: PartialOrd,
{
    /// Insert an item into the set if it does not already exist,
    /// call a continuation on the new (or old) set, and return its result
    ///
    /// This is an **O(logn)** operation.
    pub fn try_insert<F, R>(&self, item: T, then: F) -> R
    where
        F: FnOnce(&Set<T>) -> R,
    {
        if self.contains(&item) {
            then(self)
        } else {
            self.insert(item, then)
        }
    }
    /// Insert an item into the set, call a continuation on the
    /// new set, and return its result
    ///
    /// If an entry with the item already exists in the set, it is not removed,
    /// but the new entry is still inserted. All lookups on the new set
    /// will find the most recently inserted item.
    ///
    /// This is an **O(logn)** operation.
    pub fn insert<F, R>(&self, item: T, then: F) -> R
    where
        F: FnOnce(&Set<T>) -> R,
    {
        let mut node = SetNode {
            item,
            left: None,
            right: None,
        };
        if let Some(head) = self.head {
            if node.item < head.item {
                node.right = Some(head);
                let mut curr = head;
                while node.item < curr.item {
                    curr = if let Some(next) = curr.left.or(curr.right) {
                        next
                    } else {
                        break;
                    }
                }
                if node.item > curr.item {
                    node.left = Some(curr);
                }
            } else {
                node.left = Some(head);
                let mut curr = head;
                while node.item >= curr.item {
                    curr = if let Some(next) = curr.right.or(curr.left) {
                        next
                    } else {
                        break;
                    }
                }
                if node.item < curr.item {
                    node.right = Some(curr);
                }
            }
        }
        then(&Set {
            head: Some(&node),
            len: self.len + 1,
        })
    }
    /// Get an iterator over the key/item pairs of the list
    ///
    /// The iterator yields items in the opposite order of their insertion.
    pub fn iter<'s>(&'s self) -> Iter<'a, 's, T> {
        Iter { node: self.head }
    }
    /// Collect an iterator into a set and call a continuation function on it
    ///
    /// # Example
    /// ```
    /// use nolloc::Set;
    ///
    /// let nums = [2, 6, 2, 8, 5, 4];
    ///
    /// Set::collect(nums, |set| {
    ///     for n in &nums {
    ///         assert!(set.contains(n));
    ///     }
    /// });
    /// ```
    pub fn collect<I, F, R>(iter: I, then: F) -> R
    where
        T: PartialOrd,
        I: IntoIterator<Item = T>,
        F: FnOnce(&Set<T>) -> R,
    {
        Set::default().extend(iter, then)
    }
    /// Extend the set with an iterator and call a continuation function on it
    pub fn extend<I, F, R>(&self, iter: I, then: F) -> R
    where
        T: PartialOrd,
        I: IntoIterator<Item = T>,
        F: FnOnce(&Set<T>) -> R,
    {
        let mut iter = iter.into_iter();
        if let Some(item) = iter.next() {
            self.insert(item, |list| list.extend(iter, then))
        } else {
            then(self)
        }
    }
}

/// An iterator over the key/item pairs of a [`Set`]
pub struct Iter<'a, 's, T> {
    node: Option<&'s SetNode<'a, T>>,
}

impl<'a, T> SetNode<'a, T> {
    fn contains_child(&self, child: &Self) -> bool {
        self.left.map_or(false, |node| ptr::eq(node, child))
            || self.right.map_or(false, |node| ptr::eq(node, child))
            || self.left.map_or(false, |node| node.contains_child(child))
            || self.right.map_or(false, |node| node.contains_child(child))
    }
}

impl<'a, 's, T> Iterator for Iter<'a, 's, T>
where
    T: PartialOrd,
{
    type Item = &'s T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node?;
        let res = &node.item;
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

impl<'a, 's, T> IntoIterator for &'s Set<'a, T>
where
    T: PartialOrd,
{
    type Item = &'s T;
    type IntoIter = Iter<'a, 's, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Default for Set<'a, T> {
    fn default() -> Self {
        Set { head: None, len: 0 }
    }
}

impl<'a, T> Clone for Set<'a, T> {
    fn clone(&self) -> Self {
        Set {
            head: self.head,
            len: self.len,
        }
    }
}

impl<'a, T> Copy for Set<'a, T> {}

impl<'a, T> PartialEq for Set<'a, T>
where
    T: PartialOrd,
{
    fn eq(&self, other: &Self) -> bool {
        for item in self {
            if !other.contains(item) {
                return false;
            }
        }
        for item in other {
            if !self.contains(item) {
                return false;
            }
        }
        true
    }
}

impl<'a, T> Eq for Set<'a, T> where T: PartialOrd + Eq {}

impl<'a, T> fmt::Debug for Set<'a, T>
where
    T: PartialOrd + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
