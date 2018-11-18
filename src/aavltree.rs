extern crate num_traits;
use num_traits::Zero;
use std::ops::{Add, Sub};
use std::cmp::{Ordering};
use std::cmp;

use cmap::*;

#[derive(Debug)]
pub struct AAVLTree<K, V> {
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
    pub fn new() -> AAVLTree<K, V> {
        AAVLTree {
            nodes: Vec::new(),
            root: None,
        }
    }

    pub fn with_capacity(cap: usize) -> AAVLTree<K, V> {
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

    fn get(&self, ix: usize) -> &AAVLNode<K, V> {
        unsafe { self.nodes.get_unchecked(ix) }
    }

    fn get_mut(&mut self, ix: usize) -> &mut AAVLNode<K, V> {
        unsafe { self.nodes.get_unchecked_mut(ix) }
    }
}

impl<K, V> AAVLTree<K, V> {
    fn fix_height(&mut self, ix: usize) {
        let lh = self.get(ix).left.map_or(0, |l| self.get(l).height);
        let rh = self.get(ix).right.map_or(0, |r| self.get(r).height);
        let node = self.get_mut(ix);
        node.height = 1 + cmp::max(lh, rh);
        node.bal = rh as i32 - lh as i32;
    }
}

impl<K, V> AAVLTree<K, V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Ord + Clone,
    K: Ord + Clone,
{
    fn get_total(&self, ix: usize) -> V {
        let cur = self.get(ix);
        cur.val.clone() + cur.right.map_or(V::zero(), |r| self.get_total(r))
    }

    fn left_rotate(&mut self, ix: usize) -> usize {
        let rix = self.get_mut(ix).right.take().unwrap();
        self.get_mut(ix).right = self.get_mut(rix).left.take();
        self.get_mut(rix).left = Some(ix);
        self.fix_height(ix);
        self.fix_height(rix);
        self.get_mut(rix).val = self.get(rix).val.clone() + self.get(ix).val.clone();
        rix
    }

    fn right_rotate(&mut self, ix: usize) -> usize {
        let lix = self.get_mut(ix).left.take().unwrap();
        self.get_mut(ix).left = self.get_mut(lix).right.take();
        self.get_mut(lix).right = Some(ix);
        self.fix_height(ix);
        self.fix_height(lix);
        self.get_mut(ix).val = self.get(ix).val.clone() - self.get(lix).val.clone();
        lix
    }

    fn left_right_rotate(&mut self, ix: usize) -> usize {
        self.get_mut(ix).left = Some(self.left_rotate(self.get(ix).left.unwrap()));
        self.right_rotate(ix)
    }

    fn right_left_rotate(&mut self, ix: usize) -> usize {
        self.get_mut(ix).right = Some(self.right_rotate(self.get(ix).right.unwrap()));
        self.left_rotate(ix)
    }

    fn rebalance(&mut self, ix: usize) -> usize {
        let bal = self.get(ix).bal;
        if bal > 1 {
            let rix = self.get(ix).right.unwrap();
            if self.get(rix).bal > 0 {
                self.left_rotate(ix)
            } else {
                self.right_left_rotate(ix)
            }
        } else if bal < -1 {
            let lix = self.get(ix).left.unwrap();
            if self.get(lix).bal < 0 {
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
        let cur = self.get_mut(ix);
        match (key.cmp(&cur.key), cur.left, cur.right) {
            (Ordering::Less, Some(l), _) => {
                cur.val = cur.val.clone() + val.clone();
                self.get_mut(ix).left = Some(self.rec_insert(l, key.clone(), val.clone()));
                rebal = true;
            },
            (Ordering::Less, _, _) => {
                cur.left = Some(nxtnode);
                mknode = true;
                cur.val = cur.val.clone() + val.clone();
            },
            (Ordering::Greater, _, Some(r)) => {
                self.get_mut(ix).right = Some(self.rec_insert(r, key.clone(), val.clone()));
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

    fn rec_get_cuml(&self, ix: usize, key: K, acc: V) -> V {
        let cur = self.get(ix);
        match (key.cmp(&cur.key), cur.left, cur.right) {
            (Ordering::Less, Some(l), _) => self.rec_get_cuml(l, key, acc),
            (Ordering::Less, _, _) => acc,
            (Ordering::Greater, _, Some(r)) => self.rec_get_cuml(r, key, acc + cur.val.clone()),
            (_, _, _) => acc + cur.val.clone(),
        }
    }

    fn rec_get_single(&self, ix: usize, key: K) -> V {
        let cur = self.get(ix);
        match (key.cmp(&cur.key), cur.left, cur.right) {
            (Ordering::Less, Some(l), _) => self.rec_get_single(l, key),
            (Ordering::Greater, _, Some(r)) => self.rec_get_single(r, key),
            (Ordering::Equal, Some(l), _) => cur.val.clone() - self.get_total(l),
            (Ordering::Equal, _, _) => cur.val.clone(),
            (_, _, _) => V::zero(),
        }
    }

    fn rec_get_quant(&self, ix: usize, v: V) -> Option<K> {
        let cur = self.get(ix);
        match (v.cmp(&cur.val), cur.left, cur.right) {
            (Ordering::Less, Some(l), _) => match self.rec_get_quant(l, v) {
                None => Some(cur.key.clone()),
                s => s,
            },
            (Ordering::Greater, _, Some(r)) => self.rec_get_quant(r, v - cur.val.clone()),
            (Ordering::Greater, _, _) => None,
            (_, _, _) => Some(cur.key.clone()),
        }
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
        match self.root {
            Some(node) => self.root = Some(self.rec_insert(node, key, val)),
            None => self.root = Some(self.mknode(key, val)),
        }
    }

    fn get_cuml(&self, key: K) -> V {
        match self.root {
            Some(node) => self.rec_get_cuml(node, key, V::zero()),
            None => V::zero(),
        }
    }

    fn get_single(&self, key: K) -> V {
        match self.root {
            Some(node) => self.rec_get_single(node, key),
            None => V::zero(),
        }
    }

    fn get_quantile(&self, quant: V) -> Option<K> {
        match self.root {
            Some(node) => self.rec_get_quant(node, quant),
            None => None,
        }
    }
}
