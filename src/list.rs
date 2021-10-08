use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

/// A growable list where all items exist on the stack
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
    pub fn len(&self) -> usize {
        self.len
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
        F: FnOnce(List<T>) -> R,
    {
        let new_head = ListNode::Cons(item, self.head);
        let list = List {
            head: &new_head,
            len: self.len + 1,
        };
        then(list)
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
    pub fn iter<'l>(&'l self) -> ListIter<'a, 'l, T> {
        ListIter { node: self.head }
    }
    /// Check if the list contains an item
    pub fn contains<U>(&self, item: &U) -> bool
    where
        T: PartialEq<U>,
    {
        self.iter().any(|i| i == item)
    }
    /// Collect an iterator into a list and call a continuation function on the list
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
        F: Fn(&List<I::Item>) -> R,
    {
        List::default().extend(iter, then)
    }
    /// Extend the list with an iterator and call a continuation function on it
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
    pub fn extend<I, F, R>(&self, iter: I, f: F) -> R
    where
        I: IntoIterator<Item = T>,
        F: Fn(&List<I::Item>) -> R,
    {
        let mut iter = iter.into_iter();
        if let Some(item) = iter.next() {
            self.push(item, |list| list.extend(iter, f))
        } else {
            f(self)
        }
    }
}

/// A borrowing iterator over the items in a list
pub struct ListIter<'a, 'l, T> {
    node: &'l ListNode<'a, T>,
}

impl<'a, 'l, T> Iterator for ListIter<'a, 'l, T> {
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
    type IntoIter = ListIter<'a, 'l, T>;
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
