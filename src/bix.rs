extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::mem;
use std::cmp;
use std::fmt::Debug;

use cmap::*;

/*****************************************************************************
 * Binary Index Tree, per Peter Fenwick
 *****************************************************************************/

/// A Fenwick Tree[^fn] structure, useful for very quickly mapping a
/// non-negative integer key to a cumulative value.
///
/// [^fn]: Peter M. Fenwick (1994). "A new data structure for cumulative
/// frequency tables" (PDF). Software: Practice and Experience. 24 (3): 327–336.
/// CiteSeerX 10.1.1.14.8917 Freely accessible. doi:10.1002/spe.4380240306
pub struct FenwickTree<V> {
    capacity: usize,
    data: Vec<V>,
}

impl<V> FenwickTree<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    /// Create a `FenwickTree` object with a fixed capacity.
    /// 
    /// # Examples
    /// ```
    /// use cuml_map::FenwickTree as FT;
    /// 
    /// let ft: FT<u32> = FT::with_capacity(1234);
    /// ```
    pub fn with_capacity(c: usize) -> FenwickTree<V> {
        FenwickTree {
            capacity: c,
            data: vec![V::zero(); c],
        }
    }
}

impl<V> CumlMap for FenwickTree<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = usize;
    type Value = V;

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        assert!(key < self.capacity);
        let mut key = key;
        while key < self.capacity {
            self.data[key as usize] = self.data[key as usize] + val;
            if key == 0 {
                break;
            }
            key += 1 << key.trailing_zeros();
        }
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        let mut key = cmp::min(key, self.capacity - 1);
        let mut sum = self.data[0];
        while key > 0 {
            sum = sum + self.data[key];
            key = key & (key - 1);
        }
        sum
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
        if key >= self.capacity { return Self::Value::zero() }
        let mut val = self.data[key];
        let mut key = key;
        if key == 0 {
            return val;
        }
        let parent = key & (key - 1);
        key -= 1;
        while parent != key {
            val = val - self.data[key];
            key = key & (key - 1);
        }
        val
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        if quant <= self.data[0] {
            Some(0)
        } else {
            let mut step = self.capacity.next_power_of_two() >> 1;
            let mut ix = 0;
            let mut quant = quant - self.data[0];
            while step > 0 {
                if ix + step < self.capacity && self.data[ix + step] < quant {
                    ix += step;
                    quant = quant - self.data[ix];
                }
                step >>= 1;
            }
            if quant == Self::Value::zero() {
                Some(ix)
            } else if ix + 1 < self.capacity {
                Some(ix + 1)
            } else {
                None
            }
        }
    }
}

/*****************************************************************************
 * Extensible Binary Index Tree, allowing negative indices
 ****************************************************************************/

/// An extensible version of the Fenwick Tree. `ExtensibleFenwickTree` uses
/// a `FenwickTree` internally, but applies an offset to any keys inserted
/// or queried, allowing it to take negative keys. If a key is inserted
/// that is outside the bounds of the underlying `FenwickTree`, then a new
/// `FenwickTree` is created with sufficient capacity, all entries from the
/// old tree are inserted into the new tree, and the old tree is dropped.
pub struct ExtensibleFenwickTree<V> {
    offset: i64, // minimum possible key in mapping
    tree: FenwickTree<V>,
}

impl<V> ExtensibleFenwickTree<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord + Debug,
{
    /// Creates an empty tree with zero initial capacity.
    /// 
    /// # Examples
    /// ```
    /// use cuml_map::ExtensibleFenwickTree as EFT;
    /// 
    /// let eft: EFT<u32> = EFT::new();
    /// ```
    pub fn new() -> Self {
        Self {
            offset: 0,
            tree: FenwickTree::with_capacity(0),
        }
    }

    /// Creates an empty tree with fixed initial capacity.
    /// 
    /// # Examples
    /// ```
    /// use cuml_map::ExtensibleFenwickTree as EFT;
    /// 
    /// let eft: EFT<u32> = EFT::with_capacity(1234);
    /// ```
    pub fn with_capacity(c: usize) -> Self {
        Self {
            offset: 0,
            tree: FenwickTree::with_capacity(c),
        }
    }

    /// Creates an empty tree with a capacity `c` and offset `o`, such that
    /// the tree initially covers the range of keys `[o, o+c)`.
    /// 
    /// # Examples
    /// ```
    /// use cuml_map::ExtensibleFenwickTree as EFT;
    /// 
    /// let eft: EFT<u32> = EFT::with_extent(0, 1234);
    /// ```
    pub fn with_extent(o: i64, c: usize) -> Self {
        Self {
            offset: o,
            tree: FenwickTree::with_capacity(c),
        }
    }

    fn extent(&self) -> (i64, i64) {
        (self.offset, self.offset + (self.tree.capacity as i64))
    }

    fn extend(&mut self, by: i64) {
        let oldcap = self.tree.capacity;
        let cap = oldcap + by.abs() as usize;
        let new = FenwickTree::with_capacity(cap);
        let old = mem::replace(&mut self.tree, new);
        let oldoff = self.offset;
        self.offset += cmp::min(by, 0);

        for i in 0..oldcap {
            self.tree.insert((i as i64 - self.offset + oldoff) as usize, old.get_single(i));
        }
    }

    /// Ensures that the tree will cover key `key`, in addition to all keys
    /// previously covered. Reallocates and rebuilds the tree if necessary.
    /// 
    /// # Examples
    /// ```
    /// use cuml_map::ExtensibleFenwickTree as EFT;
    /// 
    /// let mut eft1: EFT<u32> = EFT::new();
    /// eft1.ensure_contains(1024);
    /// 
    /// let mut eft2: EFT<u32> = EFT::with_extent(0, 1234);
    /// eft2.ensure_contains(1024);
    /// ```
    pub fn ensure_contains(&mut self, key: i64) {
        let (l, r) = self.extent();
        let extra =
            if key >= r { key - r + 1 }
            else if key < l { key - l }
            else { return };
        let mut cap = cmp::max(8, self.tree.capacity as i64) * extra.signum();
        while cap.abs() < extra.abs() {
            cap *= 2;
        }
        self.extend(cap);
    }
}

impl<V> CumlMap for ExtensibleFenwickTree<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord + Debug,
{
    type Key = i64;
    type Value = V;

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        self.ensure_contains(key);
        self.tree.insert((key - self.offset) as usize, val);
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        if self.offset > key {
            Self::Value::zero()
        } else {
            self.tree.get_cuml((key - self.offset) as usize)
        }
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
        if self.offset > key {
            Self::Value::zero()
        } else {
            self.tree.get_single((key - self.offset) as usize)
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        self.tree.get_quantile(quant).map(|x| x as i64 + self.offset)
    }
}

