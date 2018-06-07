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

struct CumlTable<V> {
    capacity: usize,
    total: V,
    tables: Vec<Vec<V>>,
}

impl<V> CumlTable<V>
where
    V: Add + Sub + Zero + Copy,
{
    fn with_capacity(c: usize) -> CumlTable<V> {
        let cap = c.next_power_of_two();
        let mut ret = CumlTable {
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

impl<V> CumlMap for CumlTable<V>
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
