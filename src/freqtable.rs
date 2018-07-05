use super::num_traits::Zero;
use std::ops::{Add, Sub};

use cmap::*;

/*****************************************************************************
 * Cumulative Frequency Table, per Simon Tatham
 *****************************************************************************/

pub struct CumlFreqTable<V> {
    capacity: usize,
    total: V,
    tables: Vec<Vec<V>>,
}

impl<V> CumlMap for CumlFreqTable<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = usize;
    type Value = V;

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

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        assert!(key < self.capacity);
        self.total = self.total + val;
        let mut bit: usize = self.tables.len();
        for ref mut tbl in self.tables.iter_mut() {
            bit -= 1;
            if (key & (1 << bit)) != 0 {
                continue;
            }
            let j = key >> (bit + 1);
            tbl[j] = tbl[j] + val;
        }
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        if key >= self.capacity - 1 {
            return self.total;
        }
        let key = key + 1;
        let mut acc: Self::Value = Self::Value::zero();
        let mut bit: usize = self.tables.len();
        for ref tbl in self.tables.iter() {
            bit -= 1;
            if (key & (1 << bit)) == 0 {
                continue;
            }
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
            self.tables[self.tables.len() - 1][0]
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        if quant > self.total {
            return None;
        }
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
        Some(index)
    }
}
