extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::ptr;
use std::mem;
use std::cmp::{PartialEq, Eq};

use cmap::*;

/*****************************************************************************
 * Cumulative frequency tree with raw pointers, Andressen balancing
 *****************************************************************************/

#[derive(PartialEq,Clone,Copy)]
enum Color {
    Red,
    Black,
}

struct RBCumlNode<K, V> {
    index: K,
    val: V,
    left: Node<K, V>,
    right: Node<K, V>,
    parent: Node<K, V>,
    color: Color,
}

struct Node<K, V>(*mut RBCumlNode<K, V>);

impl<K, V> Clone for Node<K, V> {
    fn clone(&self) -> Node<K, V> {
        Node(self.0)
    }
}

impl<K, V> PartialEq for Node<K, V> {
    fn eq(&self, other: &Node<K, V>) -> bool {
        self.0 == other.0
    }
}

impl<K, V> Eq for Node<K, V> {}

impl<K, V> Copy for Node<K, V> {}

impl<K, V> Node<K, V> {
    fn null() -> Node<K, V> {
        Node(ptr::null_mut())
    }

    fn new(k: K, v: V, p: Node<K, V>) -> Node<K, V> {
        Node(Box::into_raw(Box::new(RBCumlNode {
            index: k,
            val: v,
            left: Self::null(),
            right: Self::null(),
            parent: p,
            color: Color::Red,
        })))
    }

    unsafe fn free(&mut self) {
        if !self.is_null() {
            self.left().free();
            self.right().free();
            Box::from_raw(self.0);
        }
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

    unsafe fn parent(&self) -> Node<K, V> {
        (*self.0).parent
    }

    unsafe fn color(&mut self) -> Color {
        (*self.0).color
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

    unsafe fn set_parent(&mut self, p: Node<K, V>) {
        (*self.0).parent = p
    }

    unsafe fn recolor(&mut self, c: Color) {
        (*self.0).color = c
    }

    unsafe fn swap_child(&mut self, old: Node<K, V>, new: Node<K, V>) {
        if old == self.left() {
            self.set_left(new);
        } else if old == self.right() {
            self.set_right(new);
        } else {
            panic!("Trying to swap child, but node given is not current child");
        }
    }

    unsafe fn other_child(&self, c: Node<K, V>) -> Node<K, V> {
        if c == self.left() {
            self.right()
        } else if c == self.right() {
            self.left()
        } else {
            panic!("Trying to pick other child, but node given is not current child");
        }
    }
}

impl<K: Copy, V> Node<K, V> {
    unsafe fn index(&self) -> K {
        (*self.0).index
    }
}

impl<K, V: Copy> Node<K, V> {
    unsafe fn val(&self) -> V {
        (*self.0).val
    }
}

impl<K, V> Node<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn get_total(&self) -> V {
        if self.is_null() {
            V::zero()
        } else {
            unsafe { self.val() + self.right().get_total() }
        }
    }

    fn get_cuml(&self, k: K, acc: V) -> V {
        if self.is_null() {
            acc
        } else {
            unsafe {
                if k < self.index() {
                    self.left().get_cuml(k, acc)
                } else if k > self.index() {
                    self.right().get_cuml(k, acc + self.val())
                } else {
                    acc + self.val()
                }
            }
        }
    }

    fn get_single(&self, k: K) -> V {
        if self.is_null() {
            V::zero()
        } else {
            unsafe {
                if k < self.index() {
                    self.left().get_single(k)
                } else if k > self.index() {
                    self.right().get_single(k)
                } else {
                    self.val() - self.left().get_total()
                }
            }
        }
    }

    fn get_quantile(&self, v: V) -> Option<K> {
        if self.is_null() {
            None
        } else {
            unsafe {
                if v > self.val() {
                    self.right().get_quantile(v - self.val())
                } else if v < self.val() {
                    match self.left().get_quantile(v) {
                        None => Some(self.index()),
                        s => s,
                    }
                } else {
                    Some(self.index())
                }
            }
        }
    }
}

pub struct RBCumlTree<K, V> {
    root: Node<K, V>,
}

impl<K, V> Drop for RBCumlTree<K, V> {
    fn drop(&mut self) {
        unsafe { self.root.free(); }
    }
}

impl<K, V> RBCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    unsafe fn rb_fix(&mut self, mut n: Node<K, V>) {
        if n.parent().is_null() {
            n.recolor(Color::Black);
        } else if n.parent().color() == Color::Red {
            let mut p = n.parent();
            let mut g = p.parent(); // dad is red, so must have granddad
            let mut u = g.other_child(p);
            if u.is_null() {
                self.right_rotate(g);
                g.recolor(Color::Red);
                p.recolor(Color::Black);
            } else {
                if u.color() == Color::Red {
                    p.recolor(Color::Black);
                    u.recolor(Color::Black);
                    g.recolor(Color::Red);
                    self.rb_fix(g);
                } else {
                    if n == g.left().right() {
                        self.left_rotate(p);
                        mem::swap(&mut p, &mut n);
                    } else if n == g.right().left() {
                        self.right_rotate(p);
                        mem::swap(&mut p, &mut n);
                    }

                    if p == g.left() {
                        self.right_rotate(g);
                    } else {
                        self.left_rotate(g);
                    }
                    p.recolor(Color::Black);
                    g.recolor(Color::Red);
                }
            }
        }
    }

    unsafe fn left_rotate(&mut self, mut oldn: Node<K, V>) {
        let mut newn = oldn.right();
        oldn.set_right(newn.left());
        oldn.right().set_parent(oldn);
        let mut par = oldn.parent();
        if par.is_null() {
            self.root = newn;
            newn.set_parent(Node::null());
        } else {
            par.swap_child(oldn, newn);
            newn.set_parent(par);
        }
        newn.set_left(oldn);
        oldn.set_parent(newn);
    }

    unsafe fn right_rotate(&mut self, mut oldn: Node<K, V>) {
        let mut newn = oldn.left();
        oldn.set_left(newn.right());
        oldn.left().set_parent(oldn);
        let mut par = oldn.parent();
        if par.is_null() {
            self.root = newn;
            newn.set_parent(Node::null());
        } else {
            par.swap_child(oldn, newn);
            newn.set_parent(par);
        }
        newn.set_right(oldn);
        oldn.set_parent(newn);
    }
}

impl<K, V> CumlMap for RBCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn with_capacity(_k: usize) -> Self {
        RBCumlTree { root: Node::null() }
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        let mut n = self.root;
        let mut p = Node::null();
        unsafe {
            while !n.is_null() {
                p = n;
                if k < n.index() {
                    n = n.left()
                } else if k > n.index() {
                    n = n.right()
                } else {
                    n.set_val(n.val() + v);
                }
            }
            n = Node::new(k, v, p);
            if p.is_null() {
                self.root = n;
            } else if k < p.index() {
                p.set_left(n);
            } else if k > p.index() {
                p.set_right(n);
            } else {
                panic!("Cosmic-ray error");
            }
            self.rb_fix(n);
        }
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        self.root.get_cuml(k, Self::Value::zero())
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        self.root.get_single(k)
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        self.root.get_quantile(quant)
    }
}
