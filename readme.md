# Description

This is a Rust library that provides growable data structures that do not dynamically allocate memory.
All of their items are kept on the stack. This is done by passing a continuation function to all methods
that can add elements.

These structures are useful for when size in known only at runtime and dynamic allocation would be too
slow or even impossible, such on on embedded devices.

This library is fully `no_std` compatible, and it has no dependencies.

For more information, check out [the documentation](https://docs.rs/nolloc).

# Use Cases

Let's say you have some iterator of numbers of unknown length, and you want sum
the products of all pairs of numbers.

One way to do this would be collect the numbers into a `Vec`:
```rust
let numbers: Vec<i32> = (0..100).filter(some_predicate).collect();
let sum: i32 = numbers.iter().flat_map(|i| numbers.iter().map(move |j| i * j)).sum();
```
But this performs an allocation, which is not always desired or even possible.

Another option is reconstruct the iterator:
```rust
let numbers = || (0..100).filter(some_predicate);
let sum: i32 = numbers().flat_map(|i| numbers().map(move |j| i * j)).sum();
```
This is inefficient because `some_predicate` may be complex, and we are running
it n^2 times!

This crate provides `List`, a resizable list type where all items exist on the stack.
`List` works by using continuations to process pushed-to lists.

Here is how you could use `List` to get the sum:
```rust
use nolloc::List;

let sum: i32 = List::collect((0..100).filter(some_predicate), |list| {
    list.iter().flat_map(|i| list.iter().map(move |j| i * j)).sum()
});
```
With this solution, `some_predicate` is only called once per item, and
no allocation occurs!