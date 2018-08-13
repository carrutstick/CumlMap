extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};

use std::num::NonZeroUsize;

use cmap::*;

/*****************************************************************************
 * Tree structure with preallocated arena of nodes
 *****************************************************************************/

struct ArenaCumlNode<K, V> {
    key: K,
    val: V,
    left: Option<NonZeroUsize>,
    right: Option<NonZeroUsize>,
}

pub struct ArenaCumlTree<K, V> {
    nodes: Vec<ArenaCumlNode<K, V>>,
    root: Option<NonZeroUsize>,
}

impl<K, V> ArenaCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn get_total(&self, n: &Option<NonZeroUsize>) -> V {
        let mut p = n;
        let mut acc = V::zero();
        while let Some(i) = *p {
            let n = &self.nodes[i.get()];
            acc = acc + n.val;
            p = &n.right;
        }
        acc
    }

    pub fn new() -> Self {
        let mut nodes = Vec::new();
        nodes.push(ArenaCumlNode {
            key: K::zero(),
            val: V::zero(),
            left: None,
            right: None,
        });
        ArenaCumlTree {
            nodes: nodes,
            root: None,
        }
    }


    pub fn with_capacity(c: usize) -> Self {
        let mut nodes = Vec::with_capacity(c);
        nodes.push(ArenaCumlNode {
            key: K::zero(),
            val: V::zero(),
            left: None,
            right: None,
        });
        ArenaCumlTree {
            nodes: nodes,
            root: None,
        }
    }
}

impl<K, V> CumlMap for ArenaCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        let l = self.nodes.len();
        let mut p = &mut self.root;
        while let Some(i) = *p {
            let n = &mut self.nodes[i.get()];
            if k < n.key {
                n.val = n.val + v;
                p = &mut n.left;
            } else if k > n.key {
                p = &mut n.right;
            } else {
                n.val = n.val + v;
                return;
            }
        }
        *p = NonZeroUsize::new(l);
        self.nodes.push(ArenaCumlNode {
            key: k,
            val: v,
            left: None,
            right: None,
        });
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        let mut acc = Self::Value::zero();
        let mut p = &self.root;
        while let Some(i) = *p {
            let n = &self.nodes[i.get()];
            if k < n.key {
                p = &n.left;
            } else if k > n.key {
                acc = acc + n.val;
                p = &n.right;
            } else {
                return acc + n.val;
            }
        }
        acc
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        let mut p = &self.root;
        while let Some(i) = *p {
            let n = &self.nodes[i.get()];
            if k < n.key {
                p = &n.left;
            } else if k > n.key {
                p = &n.right;
            } else {
                return n.val - self.get_total(&n.left);
            }
        }
        Self::Value::zero()
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        let mut p = &self.root;
        let mut lastabove = None;
        let mut last = None;
        let mut q = quant;
        while let Some(i) = *p {
            let n = &self.nodes[i.get()];
            if q < n.val {
                lastabove = Some(n.key);
                p = &n.left
            } else if q > n.val {
                last = Some(n.key);
                q = q - n.val;
                p = &n.right
            } else {
                return Some(n.key);
            }
        }
        if q > Self::Value::zero() {
            lastabove
        } else {
            last
        }
    }
}
