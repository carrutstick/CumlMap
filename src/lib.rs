extern crate num_traits;

use num_traits::Zero;
use std::ops::{Add, Sub};

pub trait CumlMap {
    type Key;
    type Value;

    fn insert(&mut self, Self::Key, Self::Value);
    fn get_cuml(&self, Self::Key) -> Self::Value;
    fn get_single(&self, Self::Key) -> Self::Value;
    fn get_quantile(&self, Self::Value) -> Self::Key;
}

/*****************************************************************************
 * Cumulative Frequency Table, per Simon Tatham
 *****************************************************************************/

struct CumlFreqTable<V> {
    capacity: usize,
    total: V,
    tables: Vec<Vec<V>>,
}

impl<V> CumlFreqTable<V>
where
    V: Add + Sub + Zero + Copy,
{
    fn with_capacity(c: usize) -> CumlFreqTable<V> {
        let cap = c.next_power_of_two();
        let mut ret = CumlFreqTable {
            capacity: cap,
            total: V::zero(),
            tables: Vec::new(),
        };
        let mut column_size = 1;
        while column_size < cap {
            ret.tables.push(vec![V::zero(); column_size]);
            column_size = column_size << 1;
        }
        ret
    }
}

impl<V> CumlMap for CumlFreqTable<V>
where
    V: Add<Output=V> + Sub<Output=V> + Zero + Copy + PartialOrd,
{
    type Key = usize;
    type Value = V;

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        assert!(key < self.capacity);
        self.total = self.total + val;
        let mut bit: usize = self.tables.len();
        for ref mut tbl in self.tables.iter_mut() {
            bit -= 1;
            if (key & (1 << bit)) != 0 { continue }
            let j = key >> (bit + 1);
            tbl[j] = tbl[j] + val;
        }
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        if key >= self.capacity - 1 { return self.total }
        let key = key + 1;
        let mut acc: Self::Value = Self::Value::zero();
        let mut bit: usize = self.tables.len();
        for ref tbl in self.tables.iter() {
            bit -= 1;
            if (key & (1 << bit)) == 0 { continue }
            let j = key >> (bit + 1);
            acc = acc + tbl[j];
        }
        acc
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
        assert!(key < self.capacity);
        if key > 0 {
            self.get_cuml(key) - self.get_cuml(key - 1)
        } else {
            self.tables[self.tables.len()-1][0]
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Self::Key {
        assert!(quant <= self.total);
        let mut index = 0;
        let mut acc = Self::Value::zero();
        for ref tbl in self.tables.iter() {
            if tbl[index] + acc >= quant {
                index = index << 1
            } else {
                acc = acc + tbl[index];
                index = (index << 1) + 1
            }
        }
        index
    }
}

/*****************************************************************************
 * Binary Index Tree, per Peter Fenwick
 *****************************************************************************/

struct BinaryIndexTree<V> {
    capacity: usize,
    data: Vec<V>,
}

impl<V> BinaryIndexTree<V>
where
    V: Add + Sub + Zero + Copy,
{
    fn with_capacity(c: usize) -> BinaryIndexTree<V> {
        BinaryIndexTree {
            capacity: c,
            data: vec![V::zero(); c],
        }
    }
}

impl<V> CumlMap for BinaryIndexTree<V>
where
    V: Add + Sub + Zero + Copy,
{
    type Key = usize;
    type Value = V;

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        unimplemented!();
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        unimplemented!();
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
        unimplemented!();
    }

    fn get_quantile(&self, quant: Self::Value) -> Self::Key {
        unimplemented!();
    }
}

/*****************************************************************************
 * Tests, etc.
 *****************************************************************************/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn trivial_cft() {
        let mut t = CumlFreqTable::with_capacity(4);
        t.insert(0, 1);
        t.insert(1, 2);
        t.insert(2, 3);
        t.insert(3, 5);

        assert_eq!(t.get_single(0), 1);
        assert_eq!(t.get_single(1), 2);
        assert_eq!(t.get_single(2), 3);
        assert_eq!(t.get_single(3), 5);

        assert_eq!(t.get_cuml(0), 1);
        assert_eq!(t.get_cuml(1), 3);
        assert_eq!(t.get_cuml(2), 6);
        assert_eq!(t.get_cuml(3), 11);

        assert_eq!(t.get_quantile(0), 0);
        assert_eq!(t.get_quantile(1), 0);
        assert_eq!(t.get_quantile(2), 1);
        assert_eq!(t.get_quantile(3), 1);
        assert_eq!(t.get_quantile(4), 2);
        assert_eq!(t.get_quantile(6), 2);
        assert_eq!(t.get_quantile(10), 3);
    }

    #[test]
    fn trivial_bix() {
        let mut t = BinaryIndexTree::with_capacity(4);
        t.insert(0, 1);
        t.insert(1, 2);
        t.insert(2, 3);
        t.insert(3, 5);

        assert_eq!(t.get_single(0), 1);
        assert_eq!(t.get_single(1), 2);
        assert_eq!(t.get_single(2), 3);
        assert_eq!(t.get_single(3), 5);

        assert_eq!(t.get_cuml(0), 1);
        assert_eq!(t.get_cuml(1), 3);
        assert_eq!(t.get_cuml(2), 6);
        assert_eq!(t.get_cuml(3), 11);
    }
}
