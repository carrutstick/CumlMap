# cuml_map
[![Docs.rs](https://docs.rs/cuml_map/badge.svg)](https://docs.rs/cuml_map)

The goal of this library is to provide data structures that allow for efficient creation and querying of cumulative maps.
That is, for some mapping `K -> V`, we want to make it quick and efficient for users to ask the following questions:
1. What is the value associated with key `k`?
2. What is the sum of all values associated with keys `<= k`?
3. What is the first value `k` where the sum of all values with key `<= k` is `>=` some value `v`?

These queries are abstracted into the `CumlMap` trait, which has the following definition:
```rust
pub trait CumlMap {
    type Key;
    type Value;
    fn insert(&mut self, Self::Key, Self::Value);
    fn get_cuml(&self, Self::Key) -> Self::Value;              // Question 2
    fn get_single(&self, Self::Key) -> Self::Value;            // Question 1
    fn get_quantile(&self, Self::Value) -> Option<Self::Key>;  // Question 3
}
```

Additionally, three implementations of this trait are provided:
1. `FenwickTree` implements a very efficient array-based mapping<sup>1</sup> where `Key=usize`.
2. `ExtensibleFenwickTree` implements a wrapper around (1), which allows it to be dynamically resized, and take
potentially negative keys.
3. `CumlTree` uses a red-black tree based structure to generalize to any ordered keys, and will be much more
space-efficient than the other two for sparse keys.

<sup>1</sup> Peter M. Fenwick (1994). "A new data structure for cumulative
frequency tables" (PDF). Software: Practice and Experience. 24 (3): 327–336.
CiteSeerX 10.1.1.14.8917 Freely accessible. doi:10.1002/spe.4380240306
