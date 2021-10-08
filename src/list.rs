//! A growable, singly-linked list where all items exist on the stack

use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

/// A growable, singly-linked list where all items exist on the stack
///
/// When using [`List::push`], the new list with the pushed
/// item cannot be accessed from the same scope. Instead,
/// a continuation function is called on the new list, and its
/// result is returned to the calling scope.
pub struct List<'a, T> {
    head: &'a ListNode<'a, T>,
    len: usize,
}

#[derive(Eq, Hash)]
enum ListNode<'a, T> {
    Nil,
    Cons(T, &'a ListNode<'a, T>),
}

impl<'a, T> List<'a, T> {
    /// Create a new list
    pub fn new() -> Self {
        List::default()
    }
    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Get the list's length
    ///
    /// This is an **O(1)** operation.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Get the first item in the list
    pub fn first(&self) -> Option<&'a T> {
        match self.head {
            ListNode::Nil => None,
            ListNode::Cons(x, _) => Some(x),
        }
    }
    /// Get all items after the first item in the list
    pub fn rest(&self) -> List<'a, T> {
        match self.head {
            ListNode::Nil => List::default(),
            ListNode::Cons(_, xs) => List {
                head: xs,
                len: self.len - 1,
            },
        }
    }
    /// Get the last item in the list
    ///
    /// This is an **O(n)** operation.
    pub fn last(&self) -> Option<&'a T> {
        match self.head {
            ListNode::Nil => None,
            ListNode::Cons(x, mut xs) => {
                let mut x = x;
                while let ListNode::Cons(xs_x, xs_xs) = xs {
                    x = xs_x;
                    xs = xs_xs;
                }
                Some(x)
            }
        }
    }
    /// Push an item onto the front of the list and call a continuation function
    ///
    /// # Example
    /// ```
    /// use nolloc::List;
    ///
    /// let sum: i32 = List::default().push(1, |list| {
    ///     list.push(2, |list| {
    ///         list.push(3, |list| {
    ///             for i in 1..=3 {
    ///                 assert!(list.contains(&i));
    ///             }
    ///             list.iter().sum()
    ///         })
    ///     })
    /// });
    ///
    /// assert_eq!(sum, 6);
    /// ```
    pub fn push<F, R>(&self, item: T, then: F) -> R
    where
        F: FnOnce(&List<T>) -> R,
    {
        let new_head = ListNode::Cons(item, self.head);
        let list = List {
            head: &new_head,
            len: self.len + 1,
        };
        then(&list)
    }
    /// Attempt to pop and item off the front of the list
    ///
    /// Returns a tuple of the list without the item and the item
    pub fn pop(&self) -> (Self, Option<&'a T>) {
        match self.head {
            ListNode::Nil => (*self, None),
            ListNode::Cons(x, xs) => (
                List {
                    head: xs,
                    len: self.len - 1,
                },
                Some(x),
            ),
        }
    }
    /// Attempt to pop an item off the front of the list
    pub fn pop_mut(&mut self) -> Option<&'a T> {
        let (list, item) = self.pop();
        *self = list;
        item
    }
    /// Get an iterator over the items of the list
    pub fn iter<'l>(&'l self) -> Iter<'a, 'l, T> {
        Iter { node: self.head }
    }
    /// Check if the list contains an item
    ///
    /// This is an **O(n)** operation.
    pub fn contains<U>(&self, item: &U) -> bool
    where
        T: PartialEq<U>,
    {
        self.iter().any(|i| i == item)
    }
    /// Collect an iterator into a list and call a continuation function on it
    ///
    /// The items in the list will be in reversed order. To make the list's order
    /// match the iterator's order, use [`List::collect_in_order`].
    ///
    /// # Example
    /// ```
    /// use nolloc::List;
    ///
    /// let numbers = [1, 2, 3, 4, 5];
    ///
    /// let sum: i32 = List::collect(numbers, |list| {
    ///     assert_eq!(numbers.len(), list.len());
    ///     for n in numbers {
    ///         assert!(list.contains(&n));
    ///     }
    ///     list.iter().sum()
    /// });
    ///
    /// assert_eq!(sum, 15);
    /// ```
    pub fn collect<I, F, R>(iter: I, then: F) -> R
    where
        I: IntoIterator<Item = T>,
        F: FnOnce(&List<T>) -> R,
    {
        List::default().extend(iter, then)
    }
    /// Like [`List::collect`], but collects items in order
    pub fn collect_in_order<I, F, R>(iter: I, then: F) -> R
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: DoubleEndedIterator,
        F: FnOnce(&List<T>) -> R,
    {
        List::collect(iter.into_iter().rev(), then)
    }
    /// Extend the list with an iterator and call a continuation function on it
    ///
    /// The items in the list will be in reversed order. To make the list's order
    /// match the iterator's order, use [`List::extend_in_order`].
    ///
    /// # Example
    /// ```
    /// use nolloc::List;
    ///
    /// let numbers = [1, 2, 3, 4, 5];
    ///
    /// let sum: i32 = List::default().push(6, |list| {
    ///     list.extend(numbers, |list| {
    ///         for n in numbers {
    ///             assert!(list.contains(&n));
    ///         }
    ///         list.iter().sum()
    ///     })
    /// });
    ///
    /// assert_eq!(sum, 21);
    /// ```
    pub fn extend<I, F, R>(&self, iter: I, then: F) -> R
    where
        I: IntoIterator<Item = T>,
        F: FnOnce(&List<T>) -> R,
    {
        let mut iter = iter.into_iter();
        if let Some(item) = iter.next() {
            self.push(item, |list| list.extend(iter, then))
        } else {
            then(self)
        }
    }
    /// Like [`List::extend`], but collects items in order.
    ///
    /// While the order of the extended items will match the
    /// iterator's order, they will still come before any items
    /// already in the list.
    pub fn extend_in_order<I, F, R>(&self, iter: I, then: F) -> R
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: DoubleEndedIterator,
        F: FnOnce(&List<T>) -> R,
    {
        self.extend(iter.into_iter().rev(), then)
    }
    /// Reverse the list, pass the reversed list to a continuation,
    /// and return the result.
    ///
    /// This *does not* deallocate the orignal list, so be wary of
    /// stack overflows when reversing larger lists.
    ///
    /// # Example
    ///
    /// ```
    /// use nolloc::List;
    ///
    /// let numbers = [1, 2, 3, 4, 5];
    /// List::collect(numbers, |list| {
    ///     // The collected list is reverse compared to the original iterator
    ///     // We have to reverse the numbers iterator to make the assertion pass
    ///     for (i, n) in list.iter().zip(numbers.iter().rev()) {
    ///         assert_eq!(i, n);
    ///     }
    ///     
    ///     // Reverse the list to make the order match
    ///     list.reverse(|list| {
    ///         for (i, n) in list.iter().copied().zip(&numbers) {
    ///             assert_eq!(i, n);
    ///         }
    ///     });
    /// })
    /// ```
    pub fn reverse<F, R>(&self, then: F) -> R
    where
        F: FnOnce(&List<&T>) -> R,
    {
        List::collect(self.iter(), then)
    }
}

