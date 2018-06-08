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
            column_size << 1;
        }
        ret
    }
}

impl<V> CumlMap for CumlFreqTable<V>
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

    fn get_quantile(&self, key: Self::Value) -> Self::Key {
        unimplemented!();
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

    fn get_quantile(&self, key: Self::Value) -> Self::Key {
        unimplemented!();
    }
}

/*****************************************************************************
 * Tests, etc.
 *****************************************************************************/

#[cfg(test)]
mod tests {
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
