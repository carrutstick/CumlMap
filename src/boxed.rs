extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};

use cmap::*;

/*****************************************************************************
 * Boxed cumulative frequency tree
 *****************************************************************************/

struct BoxedCumlNode<K, V> {
    index: K,
    val: V,
    left: Option<Box<BoxedCumlNode<K, V>>>,
    right: Option<Box<BoxedCumlNode<K, V>>>,
}

impl<K, V> BoxedCumlNode<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn new(k: K, v: V) -> Self {
        BoxedCumlNode {
            index: k,
            val: v,
            left: None,
            right: None,
        }
    }

    fn get_total(&self) -> V {
        self.val + match self.right {
            None => V::zero(),
            Some(ref r) => r.get_total(),
        }
    }

    fn insert(&mut self, k: K, v: V) {
        if k < self.index {
            self.val = self.val + v;
            match self.left {
                None => self.left = Some(Box::new(Self::new(k, v))),
                Some(ref mut l) => l.insert(k, v),
            }
        } else if k > self.index {
            match self.right {
                None => self.right = Some(Box::new(Self::new(k, v))),
                Some(ref mut r) => r.insert(k, v),
            }
        } else {
            self.val = self.val + v
        }
    }

    fn get_cuml(&self, k: K, acc: V) -> V {
        if k < self.index {
            match self.left {
                None => acc,
                Some(ref l) => l.get_cuml(k, acc),
            }
        } else if k > self.index {
            match self.right {
                None => acc + self.val,
                Some(ref r) => r.get_cuml(k, acc + self.val),
            }
        } else {
            acc + self.val
        }
    }

    fn get_single(&self, k: K) -> V {
        if k < self.index {
            match self.left {
                None => V::zero(),
                Some(ref l) => l.get_single(k),
            }
        } else if k > self.index {
            match self.right {
                None => V::zero(),
                Some(ref r) => r.get_single(k),
            }
        } else {
            match self.left {
                None => self.val,
                Some(ref l) => self.val - l.get_total(),
            }
        }
    }

    fn get_quantile(&self, v: V) -> Option<K> {
        if v > self.val {
            match self.right {
                None => None,
                Some(ref r) => r.get_quantile(v - self.val),
            }
        } else if v < self.val {
            match self.left {
                None => Some(self.index),
                Some(ref l) => match l.get_quantile(v) {
                    None => Some(self.index),
                    s => s,
                },
            }
        } else {
            Some(self.index)
        }
    }
}

pub struct BoxedCumlTree<K, V> {
    root: Option<Box<BoxedCumlNode<K, V>>>,
}

impl<K, V> BoxedCumlTree<K, V> {
    pub fn new() -> Self {
        BoxedCumlTree { root: None }
    }
}

impl<K, V> CumlMap for BoxedCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        match self.root {
            Some(ref mut n) => n.insert(k, v),
            None => self.root = Some(Box::new(BoxedCumlNode::new(k, v))),
        }
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        match self.root {
            Some(ref n) => n.get_cuml(k, Self::Value::zero()),
            None => Self::Value::zero(),
        }
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        match self.root {
            Some(ref n) => n.get_single(k),
            None => Self::Value::zero(),
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        match self.root {
            Some(ref n) => n.get_quantile(quant),
            None => None,
        }
    }
}