/// An iterator over the items in a [`List`]
pub struct Iter<'a, 'l, T> {
    node: &'l ListNode<'a, T>,
}

impl<'a, 'l, T> Iterator for Iter<'a, 'l, T> {
    type Item = &'l T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node {
            ListNode::Nil => None,
            ListNode::Cons(x, xs) => {
                self.node = xs;
                Some(x)
            }
        }
    }
}

impl<'a, 'l, T> IntoIterator for &'l List<'a, T> {
    type Item = &'l T;
    type IntoIter = Iter<'a, 'l, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Default for List<'a, T> {
    fn default() -> Self {
        List {
            head: &ListNode::Nil,
            len: 0,
        }
    }
}

impl<'a, T> Clone for List<'a, T> {
    fn clone(&self) -> Self {
        List {
            head: self.head,
            len: self.len,
        }
    }
}

impl<'a, T> Copy for List<'a, T> {}

impl<'a, T, U> PartialEq<List<'a, U>> for List<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &List<'a, U>) -> bool {
        self.head == other.head
    }
}

impl<'a, T> Eq for List<'a, T> where T: Eq {}

impl<'a, T, U> PartialOrd<List<'a, U>> for List<'a, T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &List<'a, U>) -> Option<Ordering> {
        self.head.partial_cmp(other.head)
    }
}

impl<'a, T> Ord for List<'a, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.head.cmp(other.head)
    }
}

impl<'a, T> Hash for List<'a, T>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.head.hash(state)
    }
}

impl<'a, T> fmt::Debug for List<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T> fmt::Display for List<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, ")")
    }
}

impl<'a, T, U> PartialEq<ListNode<'a, U>> for ListNode<'a, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &ListNode<'a, U>) -> bool {
        match (self, other) {
            (ListNode::Nil, ListNode::Nil) => true,
            (ListNode::Cons(x, xs), ListNode::Cons(y, ys)) => x == y && xs == ys,
            _ => false,
        }
    }
}

impl<'a, T, U> PartialOrd<ListNode<'a, U>> for ListNode<'a, T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &ListNode<'a, U>) -> Option<Ordering> {
        match (self, other) {
            (ListNode::Nil, ListNode::Nil) => Some(Ordering::Equal),
            (ListNode::Nil, ListNode::Cons(..)) => Some(Ordering::Less),
            (ListNode::Cons(..), ListNode::Nil) => Some(Ordering::Greater),
            (ListNode::Cons(x, xs), ListNode::Cons(y, ys)) => x
                .partial_cmp(y)
                .and_then(|ord1| xs.partial_cmp(ys).map(|ord2| ord1.then(ord2))),
        }
    }
}

impl<'a, T> Ord for ListNode<'a, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ListNode::Nil, ListNode::Nil) => Ordering::Equal,
            (ListNode::Nil, ListNode::Cons(..)) => Ordering::Less,
            (ListNode::Cons(..), ListNode::Nil) => Ordering::Greater,
            (ListNode::Cons(x, xs), ListNode::Cons(y, ys)) => x.cmp(y).then_with(|| xs.cmp(ys)),
        }
    }
}

#[test]
fn list_order() {
    let numbers = [1, 2, 3, 4, 5];
    List::collect(numbers, |list| {
        for (i, n) in list.iter().zip(numbers.iter().rev()) {
            assert_eq!(i, n);
        }
    });
    List::collect_in_order(numbers, |list| {
        for (i, n) in list.iter().zip(&numbers) {
            assert_eq!(i, n);
        }
    });
}
