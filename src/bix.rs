extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};

use cmap::*;

/*****************************************************************************
 * Binary Index Tree, per Peter Fenwick
 *****************************************************************************/

pub struct BinaryIndexTree<V> {
    capacity: usize,
    data: Vec<V>,
}

impl<V> CumlMap for BinaryIndexTree<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = usize;
    type Value = V;

    fn with_capacity(c: usize) -> BinaryIndexTree<V> {
        BinaryIndexTree {
            capacity: c,
            data: vec![V::zero(); c],
        }
    }

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
        assert!(key < self.capacity);
        let mut key = key;
        let mut sum = self.data[0];
        while key > 0 {
            sum = sum + self.data[key];
            key = key & (key - 1);
        }
        sum
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
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
        let mut index = 0;
        let mut mask = self.capacity / 2;
        let mut quant = quant - self.data[0];
        while mask != 0 {
            let test = index + mask;
            if quant >= self.data[test] {
                quant = quant - self.data[test];
                index = test;
            }
            mask >>= 1;
        }
        if quant > Self::Value::zero() {
            if index + 1 < self.capacity {
                Some(index + 1)
            } else {
                None
            }
        } else {
            Some(index)
        }
    }
}
