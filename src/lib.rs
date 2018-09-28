#![feature(test)]
#![feature(nll)]
#![warn(missing_docs)]

//! This crate provides a trait, `CumlMap`, representing a mapping between
//! keys and cumulative values. That is, an object implementing `CumlMap`
//! should be able to store key-value mappings, and efficiently retrive
//! the sum of all values with a key less than or equal to a given key.
//!
//! This crate also provides three implementations of the `CumlMap` trait:
//! `FenwickTree`, `ExtensibleFenwickTree`, and `CumlTree`.
//! `FenwickTree` is the most restricted in terms of what keys and values
//! it can accept, but also the most performant in those restricted cases.
//!
//! `FenwickTree` accepts only non-negative keys, allocates all memory in
//! advance, and its capacity is fixed at creation time and
//! cannot be changed. If you need to get around any of these limitations
//! then use the `ExtensibleFenwickTree` object instead.
//!
//! Both `FenwickTree` and `ExtensibleFenwickTree` may be a poor choice
//! for sparse keys. Both structures must allocate space for at least every
//! possible key between the smallest and largest keys. To get around this
//! limitation use the `CumlMap` structure, which dynamically allocates
//! mappings, at the expense of insertion and lookup performance.

extern crate num_traits;

mod cmap;
pub use cmap::*;

mod bix;
pub use bix::*;

mod rctree;
pub use rctree::*;

#[cfg(test)]
mod tests;
