extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::ptr;

use cmap::*;

/*****************************************************************************
 * Boxed cumulative frequency tree
 *****************************************************************************/

type Node<K, V> = *mut AARCumlNode<K, V>;

struct AARCumlNode<K, V> {
    index: K,
    val: V,
    left: Node<K, V>,
    right: Node<K, V>,
    level: usize,
}

impl<K, V> AARCumlNode<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn new(k: K, v: V) -> Node<K, V> {
        Box::into_raw(Box::new(AARCumlNode {
            index: k,
            val: v,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            level: 1,
        }))
    }

    unsafe fn insert_node(n: Node<K, V>, k: K, v: V) -> Node<K, V> {
        if n.is_null() {
            AARCumlNode::new(k, v)
        } else {
            if k < (*n).index {
                (*n).val = (*n).val + v;
                (*n).left = AARCumlNode::insert_node((*n).left, k, v)
            } else if k > (*n).index {
                (*n).right = AARCumlNode::insert_node((*n).right, k, v)
            } else {
                (*n).val = (*n).val + v;
                return n;
            }
            AARCumlNode::split_node(AARCumlNode::skew_node(n))
        }
    }

    unsafe fn skew_node(n: Node<K, V>) -> Node<K, V> {
        if n.is_null() {
            ptr::null_mut()
        } else {
            if (*n).left.is_null() {
                n
            } else {
                let l = (*n).left;
                if (*l).level == (*n).level {
                    (*n).left = (*l).right;
                    (*l).right = n;
                    (*n).val = (*n).val - (*l).val;
                    l
                } else {
                    n
                }
            }
        }
    }

    unsafe fn split_node(n: Node<K, V>) -> Node<K, V> {
        if n.is_null() {
            ptr::null_mut()
        } else {
            if (*n).right.is_null() {
                n
            } else {
                let r = (*n).right;
                if (*r).right.is_null() {
                    (*n).right = r;
                    n
                } else {
                    if (*(*r).right).level == (*n).level {
                        (*n).right = (*r).left;
                        (*r).val = (*r).val + (*n).val;
                        (*r).left = n;
                        (*r).level += 1;
                        r
                    } else {
                        n
                    }
                }
            }
        }
    }

    unsafe fn get_total(&self) -> V {
        self.val + if self.right.is_null() {
            V::zero()
        } else {
            (*(*self).right).get_total()
        }
    }

    unsafe fn get_cuml(&self, k: K, acc: V) -> V {
        if k < self.index {
            if self.left.is_null() {
                acc
            } else {
                (*(*self).left).get_cuml(k, acc)
            }
        } else if k > self.index {
            if self.right.is_null() {
                acc + self.val
            } else {
                (*(*self).right).get_cuml(k, acc + self.val)
            }
        } else {
            acc + self.val
        }
    }

    unsafe fn get_single(&self, k: K) -> V {
        if k < self.index {
            if self.left.is_null() {
                V::zero()
            } else {
                (*(*self).left).get_single(k)
            }
        } else if k > self.index {
            if self.right.is_null() {
                V::zero()
            } else {
                (*(*self).right).get_single(k)
            }
        } else {
            if self.left.is_null() {
                self.val
            } else {
                self.val - (*(*self).left).get_total()
            }
        }
    }

    unsafe fn get_quantile(&self, v: V) -> Option<K> {
        if v > self.val {
            if self.right.is_null() {
                None
            } else {
                (*(*self).right).get_quantile(v - self.val)
            }
        } else if v < self.val {
            if self.left.is_null() {
                Some(self.index)
            } else {
                match (*(*self).left).get_quantile(v) {
                    None => Some(self.index),
                    s => s,
                }
            }
        } else {
            Some(self.index)
        }
    }
}

pub struct AARCumlTree<K, V> {
    root: Node<K, V>,
}

impl<K, V> CumlMap for AARCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn with_capacity(_k: usize) -> Self {
        AARCumlTree {
            root: ptr::null_mut(),
        }
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        self.root = unsafe { AARCumlNode::insert_node(self.root, k, v) };
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        if self.root.is_null() {
            Self::Value::zero()
        } else {
            unsafe { (*self.root).get_cuml(k, Self::Value::zero()) }
        }
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        if self.root.is_null() {
            Self::Value::zero()
        } else {
            unsafe { (*self.root).get_single(k) }
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        if self.root.is_null() {
            None
        } else {
            unsafe { (*self.root).get_quantile(quant) }
        }
    }
}
