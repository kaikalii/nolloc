#![no_std]
#![warn(missing_docs)]
#![deny(unsafe_code)]

/*!
# Description
This crate provides utilities for doing things in rust without dynamic memory allocation.
It uses `#![no_std]`, so you can be sure that no operation will allocate!

# Collections

This crate currently provides 3 collection which keep their items entirely on the stack:

- [`List`] - a singly-linked list
- [`Map`] - an append-only key-value map with O(logn) lookup and insertion
- [`Set`] - an append-only set with O(logn) lookup and insertion

# Use Cases

Let's say you have some iterator of numbers of unknown length, and you want to
get the average of them.

One way to do this would be to iterate twice:
```
# let some_predicate = |&i: &i32| i % 2 == 0;
let count = (0..100).filter(some_predicate).count();
let sum: i32 = (0..100).filter(some_predicate).sum();
let average = sum as f32 / count as f32;
```
This is inefficient because `some_predicate` may be complex, and we are running
it twice as much as necessary!

Another option is to collect the numbers into a `Vec`:
```
# let some_predicate = |&i: &i32| i % 2 == 0;
let numbers: Vec<i32> = (0..100).filter(some_predicate).collect();
let sum: i32 = numbers.iter().sum();
let average = sum as f32 / numbers.len() as f32;
```
But this performs an allocation, which is not always desired or even possible.

This crate provides [`List`], a resizeable list type where all items exist on the stack.
`List` works by using continuations to process pushed-to lists.

Here is how you could use `List` to get the average:
```
use nolloc::List;

# let some_predicate = |&i: &i32| i % 2 == 0;
let average = List::collect((0..100).filter(some_predicate), |list| {
    let sum: i32 = list.iter().sum();
    sum as f32 / list.len() as f32;
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
