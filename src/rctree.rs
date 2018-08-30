extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::ptr;
use std::cmp::{PartialEq, Eq, Ordering};

use cmap::*;

/*****************************************************************************
 * Cumulative frequency tree with raw pointers, Andressen balancing
 *****************************************************************************/

#[derive(PartialEq,Clone,Copy,Debug)]
enum Color {
    Red,
    Black,
}

struct RCCumlNode<K, V> {
    index: K,
    val: V,
    left: NodeRef<K, V>,
    right: NodeRef<K, V>,
    parent: NodeRef<K, V>,
    color: Color,
}

type Node<K, V> = RCCumlNode<K, V>;
struct NodeRef<K, V>(*mut RCCumlNode<K, V>);

impl<K, V> Clone for NodeRef<K, V> {
    fn clone(&self) -> NodeRef<K, V> {
        NodeRef(self.0)
    }
}

impl<K, V> PartialEq for NodeRef<K, V> {
    fn eq(&self, other: &NodeRef<K, V>) -> bool {
        self.0 == other.0
    }
}

impl<K, V> Eq for NodeRef<K, V> {}

impl<K, V> Copy for NodeRef<K, V> {}

impl<K, V> NodeRef<K, V> {
    fn null() -> NodeRef<K, V> {
        NodeRef(ptr::null_mut())
    }

    fn new(k: K, v: V, p: NodeRef<K, V>) -> NodeRef<K, V> {
        NodeRef(Box::into_raw(Box::new(RCCumlNode {
            index: k,
            val: v,
            left: Self::null(),
            right: Self::null(),
            parent: p,
            color: Color::Red,
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

    fn parent(&self) -> NodeRef<K, V> {
        self.parent
    }

    fn color(&self) -> Color {
        self.color
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

    fn set_parent(&mut self, p: NodeRef<K, V>) {
        self.parent = p
    }

    fn recolor(&mut self, c: Color) {
        self.color = c
    }

    fn swap_child(&mut self, old: NodeRef<K, V>, new: NodeRef<K, V>) {
        if old == self.left {
            self.right = new;
        } else if old == self.right {
            self.right = new;
        } else {
            panic!("Trying to swap child, but node given is not current child");
        }
    }

    fn other_child(&self, c: NodeRef<K, V>) -> NodeRef<K, V> {
        if c == self.left {
            self.right
        } else if c == self.right {
            self.left
        } else {
            panic!("Trying to pick other child, but node given is not current child");
        }
    }

    fn left_child_eq(&self, other: NodeRef<K, V>) -> bool {
        other == self.left
    }

    fn right_child_eq(&self, other: NodeRef<K, V>) -> bool {
        other == self.right
    }
}

impl<K: Copy, V> Node<K, V> {
    fn index(&self) -> K {
        self.index
    }
}

impl<K, V: Copy> Node<K, V> {
    fn val(&self) -> V {
        self.val
    }
}

impl<K, V> Node<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    fn get_total(&self) -> V {
        match self.right().borrow_mut() {
            None => self.val(),
            Some(r) => self.val() + r.get_total(),
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

pub struct RCCumlTree<K, V> {
    root: NodeRef<K, V>,
}

impl<K, V> RCCumlTree<K, V> {
    pub fn new() -> Self {
        RCCumlTree { root: NodeRef::null() }
    }
}

impl<K, V> Drop for RCCumlTree<K, V> {
    fn drop(&mut self) {
        unsafe { self.root.free(); }
    }
}

/*
impl<K, V> RCCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    unsafe fn rb_fix(&mut self, mut n: NodeRef<K, V>) {
        if n.parent().is_null() {
            n.recolor(Color::Black);
        } else if n.parent().color() == Color::Red {
            let mut p = n.parent();
            let mut g = p.parent(); // dad is red, so must have granddad
            let mut u = g.other_child(p);
            if u.is_null() {
                if p == g.left() {
                    self.right_rotate(g);
                } else {
                    self.left_rotate(g);
                }
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

    unsafe fn left_rotate(&mut self, mut oldn: NodeRef<K, V>) {
        let mut newn = oldn.right();
        oldn.set_right(newn.left());
        if !oldn.right().is_null() {
            oldn.right().set_parent(oldn);
        }
        newn.set_val(newn.val() + oldn.val());
        let mut par = oldn.parent();
        if par.is_null() {
            self.root = newn;
            newn.set_parent(NodeRef::null());
        } else {
            par.swap_child(oldn, newn);
            newn.set_parent(par);
        }
        newn.set_left(oldn);
        oldn.set_parent(newn);
    }

    unsafe fn right_rotate(&mut self, mut oldn: NodeRef<K, V>) {
        let mut newn = oldn.left();
        oldn.set_left(newn.right());
        if !oldn.left().is_null() {
            oldn.left().set_parent(oldn);
        }
        oldn.set_val(oldn.val() - newn.val());
        let mut par = oldn.parent();
        if par.is_null() {
            self.root = newn;
            newn.set_parent(NodeRef::null());
        } else {
            par.swap_child(oldn, newn);
            newn.set_parent(par);
        }
        newn.set_right(oldn);
        oldn.set_parent(newn);
    }
}
*/

impl<K, V> CumlMap for RCCumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Copy + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        let mut n = self.root;
        let mut p = NodeRef::null();
        while let Some(nv) = n.borrow_mut() {
            p = n;
            match k.cmp(&nv.index()) {
                Ordering::Less => {
                    nv.set_val(nv.val() + v);
                    n = nv.left();
                },
                Ordering::Greater => {
                    n = nv.right();
                },
                Ordering::Equal => {
                    nv.set_val(nv.val() + v);
                    return
                },
            }
        }
        n = NodeRef::new(k, v, p);
        if let Some(pv) = p.borrow_mut() {
            match k.cmp(&pv.index()) {
                Ordering::Less => pv.set_left(n),
                Ordering::Greater => pv.set_right(n),
                Ordering::Equal => panic!("Cosmic-ray error"),
            }
        } else {
            self.root = n;
        }
        // self.rb_fix(n);
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
