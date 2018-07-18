extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};

use std::num::NonZeroUsize;

use cmap::*;

/*****************************************************************************
 * Tree structure with preallocated arena of nodes
 *****************************************************************************/

struct ArneCumlNode<K, V> {
    key: K,
    val: V,
    left: Option<NonZeroUsize>,
    right: Option<NonZeroUsize>,
    level: usize,
}

pub struct ArneCumlTree<K, V> {
    nodes: Vec<ArneCumlNode<K, V>>,
    root: Option<NonZeroUsize>,
}

impl<K, V> ArneCumlTree<K, V>
where
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

    fn skew(&mut self, n: Option<NonZeroUsize>) -> Option<NonZeroUsize> {
        match n {
            None => None,
            Some(k) => match self.nodes[k.get()].left {
                None => Some(k),
                Some(l) => if self.nodes[l.get()].level == self.nodes[k.get()].level {
                    self.nodes[k.get()].left = self.nodes[l.get()].right;
                    self.nodes[k.get()].val =
                        self.nodes[k.get()].val - self.nodes[l.get()].val;
                    self.nodes[l.get()].right = Some(k);
                    Some(l)
                } else {
                    Some(k)
                },
            },
        }
    }

    fn split(&mut self, n: Option<NonZeroUsize>) -> Option<NonZeroUsize> {
        match n {
            None => None,
            Some(k) => match self.nodes[k.get()].right {
                None => Some(k),
                Some(r) => match self.nodes[r.get()].right {
                    None => Some(k),
                    Some(rr) => if self.nodes[k.get()].level == self.nodes[rr.get()].level {
                        self.nodes[k.get()].right = self.nodes[r.get()].left;
                        self.nodes[r.get()].val =
                            self.nodes[r.get()].val + self.nodes[k.get()].val;
                        self.nodes[r.get()].left = Some(k);
                        self.nodes[r.get()].level += 1;
                        Some(r)
                    } else {
                        Some(k)
                    },
                },
            },
        }
    }
}

impl<K, V> CumlMap for ArneCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn with_capacity(c: usize) -> Self {
        let mut nodes = Vec::with_capacity(c);
        nodes.push(ArneCumlNode {
            key: K::zero(),
            val: V::zero(),
            left: None,
            right: None,
            level: 1,
        });
        ArneCumlTree {
            nodes: nodes,
            root: None,
        }

    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        let l = self.nodes.len();
        let mut p = &mut self.root;
        let mut stack = Vec::new();
        let mut isleft = Vec::new();
        while let Some(i) = *p {
            let n = &mut self.nodes[i.get()];
            stack.push(i.get());
            if k < n.key {
                n.val = n.val + v;
                p = &mut n.left;
                isleft.push(true);
            } else if k > n.key {
                p = &mut n.right;
                isleft.push(false);
            } else {
                n.val = n.val + v;
                return;
            }
        }
        self.nodes.push(ArneCumlNode {
            key: k,
            val: v,
            left: None,
            right: None,
            level: 1,
        });

        let mut ret = NonZeroUsize::new(l);
        while !stack.is_empty() {
            let cur = stack.pop().unwrap();
            let isl = isleft.pop().unwrap();
            if isl {
                self.nodes[cur].left = ret;
            } else {
                self.nodes[cur].right = ret;
            }
            ret = self.skew(NonZeroUsize::new(cur));
            ret = self.split(ret);
        }
        self.root = ret;
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
