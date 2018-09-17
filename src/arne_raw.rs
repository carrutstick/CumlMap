extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::ptr;
use std::cmp::Ordering;

use cmap::*;

/*****************************************************************************
 * Cumulative frequency tree with raw pointers, Andressen balancing
 *****************************************************************************/

struct AARCumlNode<K, V> {
    index: K,
    val: V,
    left: NodeRef<K, V>,
    right: NodeRef<K, V>,
    level: usize,
}

type Node<K, V> = AARCumlNode<K, V>;
struct NodeRef<K, V>(*mut AARCumlNode<K, V>);

impl<K, V> Clone for NodeRef<K, V> {
    fn clone(&self) -> NodeRef<K, V> {
        NodeRef(self.0)
    }
}

impl<K, V> Copy for NodeRef<K, V> {}

impl<K, V> NodeRef<K, V> {
    fn null() -> NodeRef<K, V> {
        NodeRef(ptr::null_mut())
    }

    fn new(k: K, v: V) -> NodeRef<K, V> {
        NodeRef(Box::into_raw(Box::new(AARCumlNode {
            index: k,
            val: v,
            left: Self::null(),
            right: Self::null(),
            level: 1,
        })))
    }

    unsafe fn free(&mut self) {
        if !self.0.is_null() {
            self.borrow_mut().map(|x| x.left().free());
            self.borrow_mut().map(|x| x.right().free());
            Box::from_raw(self.0);
        }
    }

    fn borrow_mut(&self) -> Option<&mut Node<K, V>> {
        if self.0.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.0) }
        }
    }
}

impl<K, V> Node<K, V> {
    fn left(&self) -> NodeRef<K, V> {
        self.left
    }

    fn right(&self) -> NodeRef<K, V> {
        self.right
    }

    fn level(&self) -> usize {
        self.level
    }

    fn set_val(&mut self, val: V) {
        self.val = val
    }

    fn set_left(&mut self, l: NodeRef<K, V>) {
        self.left = l
    }

    fn set_right(&mut self, r: NodeRef<K, V>) {
        self.right = r
    }

    fn incr_level(&mut self) {
        self.level += 1
    }
}

impl<K: Clone, V> Node<K, V> {
    fn index(&self) -> K {
        self.index.clone()
    }
}

impl<K, V: Clone> Node<K, V> {
    fn val(&self) -> V {
        self.val.clone()
    }
}

impl<K, V> NodeRef<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn insert_node(np: NodeRef<K, V>, k: K, v: V) -> NodeRef<K, V> {
        match np.borrow_mut() {
            None => NodeRef::new(k, v),
            Some(n) => {
                if k < n.index() {
                    n.set_val(n.val() + v);
                    n.set_left(Self::insert_node(n.left(), k, v))
                } else if k > n.index() {
                    n.set_right(Self::insert_node(n.right(), k, v))
                } else {
                    n.set_val(n.val() + v);
                    return np;
                }
                Self::split_node(Self::skew_node(np))
            },
        }
    }

    fn skew_node(np: NodeRef<K, V>) -> NodeRef<K, V> {
        match np.borrow_mut() {
            None => NodeRef::null(),
            Some(n) => {
                let lp = n.left();
                match lp.borrow_mut() {
                    None => np,
                    Some(l) => {
                        if l.level() == n.level() {
                            n.set_left(l.right());
                            l.set_right(np);
                            n.set_val(n.val() - l.val());
                            lp
                        } else {
                            np
                        }
                    },
                }
            },
        }
    }

    fn split_node(np: NodeRef<K, V>) -> NodeRef<K, V> {
        match np.borrow_mut() {
            None => NodeRef::null(),
            Some(n) => {
                let rp = n.right();
                match rp.borrow_mut() {
                    None => np,
                    Some(r) => {
                        match r.right().borrow_mut() {
                            None => np,
                            Some(rr) => {
                                if rr.level() == n.level() {
                                    n.set_right(r.left());
                                    r.set_val(r.val() + n.val());
                                    r.set_left(np);
                                    r.incr_level();
                                    rp
                                } else {
                                    np
                                }
                            },
                        }
                    },
                }
            },
        }
    }
}

impl<K, V> Node<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn get_total(&self) -> V {
        self.val() + match self.right().borrow_mut() {
            None => V::zero(),
            Some(r) => r.get_total(),
        }
    }

    fn get_cuml(&self, k: K, acc: V) -> V {
        match (k.cmp(&self.index()), self.left().borrow_mut(), self.right().borrow_mut()) {
            (Ordering::Less, Some(l), _) => l.get_cuml(k, acc),
            (Ordering::Less, _, _) => acc,
            (Ordering::Greater, _, Some(r)) => r.get_cuml(k, acc + self.val()),
            (_, _, _) => acc + self.val(),
        }
    }

    fn get_single(&self, k: K) -> V {
        match (k.cmp(&self.index()), self.left().borrow_mut(), self.right().borrow_mut()) {
            (Ordering::Equal, Some(l), _) => self.val() - l.get_total(),
            (Ordering::Equal, _, _) => self.val(),
            (Ordering::Less, Some(l), _) => l.get_single(k),
            (Ordering::Greater, _, Some(r)) => r.get_single(k),
            (_, _, _) => V::zero(),
        }
    }

    fn get_quantile(&self, v: V) -> Option<K> {
        match (v.cmp(&self.val()), self.left().borrow_mut(), self.right().borrow_mut()) {
            (Ordering::Less, Some(l), _) => match l.get_quantile(v) {
                None => Some(self.index()),
                s => s,
            },
            (Ordering::Greater, _, Some(r)) => r.get_quantile(v - self.val()),
            (Ordering::Greater, _, None) => None,
            (_, _, _) => Some(self.index()),
        }
    }
}

pub struct AARCumlTree<K, V> {
    root: NodeRef<K, V>,
}

impl<K, V> AARCumlTree<K, V> {
    pub fn new() -> Self {
        AARCumlTree { root: NodeRef::null() }
    }
}

impl<K, V> Drop for AARCumlTree<K, V> {
    fn drop(&mut self) {
        unsafe { self.root.free(); }
    }
}

impl<K, V> CumlMap for AARCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        self.root = NodeRef::insert_node(self.root, k, v);
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        match self.root.borrow_mut() {
            None => V::zero(),
            Some(r) => r.get_cuml(k, V::zero()),
        }
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        match self.root.borrow_mut() {
            None => V::zero(),
            Some(r) => r.get_single(k),
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        match self.root.borrow_mut() {
            None => None,
            Some(r) => r.get_quantile(quant),
        }
    }
}
