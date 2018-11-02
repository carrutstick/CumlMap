extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::cmp::{Ordering};
use std::cmp;

use cmap::*;

type ONode<K, V> = Option<Box<AVLNode<K, V>>>;

struct AVLNode<K, V> {
    key: K,
    val: V,
    left: ONode<K, V>,
    right: ONode<K, V>,
    height: usize,
    imbal: i32,
}

impl<K, V> AVLNode<K, V> {
    fn fix_height(&mut self) {
        let lh = get_height(&self.left);
        let rh = get_height(&self.right);
        self.height = 1 + cmp::max(lh, rh);
        self.imbal = (rh as i64 - lh as i64) as i32;
    }
}

impl<K, V> AVLNode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    fn get_total(&self) -> V {
        self.val.clone() + get_total(&self.right)
    }
}

//-------------------------------------------------------------------
// Structure-only ONode functions

fn newONode<K, V>(key: K, val: V) -> ONode<K, V> {
    Some(Box::new(AVLNode {
        key: key,
        val: val,
        left: None,
        right: None,
        height: 1,
        imbal: 0,
    }))
}

fn get_height<K, V>(onode: &ONode<K, V>) -> usize {
    if let &Some(ref node) = onode {
        node.height
    } else {
        0
    }
}

fn fix_height<K, V>(onode: &mut ONode<K, V>) {
    if let Some(ref mut node) = onode {
        node.fix_height();
    }
}

//-------------------------------------------------------------------
// Functions which manipulate values

fn get_total<K, V>(onode: &ONode<K, V>) -> V
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    if let Some(ref node) = onode {
        node.get_total()
    } else {
        V::zero()
    }
}

fn left_rotate<K, V>(mut onode: ONode<K, V>) -> ONode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    let mut node = onode.as_mut()
        .expect("left_rotate called on empty node");
    let mut r = node.right.take()
        .expect("left_rotate called with no right child");
    node.right = r.left.take();
    node.fix_height();
    r.val = r.val + node.get_total();
    r.left = onode;
    r.fix_height();
    Some(r)
}

fn right_rotate<K, V>(mut onode: ONode<K, V>) -> ONode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    let mut node = onode.as_mut()
        .expect("right_rotate called on empty node");
    let mut l = node.left.take()
        .expect("right_rotate called with no left child");
    node.left = l.right.take();
    node.fix_height();
    node.val = node.val.clone() - l.val.clone();
    l.right = onode;
    l.fix_height();
    Some(l)
}

fn left_right_rotate<K, V>(mut onode: ONode<K, V>) -> ONode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    let node = onode.as_mut()
        .expect("left_right_rotate called on empty node");
    let l = node.left.take();
    node.left = left_rotate(l);
    right_rotate(onode)
}

fn right_left_rotate<K, V>(mut onode: ONode<K, V>) -> ONode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    let node = onode.as_mut()
        .expect("right_left_rotate called on empty node");
    let r = node.right.take();
    node.right = right_rotate(r);
    left_rotate(onode)
}

fn rebalance<K, V>(mut onode: ONode<K, V>) -> ONode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    if let Some(node) = &onode {
        if node.imbal > 1 {
            if node.right.as_ref().unwrap().imbal < 0 {
                right_left_rotate(onode)
            } else {
                left_rotate(onode)
            }
        } else if node.imbal < -1 {
            if node.left.as_ref().unwrap().imbal > 0 {
                left_right_rotate(onode)
            } else {
                right_rotate(onode)
            }
        } else {
            onode
        }
    } else {
        onode
    }
}

//-------------------------------------------------------------------
// 

fn insert<K, V>(mut onode: ONode<K, V>, key: K, val: V) -> ONode<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
{
    if let Some(node) = onode.as_mut() {
        match (key.cmp(&node.key), &mut node.left, &mut node.right) {
            (Ordering::Less, Some(_), _) => {
                node.val = node.val.clone() + val.clone();
                node.left = insert(node.left.take(), key, val);
            },
            (Ordering::Less, _, _) => {
                node.left = newONode(key, val);
            },
            (Ordering::Greater, _, Some(ref mut r)) => {
                node.right = insert(node.right.take(), key, val);
            },
            (Ordering::Greater, _, _) => {
                node.right = newONode(key, val);
            },
            (Ordering::Equal, _, _) => {
                node.val = node.val.clone() + val;
            },
        }
        node.fix_height();
        rebalance(onode)
    } else {
        newONode(key, val)
    }
}

pub struct AVLTree<K, V> {
    root: ONode<K, V>,
}

impl<K, V> AVLTree<K, V> {
    pub fn new() -> Self {
        AVLTree {
            root: None,
        }
    }
}

impl<K, V> CumlMap for AVLTree<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, key: K, val: V) {
        self.root = insert(self.root.take(), key, val)
    }

    fn get_cuml(&self, key: K) -> V {
        unimplemented!()
    }

    fn get_single(&self, key: K) -> V {
        unimplemented!()
    }

    fn get_quantile(&self, quant: V) -> Option<K> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    extern crate test;
    use self::test::Bencher;
    use super::*;

    #[bench]
    fn avl_build_degen(b: &mut Bencher) {
        b.iter(|| {
            let mut cm = AVLTree::<i32,i32>::new();
            for i in 0 .. 1000 {
                cm.insert(i, i);
            }
        });
    }

    #[test]
    fn avl_balance_test() {
        let mut cm = AVLTree::<i32,i32>::new();
        cm.insert(1, 1);
        cm.insert(2, 1);
        cm.insert(3, 1);
        cm.insert(4, 1);
        cm.insert(5, 1);
        cm.insert(6, 1);
        cm.insert(7, 1);
        assert_eq!(cm.root.as_ref().unwrap().height, 3);
        assert_eq!(cm.root.as_ref().unwrap().imbal, 0);
        assert_eq!(cm.root.as_ref().unwrap().key, 4);
        assert_eq!(cm.root.as_ref().unwrap().right.as_ref().unwrap().key, 6);
    }
}
