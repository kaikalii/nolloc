#![no_std]
#![warn(missing_docs)]

/*!
 * # Description
 *
 * This crate provides utilities for doing things in rust without dynamic memory.
 *
 * It uses `#![no_std]`, so you can be sure that no operation will allocate!
 */

mod list;

pub use list::*;
