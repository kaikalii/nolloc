#![no_std]
#![warn(missing_docs)]
#![deny(unsafe_code)]

/*!
# Description

This crate provides growable collections that do not use dynamic memory allocation.

It is fully `no_std` compatible, and it has no dependencies.

# Collections

This crate currently provides 3 collections which keep their items entirely on the stack:

- [`List`] - a singly-linked list
- [`Map`] - an append-only key-value map with O(logn) lookup and insertion
- [`Set`] - an append-only set with O(logn) lookup and insertion

# Use Cases

Let's say you have some iterator of numbers of unknown length, and you want sum
the products of all pairs of numbers.

One way to do this would be collect the numbers into a `Vec`:
```
# let some_predicate = |&i: &i32| i % 2 == 0;
let numbers: Vec<i32> = (0..100).filter(some_predicate).collect();
let sum: i32 = numbers.iter().flat_map(|i| numbers.iter().map(move |j| i * j)).sum();
```
But this performs an allocation, which is not always desired or even possible.

Another option is reconstruct the iterator:
```
# let some_predicate = |&i: &i32| i % 2 == 0;
let numbers = || (0..100).filter(some_predicate);
let sum: i32 = numbers().flat_map(|i| numbers().map(move |j| i * j)).sum();
```
This is inefficient because `some_predicate` may be complex, and we are running
it n^2 times!

This crate provides [`List`], a resizable list type where all items exist on the stack.
`List` works by using continuations to process pushed-to lists.

Here is how you could use `List` to get the sum:
```
use nolloc::List;

# let some_predicate = |&i: &i32| i % 2 == 0;
let sum: i32 = List::collect((0..100).filter(some_predicate), |list| {
    list.iter().flat_map(|i| list.iter().map(move |j| i * j)).sum()
});
```
With this solution, `some_predicate` is only called once per item, and
no allocation occurs!

# Stack Size

When using the [`collect`](list/struct.List.html#method.collect) methods of the collections in this crate, keep in mind the number of possible elements
you could be collecting as well as their size. All the elements are collected onto the stack, so if you are
not careful, you can get a stack overflow!
*/

pub mod list;
pub mod map;
pub mod set;

pub use {list::List, map::Map, set::Set};
