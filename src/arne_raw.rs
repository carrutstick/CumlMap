extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::ptr;

use cmap::*;

/*****************************************************************************
 * Boxed cumulative frequency tree
 *****************************************************************************/

struct AARCumlNode<K, V> {
    index: K,
    val: V,
    left: Node<K, V>,
    right: Node<K, V>,
    level: usize,
}

struct Node<K, V>(*mut AARCumlNode<K, V>);

impl<K, V> Clone for Node<K, V> {
    fn clone(&self) -> Node<K, V> {
        Node ( self.0 )
    }
}

impl<K, V> Copy for Node<K, V> {}

impl<K, V> Node<K, V> {
    fn null() -> Node<K, V> {
        Node ( ptr::null_mut() )
    }

    fn new(k: K, v: V) -> Node<K, V> {
        Node (
            Box::into_raw(
                Box::new(
                    AARCumlNode {
                        index: k,
                        val: v,
                        left: Self::null(),
                        right: Self::null(),
                        level: 1,
                    }
                )
            )
        )
    }

    fn is_null(&self) -> bool {
        self.0.is_null()
    }

    unsafe fn left(&self) -> Node<K, V> {
        (*self.0).left
    }

    unsafe fn right(&self) -> Node<K, V> {
        (*self.0).right
    }

    unsafe fn level(&self) -> usize {
        (*self.0).level
    }

    unsafe fn set_val(&mut self, val: V) {
        (*self.0).val = val
    }

    unsafe fn set_left(&mut self, l: Node<K, V>) {
        (*self.0).left = l
    }

    unsafe fn set_right(&mut self, r: Node<K, V>) {
        (*self.0).right = r
    }

    unsafe fn incr_level(&mut self) {
        (*self.0).level += 1
    }
}

impl<K, V> Node<K, V> where K: Copy {
    unsafe fn index(&self) -> K {
        (*self.0).index
    }
}

impl<K, V> Node<K, V> where V: Copy {
    unsafe fn val(&self) -> V {
        (*self.0).val
    }
}

impl<K, V> Node<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn insert_node(mut n: Node<K, V>, k: K, v: V) -> Node<K, V> {
        if n.is_null() {
            Node::new(k, v)
        } else {
            unsafe {
                if k < n.index() {
                    n.set_val(n.val() + v);
                    n.set_left(Self::insert_node(n.left(), k, v))
                } else if k > n.index() {
                    n.set_right(Self::insert_node(n.right(), k, v))
                } else {
                    n.set_val(n.val() + v);
                    return n;
                }
                Self::split_node(Self::skew_node(n))
            }
        }
    }

    fn skew_node(mut n: Node<K, V>) -> Node<K, V> {
        if n.is_null() {
            n
        } else {
            unsafe {
                if n.left().is_null() {
                    n
                } else {
                    let mut l = n.left();
                    if l.level() == n.level() {
                        n.set_left(l.right());
                        l.set_right(n);
                        n.set_val(n.val() - l.val());
                        l
                    } else {
                        n
                    }
                }
            }
        }
    }

    fn split_node(mut n: Node<K, V>) -> Node<K, V> {
        if n.is_null() {
            n
        } else {
            unsafe {
                if n.right().is_null() {
                    n
                } else {
                    let mut r = n.right();
                    if r.right().is_null() {
                        n
                    } else {
                        if r.right().level() == n.level() {
                            n.set_right(r.left());
                            r.set_val(r.val() + n.val());
                            r.set_left(n);
                            r.incr_level();
                            r
                        } else {
                            n
                        }
                    }
                }
            }
        }
    }

    unsafe fn get_total(&self) -> V {
        self.val() + if self.right().is_null() {
            V::zero()
        } else {
            self.right().get_total()
        }
    }

    unsafe fn get_cuml(&self, k: K, acc: V) -> V {
        if k < self.index() {
            if self.left().is_null() {
                acc
            } else {
                self.left().get_cuml(k, acc)
            }
        } else if k > self.index() {
            if self.right().is_null() {
                acc + self.val()
            } else {
                self.right().get_cuml(k, acc + self.val())
            }
        } else {
            acc + self.val()
        }
    }

    unsafe fn get_single(&self, k: K) -> V {
        if k < self.index() {
            if self.left().is_null() {
                V::zero()
            } else {
                self.left().get_single(k)
            }
        } else if k > self.index() {
            if self.right().is_null() {
                V::zero()
            } else {
                self.right().get_single(k)
            }
        } else {
            if self.left().is_null() {
                self.val()
            } else {
                self.val() - self.left().get_total()
            }
        }
    }

    unsafe fn get_quantile(&self, v: V) -> Option<K> {
        if v > self.val() {
            if self.right().is_null() {
                None
            } else {
                self.right().get_quantile(v - self.val())
            }
        } else if v < self.val() {
            if self.left().is_null() {
                Some(self.index())
            } else {
                match self.left().get_quantile(v) {
                    None => Some(self.index()),
                    s => s,
                }
            }
        } else {
            Some(self.index())
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
            root: Node::null(),
        }
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        self.root = Node::insert_node(self.root, k, v);
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        if self.root.is_null() {
            Self::Value::zero()
        } else {
            unsafe { self.root.get_cuml(k, Self::Value::zero()) }
        }
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        if self.root.is_null() {
            Self::Value::zero()
        } else {
            unsafe { self.root.get_single(k) }
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        if self.root.is_null() {
            None
        } else {
            unsafe { self.root.get_quantile(quant) }
        }
    }
}
