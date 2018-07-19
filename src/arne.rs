extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};

use cmap::*;

/*****************************************************************************
 * Boxed cumulative frequency tree
 *****************************************************************************/

struct AACumlNode<K, V> {
    index: K,
    val: V,
    left: Option<Box<AACumlNode<K, V>>>,
    right: Option<Box<AACumlNode<K, V>>>,
    level: usize,
}

fn insert_node<K, V>(n: Option<Box<AACumlNode<K, V>>>, k: K, v: V) -> Option<Box<AACumlNode<K, V>>>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    match n {
        None => Some(Box::new(AACumlNode::new(k, v))),
        Some(mut nn) => {
            if k < nn.index {
                nn.val = nn.val + v;
                nn.left = insert_node(nn.left, k, v)
            } else if k > nn.index {
                nn.right = insert_node(nn.right, k, v)
            } else {
                nn.val = nn.val + v;
                return Some(nn)
            }
            split_node(skew_node(Some(nn)))
        },
    }
}

fn skew_node<K, V>(n: Option<Box<AACumlNode<K, V>>>) -> Option<Box<AACumlNode<K, V>>>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    match n {
        None => None,
        Some(mut nn) => if nn.left.is_none() {
            Some(nn)
        } else {
            let mut l = nn.left.take().unwrap();
            if l.level == nn.level {
                nn.left = l.right.take();
                nn.val = nn.val - l.val;
                l.right = Some(nn);
                Some(l)
            } else {
                nn.left = Some(l);
                Some(nn)
            }
        },
    }
}

fn split_node<K, V>(n: Option<Box<AACumlNode<K, V>>>) -> Option<Box<AACumlNode<K, V>>>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    match n {
        None => None,
        Some(mut nn) => if nn.right.is_none() {
            Some(nn)
        } else {
            let mut r = nn.right.take().unwrap();
            if r.right.is_none() {
                nn.right = Some(r);
                Some(nn)
            } else {
                if r.right.as_ref().unwrap().level == nn.level {
                    nn.right = r.left.take();
                    r.val = r.val + nn.val;
                    r.left = Some(nn);
                    r.level += 1;
                    Some(r)
                } else {
                    nn.right = Some(r);
                    Some(nn)
                }
            }
        },
    }
}

impl<K, V> AACumlNode<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn new(k: K, v: V) -> AACumlNode<K, V> {
        AACumlNode {
            index: k,
            val: v,
            left: None,
            right: None,
            level: 1,
        }
    }

    fn get_total(&self) -> V {
        self.val + match self.right {
            None => V::zero(),
            Some(ref r) => r.get_total(),
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

pub struct AACumlTree<K, V> {
    root: Option<Box<AACumlNode<K, V>>>,
}

impl<K, V> CumlMap for AACumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn with_capacity(_k: usize) -> Self {
        AACumlTree { root: None }
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        self.root = insert_node(self.root.take(), k, v);
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
