extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::ptr;
use std::cmp::{PartialEq, Eq, Ordering};

use cmap::*;

/*****************************************************************************
 * Cumulative frequency tree with raw pointers, Red-Black balancing
 *****************************************************************************/

#[derive(PartialEq,Clone,Copy,Debug)]
enum Color {
    Red,
    Black,
}

struct CumlNode<K, V> {
    index: K,
    val: V,
    left: NodeRef<K, V>,
    right: NodeRef<K, V>,
    parent: NodeRef<K, V>,
    color: Color,
}

type Node<K, V> = CumlNode<K, V>;
struct NodeRef<K, V>(*mut CumlNode<K, V>);

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
        NodeRef(Box::into_raw(Box::new(CumlNode {
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

    fn left_child_eq(&self, other: NodeRef<K, V>) -> bool {
        other == self.left
    }

    fn right_child_eq(&self, other: NodeRef<K, V>) -> bool {
        other == self.right
    }

    fn swap_child(&mut self, old: NodeRef<K, V>, new: NodeRef<K, V>) {
        if old == self.left {
            self.left = new;
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

impl<K, V> Node<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
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

/// The `CumlTree` type. An unbounded mapping between ordered keys and
/// cumulative values, represented as a red-black tree.
pub struct CumlTree<K, V> {
    root: NodeRef<K, V>,
}

impl<K, V> CumlTree<K, V> {
    /// Create an empty `CumlTree` object.
    pub fn new() -> Self {
        CumlTree { root: NodeRef::null() }
    }
}

impl<K, V> Drop for CumlTree<K, V> {
    fn drop(&mut self) {
        unsafe { self.root.free(); }
    }
}

impl<K, V> CumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
{
    fn rb_fix(&mut self, np: NodeRef<K, V>) {
        let nv = np.borrow_mut().unwrap();
        let mut pp = nv.parent();
        if let Some(mut pv) = pp.borrow_mut() {
            if pv.color() == Color::Red {
                let gp = pv.parent();
                let gv = gp.borrow_mut().unwrap();
                let up = gv.other_child(pp);
                if let Some(uv) = up.borrow_mut() {
                    if uv.color() == Color::Red {
                        pv.recolor(Color::Black);
                        uv.recolor(Color::Black);
                        gv.recolor(Color::Red);
                        self.rb_fix(gp);
                    } else {
                        if gv.left_child_eq(pp) && pv.right_child_eq(np) {
                            unsafe { self.left_rotate(pp) };
                            pp = np;
                            pv = nv;
                        } else if gv.right_child_eq(pp) && pv.left_child_eq(np) {
                            unsafe { self.right_rotate(pp) };
                            pp = np;
                            pv = nv;
                        }
                        if gv.left_child_eq(pp) {
                            unsafe { self.right_rotate(gp) };
                        } else {
                            unsafe { self.left_rotate(gp) };
                        }
                        pv.recolor(Color::Black);
                        gv.recolor(Color::Red);
                    }
                } else {
                    if gv.left_child_eq(pp) {
                        unsafe { self.right_rotate(gp) };
                    } else {
                        unsafe { self.left_rotate(gp) };
                    }
                    gv.recolor(Color::Red);
                    pv.recolor(Color::Black);
                }
            }
        } else {
            nv.recolor(Color::Black);
        }
    }

    unsafe fn left_rotate(&mut self, oldn: NodeRef<K, V>) {
        let oldnv = oldn.borrow_mut().unwrap();
        let newn = oldnv.right();
        let newnv = newn.borrow_mut().unwrap();
        oldnv.set_right(newnv.left());
        if let Some(r) = oldnv.right().borrow_mut() {
            r.set_parent(oldn);
        }
        newnv.set_val(newnv.val() + oldnv.val());
        if let Some(p) = oldnv.parent().borrow_mut() {
            p.swap_child(oldn, newn);
            newnv.set_parent(oldnv.parent());
        } else {
            self.root = newn;
            newnv.set_parent(NodeRef::null());
        }
        newnv.set_left(oldn);
        oldnv.set_parent(newn);
    }

    unsafe fn right_rotate(&mut self, oldn: NodeRef<K, V>) {
        let oldnv = oldn.borrow_mut().unwrap();
        let newn = oldnv.left();
        let newnv = newn.borrow_mut().unwrap();
        oldnv.set_left(newnv.right());
        if let Some(l) = oldnv.left().borrow_mut() {
            l.set_parent(oldn);
        }
        oldnv.set_val(oldnv.val() - newnv.val());
        if let Some(p) = oldnv.parent().borrow_mut() {
            p.swap_child(oldn, newn);
            newnv.set_parent(oldnv.parent());
        } else {
            self.root = newn;
            newnv.set_parent(NodeRef::null());
        }
        newnv.set_right(oldn);
        oldnv.set_parent(newn);
    }
}

impl<K, V> CumlMap for CumlTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
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
                    nv.set_val(nv.val() + v.clone());
                    n = nv.left();
                },
                Ordering::Greater => {
                    n = nv.right();
                },
                Ordering::Equal => {
                    nv.set_val(nv.val() + v.clone());
                    return
                },
            }
        }
        n = NodeRef::new(k.clone(), v.clone(), p);
        if let Some(pv) = p.borrow_mut() {
            match k.cmp(&pv.index()) {
                Ordering::Less => pv.set_left(n),
                Ordering::Greater => pv.set_right(n),
                Ordering::Equal => panic!("Cosmic-ray error"),
            }
        } else {
            self.root = n;
        }
        self.rb_fix(n);
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
