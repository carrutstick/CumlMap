extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::cmp::{Ordering};
use std::cmp;
use std::mem;

use cmap::*;

struct AAVLTree<K, V> {
    nodes: Vec<AAVLNode<K, V>>,
    root: Option<usize>,
}

#[derive(Debug)]
struct AAVLNode<K, V> {
    key: K,
    val: V,
    left: Option<usize>,
    right: Option<usize>,
    height: usize,
    bal: i32,
}

impl<K, V> AAVLTree<K, V> {
    fn new() -> AAVLTree<K, V> {
        AAVLTree {
            nodes: Vec::new(),
            root: None,
        }
    }

    fn with_capacity(cap: usize) -> AAVLTree<K, V> {
        AAVLTree {
            nodes: Vec::with_capacity(cap),
            root: None,
        }
    }

    fn mknode(&mut self, key: K, val: V) -> usize {
        let ix = self.nodes.len();
        self.nodes.push(AAVLNode {
            key: key,
            val: val,
            left: None,
            right: None,
            height: 1,
            bal: 0,
        });
        ix
    }
}

macro_rules! get {
    ($tree: ident, $ix: expr) => (
        (unsafe { $tree.nodes.get_unchecked($ix) })
    )
}
macro_rules! get_mut {
    ($tree: ident, $ix: expr) => (
        (unsafe { $tree.nodes.get_unchecked_mut($ix) })
    )
}
/*
macro_rules! get {
    ($tree: ident, $ix: expr) => (
        (unsafe { $tree.nodes.get($ix).unwrap() })
    )
}
macro_rules! get_mut {
    ($tree: ident, $ix: expr) => (
        (unsafe { $tree.nodes.get_mut($ix).unwrap() })
    )
}
*/

impl<K, V> AAVLNode<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
{
    fn get_total(&self, tree: &AAVLTree<K, V>) -> V {
        self.val.clone() + self.right.map_or(V::zero(), |r| get!(tree, r).get_total(tree))
    }
}

impl<K, V> AAVLTree<K, V> {
    fn fix_height(&mut self, ix: usize) {
        let lh = get!(self, ix).left.map_or(0, |l| get!(self, l).height);
        let rh = get!(self, ix).right.map_or(0, |r| get!(self, r).height);
        let node = get_mut!(self, ix);
        node.height = 1 + cmp::max(lh, rh);
        node.bal = rh as i32 - lh as i32;
    }
}

impl<K, V> AAVLTree<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone,
    K: Ord + Clone,
{
    fn left_rotate(&mut self, ix: usize) -> usize {
        let rix = get_mut!(self, ix).right.take().unwrap();
        get_mut!(self, ix).right = get_mut!(self, rix).left.take();
        get_mut!(self, rix).left = Some(ix);
        self.fix_height(ix);
        self.fix_height(rix);
        get_mut!(self, rix).val = get!(self, rix).val.clone() + get!(self, ix).get_total(self);
        rix
    }

    fn right_rotate(&mut self, ix: usize) -> usize {
        let lix = get_mut!(self, ix).left.take().unwrap();
        get_mut!(self, ix).left = get_mut!(self, lix).right.take();
        get_mut!(self, lix).right = Some(ix);
        self.fix_height(ix);
        self.fix_height(lix);
        get_mut!(self, ix).val = get!(self, ix).val.clone() - get!(self, lix).val.clone();
        lix
    }

    fn left_right_rotate(&mut self, ix: usize) -> usize {
        get_mut!(self, ix).left = Some(self.left_rotate(get!(self, ix).left.unwrap()));
        self.right_rotate(ix)
    }

    fn right_left_rotate(&mut self, ix: usize) -> usize {
        get_mut!(self, ix).right = Some(self.right_rotate(get!(self, ix).right.unwrap()));
        self.left_rotate(ix)
    }

    fn rebalance(&mut self, ix: usize) -> usize {
        let bal = get!(self, ix).bal;
        if bal > 1 {
            let rix = get!(self, ix).right.unwrap();
            if get!(self, rix).bal > 0 {
                self.left_rotate(ix)
            } else {
                self.right_left_rotate(ix)
            }
        } else if bal < -1 {
            let lix = get!(self, ix).left.unwrap();
            if get!(self, lix).bal < 0 {
                self.right_rotate(ix)
            } else {
                self.left_right_rotate(ix)
            }
        } else {
            ix
        }
    }

    fn rec_insert(&mut self, ix: usize, key: K, val: V) -> usize {
        let nxtnode = self.nodes.len();
        let mut mknode = false;
        let mut rebal = false;
        let cur = get_mut!(self, ix);
        match (key.cmp(&cur.key), cur.left, cur.right) {
            (Ordering::Less, Some(l), _) => {
                cur.val = cur.val.clone() + val.clone();
                get_mut!(self, ix).left = Some(self.rec_insert(l, key.clone(), val.clone()));
                rebal = true;
            },
            (Ordering::Less, _, _) => {
                cur.left = Some(nxtnode);
                mknode = true;
                cur.val = cur.val.clone() + val.clone();
            },
            (Ordering::Greater, _, Some(r)) => {
                get_mut!(self, ix).right = Some(self.rec_insert(r, key.clone(), val.clone()));
                rebal = true;
            },
            (Ordering::Greater, _, _) => {
                cur.right = Some(nxtnode);
                mknode = true;
            },
            (Ordering::Equal, _, _) => {
                cur.val = cur.val.clone() + val.clone();
            },
        }
        if mknode { self.mknode(key, val); }
        self.fix_height(ix);
        let res = if rebal { self.rebalance(ix) } else { ix };
        res
    }
}

impl<K, V> CumlMap for AAVLTree<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Clone + Ord,
    K: Add<Output = K> + Sub<Output = K> + Zero + Clone + Ord,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, key: K, val: V) {
        if let Some(node) = self.root {
            self.root = Some(self.rec_insert(node, key, val));
        } else {
            self.root = Some(self.mknode(key, val));
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
    fn aav_build_degen(b: &mut Bencher) {
        b.iter(|| {
            let mut cm = AAVLTree::<i64,i64>::with_capacity(1000);
            for i in 0 .. 1000 {
                cm.insert(i, i);
            }
        });
    }

    #[test]
    fn aav_balance_test() {
        let mut cm = AAVLTree::<i32,i32>::new();
        cm.insert(1, 1);
        cm.insert(2, 1);
        cm.insert(3, 1);
        cm.insert(4, 1);
        cm.insert(5, 1);
        cm.insert(6, 1);
        cm.insert(7, 1);
        assert_eq!(cm.nodes[cm.root.unwrap()].height, 3);
    }
}
