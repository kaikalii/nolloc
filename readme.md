# Description

This is a Rust library that provides growable data structures that do not dynamically allocate memory.
All of their items are kept on the stack. This is done by passing a continuation function to all methods
that can add elements.

These structures are useful for when size in known only at runtime and dynamic allocation would be too
slow or even impossible, such on on embedded devices.

This library is fully `no_std` compatible, and it has no dependencies.

For more information, check out [the documentation](https://docs.rs/nolloc).

# Use Cases

Let's say you have some iterator of numbers of unknown length, and you want to
get the average of them.

One way to do this would be to iterate twice:
```rust
let count = (0..100).filter(some_predicate).count();
let sum: i32 = (0..100).filter(some_predicate).sum();
let average = sum as f32 / count as f32;
```
This is inefficient because `some_predicate` may be complex, and we are running
it twice as much as necessary!

Another option is to collect the numbers into a `Vec`:
```rust
let numbers: Vec<i32> = (0..100).filter(some_predicate).collect();
let sum: i32 = numbers.iter().sum();
let average = sum as f32 / numbers.len() as f32;
```
But this performs an allocation, which is not always desired or even possible.

This crate provides `List`, a resizable list type where all items exist on the stack.
`List` works by using continuations to process pushed-to lists.

Here is how you could use `List` to get the average:
```rust
use nolloc::List;

let average = List::collect((0..100).filter(some_predicate), |list| {
    let sum: i32 = list.iter().sum();
    sum as f32 / list.len() as f32;
});
```
With this solution, `some_predicate` is only called once per item, and
no allocation occurs!