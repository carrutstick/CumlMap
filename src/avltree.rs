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
    fn new(key: K, val: V) -> AVLNode<K, V> {
        AVLNode {
            key: key,
            val: val,
            left: None,
            right: None,
            height: 1,
            imbal: 0,
        }
    }

    fn fix_height(&mut self) {
        let lh = match self.left {
            Some(ref l) => l.height,
            None => 0,
        };
        let rh = match self.right {
            Some(ref r) => r.height,
            None => 0,
        };
        self.height = 1 + cmp::max(lh, rh);
        self.imbal = (rh as i64 - lh as i64) as i32;
    }
}

impl<K, V> AVLNode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    fn get_total(&self) -> V {
        self.val.clone() + if let Some(ref r) = self.right {
            r.get_total()
        } else {
            V::zero()
        }
    }

    fn left_rotate(mut self) -> ONode<K, V> {
        let mut r = self.right.take()
            .expect("left_rotate called with no right child");
        r.val = r.val + self.get_total();
        self.right = r.left.take();
        self.fix_height();
        r.left = Some(Box::new(self));
        r.fix_height();
        Some(r)
    }

    fn right_rotate(mut self) -> ONode<K, V> {
        let mut l = self.left.take()
            .expect("left_rotate called with no right child");
        self.val = self.val - l.val.clone();
        self.left = l.right.take();
        self.fix_height();
        l.right = Some(Box::new(self));
        l.fix_height();
        Some(l)
    }

    fn left_right_rotate(mut self) -> ONode<K, V> {
        let l = self.left.take()
            .expect("left_right_rotate called with no left child");
        self.left = l.left_rotate();
        self.right_rotate()
    }

    fn right_left_rotate(mut self) -> ONode<K, V> {
        let r = self.right.take()
            .expect("right_left_rotate called with no right child");
        self.right = r.right_rotate();
        self.left_rotate()
    }

    fn rebalance(self) -> ONode<K, V> {
        if self.imbal > 1 {
            if self.right.as_ref().unwrap().imbal < 0 {
                self.right_left_rotate()
            } else {
                self.left_rotate()
            }
        } else if self.imbal < -1 {
            if self.left.as_ref().unwrap().imbal > 0 {
                self.left_right_rotate()
            } else {
                self.right_rotate()
            }
        } else {
            Some(Box::new(self))
        }
    }
}

impl<K, V> AVLNode<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
{
    fn insert(mut self, key: K, val: V) -> ONode<K, V> {
        match (key.cmp(&self.key), &mut self.left, &mut self.right) {
            (Ordering::Less, Some(_), _) => {
                self.val = self.val.clone() + val.clone();
                self.left = self.left.take().unwrap().insert(key, val);
            },
            (Ordering::Less, _, _) => {
                self.left = Some(Box::new(AVLNode::new(key, val)));
            },
            (Ordering::Greater, _, Some(ref mut r)) => {
                self.right = self.right.take().unwrap().insert(key, val);
            },
            (Ordering::Greater, _, _) => {
                self.right = Some(Box::new(AVLNode::new(key, val)));
            },
            (Ordering::Equal, _, _) => {
                self.val = self.val.clone() + val;
            },
        }
        self.fix_height();
        self.rebalance()
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
        self.root = if let Some(node) = self.root.take() {
            node.insert(key, val)
        } else {
            Some(Box::new(AVLNode::new(key, val)))
        }
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
