#![feature(test)]
#![feature(nll)]
extern crate num_traits;

use num_traits::Zero;
use std::ops::{Add, Sub};

pub trait CumlMap {
    type Key;
    type Value;

    fn with_capacity(usize) -> Self;
    fn insert(&mut self, Self::Key, Self::Value);
    fn get_cuml(&self, Self::Key) -> Self::Value;
    fn get_single(&self, Self::Key) -> Self::Value;
    fn get_quantile(&self, Self::Value) -> Option<Self::Key>;
}

/*****************************************************************************
 * Cumulative Frequency Table, per Simon Tatham
 *****************************************************************************/

struct CumlFreqTable<V> {
    capacity: usize,
    total: V,
    tables: Vec<Vec<V>>,
}

impl<V> CumlMap for CumlFreqTable<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = usize;
    type Value = V;

    fn with_capacity(c: usize) -> CumlFreqTable<V> {
        let cap = c.next_power_of_two();
        let mut ret = CumlFreqTable {
            capacity: cap,
            total: V::zero(),
            tables: Vec::new(),
        };
        let mut column_size = 1;
        while column_size < cap {
            ret.tables.push(vec![V::zero(); column_size]);
            column_size = column_size << 1;
        }
        ret
    }

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        assert!(key < self.capacity);
        self.total = self.total + val;
        let mut bit: usize = self.tables.len();
        for ref mut tbl in self.tables.iter_mut() {
            bit -= 1;
            if (key & (1 << bit)) != 0 {
                continue;
            }
            let j = key >> (bit + 1);
            tbl[j] = tbl[j] + val;
        }
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        if key >= self.capacity - 1 {
            return self.total;
        }
        let key = key + 1;
        let mut acc: Self::Value = Self::Value::zero();
        let mut bit: usize = self.tables.len();
        for ref tbl in self.tables.iter() {
            bit -= 1;
            if (key & (1 << bit)) == 0 {
                continue;
            }
            let j = key >> (bit + 1);
            acc = acc + tbl[j];
        }
        acc
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
        assert!(key < self.capacity);
        if key > 0 {
            self.get_cuml(key) - self.get_cuml(key - 1)
        } else {
            self.tables[self.tables.len() - 1][0]
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        if quant > self.total { return None }
        let mut index = 0;
        let mut acc = Self::Value::zero();
        for ref tbl in self.tables.iter() {
            if tbl[index] + acc >= quant {
                index = index << 1
            } else {
                acc = acc + tbl[index];
                index = (index << 1) + 1
            }
        }
        Some(index)
    }
}

/*****************************************************************************
 * Binary Index Tree, per Peter Fenwick
 *****************************************************************************/

struct BinaryIndexTree<V> {
    capacity: usize,
    data: Vec<V>,
}

impl<V> CumlMap for BinaryIndexTree<V>
where
    V: Add<Output = V> + Sub<Output = V> + Zero + Copy + Ord,
{
    type Key = usize;
    type Value = V;

    fn with_capacity(c: usize) -> BinaryIndexTree<V> {
        BinaryIndexTree {
            capacity: c,
            data: vec![V::zero(); c],
        }
    }

    fn insert(&mut self, key: Self::Key, val: Self::Value) {
        assert!(key < self.capacity);
        let mut key = key;
        while key < self.capacity {
            self.data[key as usize] = self.data[key as usize] + val;
            if key == 0 {
                break;
            }
            key += 1 << key.trailing_zeros();
        }
    }

    fn get_cuml(&self, key: Self::Key) -> Self::Value {
        assert!(key < self.capacity);
        let mut key = key;
        let mut sum = self.data[0];
        while key > 0 {
            sum = sum + self.data[key];
            key = key & (key - 1);
        }
        sum
    }

    fn get_single(&self, key: Self::Key) -> Self::Value {
        let mut val = self.data[key];
        let mut key = key;
        if key == 0 {
            return val;
        }
        let parent = key & (key - 1);
        key -= 1;
        while parent != key {
            val = val - self.data[key];
            key = key & (key - 1);
        }
        val
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        let mut index = 0;
        let mut mask = self.capacity / 2;
        let mut quant = quant - self.data[0];
        while mask != 0 {
            let test = index + mask;
            if quant >= self.data[test] {
                quant = quant - self.data[test];
                index = test;
            }
            mask >>= 1;
        }
        if quant > Self::Value::zero() {
            if index + 1 < self.capacity {
                Some(index + 1)
            } else {
                None
            }
        } else {
            Some(index)
        }
    }
}

/*****************************************************************************
 * Boxed cumulative frequency tree
 *****************************************************************************/

struct BoxedCumlNode<K,V> {
    index: K,
    val: V,
    left: Option<Box<BoxedCumlNode<K,V>>>,
    right: Option<Box<BoxedCumlNode<K,V>>>,
}

impl<K,V> BoxedCumlNode<K,V>
where
    K: Add<Output=K> + Sub<Output=K> + Zero + Copy + Ord,
    V: Add<Output=V> + Sub<Output=V> + Zero + Copy + Ord,
{
    fn new(k: K, v: V) -> BoxedCumlNode<K, V>
    {
        BoxedCumlNode { index: k, val: v, left: None, right: None }
    }

    fn get_total(&self) -> V {
        self.val + match self.right {
            None => V::zero(),
            Some(ref r) => r.get_total()
        }
    }

    fn insert(&mut self, k: K, v: V) {
        if k < self.index {
            self.val = self.val + v;
            match self.left {
                None => self.left = Some(Box::new(Self::new(k, v))),
                Some(ref mut l) => l.insert(k, v),
            }
        } else if k > self.index {
            match self.right {
                None => self.right = Some(Box::new(Self::new(k, v))),
                Some(ref mut r) => r.insert(k, v),
            }
        } else {
            self.val = self.val + v
        }
    }

    fn get_cuml(&self, k: K, acc: V) -> V {
        if k < self.index {
            match self.left {
                None => acc,
                Some(ref l) => l.get_cuml(k, acc),
            }
        } else if k > self.index {
            match self.right {
                None => acc + self.val,
                Some(ref r) => r.get_cuml(k, acc + self.val),
            }
        } else {
            acc + self.val
        }
    }

    fn get_single(&self, k: K) -> V {
        if k < self.index {
            match self.left {
                None => V::zero(),
                Some(ref l) => l.get_single(k),
            }
        } else if k > self.index {
            match self.right {
                None => V::zero(),
                Some(ref r) => r.get_single(k),
            }
        } else {
            match self.left {
                None => self.val,
                Some(ref l) => self.val - l.get_total(),
            }
        }
    }

    fn get_quantile(&self, v: V) -> Option<K> {
        if v > self.val {
            match self.right {
                None => None,
                Some(ref r) => r.get_quantile(v - self.val),
            }
        } else if v < self.val {
            match self.left {
                None => Some(self.index),
                Some(ref l) => match l.get_quantile(v) {
                    None => Some(self.index),
                    s => s,
                },
            }
        } else {
            Some(self.index)
        }
    }
}

struct BoxedCumlTree<K,V> {
    root: Option<Box<BoxedCumlNode<K,V>>>,
}

impl<K,V> CumlMap for BoxedCumlTree<K,V>
where
    K: Add<Output=K> + Sub<Output=K> + Zero + Copy + Ord,
    V: Add<Output=V> + Sub<Output=V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn with_capacity(_k: usize) -> Self {
        BoxedCumlTree { root: None }
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        match self.root {
            Some(ref mut n) => n.insert(k, v),
            None => self.root = Some(Box::new(BoxedCumlNode::new(k, v))),
        }
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        match self.root {
            Some(ref n) => n.get_cuml(k, Self::Value::zero()),
            None => Self::Value::zero(),
        }
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        match self.root {
            Some(ref n) => n.get_single(k),
            None => Self::Value::zero(),
        }
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        match self.root {
            Some(ref n) => n.get_quantile(quant),
            None => None,
        }
    }
}

/*****************************************************************************
 * Tree structure with preallocated arena of nodes
 *****************************************************************************/

struct ArenaCumlNode<K,V> {
    key: K,
    val: V,
    left: Option<usize>,
    right: Option<usize>,
}

struct ArenaCumlTree<K,V> {
    nodes: Vec<ArenaCumlNode<K,V>>,
    root: Option<usize>,
}

impl<K,V> ArenaCumlTree<K,V>
where
    V: Add<Output=V> + Sub<Output=V> + Zero + Copy + Ord,
{
    fn get_total(&self, n: &Option<usize>) -> V {
        let mut p = n;
        let mut acc = V::zero();
        while let Some(i) = *p {
            let n = &self.nodes[i];
            acc = acc + n.val;
            p = &n.right;
        }
        acc
    }
}

impl<K,V> CumlMap for ArenaCumlTree<K,V>
where
    K: Add<Output=K> + Sub<Output=K> + Zero + Copy + Ord,
    V: Add<Output=V> + Sub<Output=V> + Zero + Copy + Ord,
{
    type Key = K;
    type Value = V;

    fn with_capacity(c: usize) -> Self {
        ArenaCumlTree { nodes: Vec::with_capacity(c), root: None }
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) {
        let l = self.nodes.len();
        let mut p = &mut self.root;
        while let Some(i) = *p {
            let n = &mut self.nodes[i];
            if k < n.key {
                n.val = n.val + v;
                p = &mut n.left;
            } else if k > n.key {
                p = &mut n.right;
            } else {
                n.val = n.val + v;
                return;
            }
        }
        *p = Some(l);
        self.nodes.push(ArenaCumlNode { key: k, val: v, left: None, right: None });
    }

    fn get_cuml(&self, k: Self::Key) -> Self::Value {
        let mut acc = Self::Value::zero();
        let mut p = &self.root;
        while let Some(i) = *p {
            let n = &self.nodes[i];
            if k < n.key {
                p = &n.left;
            } else if k > n.key {
                acc = acc + n.val;
                p = &n.right;
            } else {
                return acc + n.val;
            }
        }
        acc
    }

    fn get_single(&self, k: Self::Key) -> Self::Value {
        let mut p = &self.root;
        while let Some(i) = *p {
            let n = &self.nodes[i];
            if k < n.key {
                p = &n.left;
            } else if k > n.key {
                p = &n.right;
            } else {
                return n.val - self.get_total(&n.left)
            }
        }
        Self::Value::zero()
    }

    fn get_quantile(&self, quant: Self::Value) -> Option<Self::Key> {
        let mut p = &self.root;
        let mut lastabove = None;
        let mut last = None;
        let mut q = quant;
        while let Some(i) = *p {
            let n = &self.nodes[i];
            if q < n.val {
                lastabove = Some(n.key);
                p = &n.left
            } else if q > n.val {
                last = Some(n.key);
                q = q - n.val;
                p = &n.right
            } else {
                return Some(n.key)
            }
        }
        if q > Self::Value::zero() {
            lastabove
        } else {
            last
        }
    }
}


/*****************************************************************************
 * Tests, etc.
 *****************************************************************************/

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use self::test::Bencher;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn test_trivial<T>()
        where T: CumlMap<Key=usize,Value=i32>,
    {
        let mut t = T::with_capacity(4);
        t.insert(0, 1);
        t.insert(1, 2);
        t.insert(2, 3);
        t.insert(3, 5);

        assert_eq!(t.get_single(0), 1);
        assert_eq!(t.get_single(1), 2);
        assert_eq!(t.get_single(2), 3);
        assert_eq!(t.get_single(3), 5);

        assert_eq!(t.get_cuml(0), 1);
        assert_eq!(t.get_cuml(1), 3);
        assert_eq!(t.get_cuml(2), 6);
        assert_eq!(t.get_cuml(3), 11);

        assert_eq!(t.get_quantile(1), Some(0));
        assert_eq!(t.get_quantile(2), Some(1));
        assert_eq!(t.get_quantile(3), Some(1));
        assert_eq!(t.get_quantile(4), Some(2));
        assert_eq!(t.get_quantile(6), Some(2));
        assert_eq!(t.get_quantile(10), Some(3));
    }

    #[test]
    fn trivial_cft() {
        test_trivial::<CumlFreqTable<i32>>();
    }

    #[test]
    fn trivial_bix() {
        test_trivial::<BinaryIndexTree<i32>>();
    }

    #[test]
    fn trivial_dct() {
        test_trivial::<BoxedCumlTree<usize,i32>>();
    }

    #[test]
    fn trivial_act() {
        test_trivial::<ArenaCumlTree<usize,i32>>();
    }

    fn load_updates(fname: &str) -> (usize, Vec<usize>, Vec<i32>) {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;

        let fp = File::open(fname).expect("Could not open file");
        let mut reader = BufReader::new(fp);
        let mut line = String::new();

        let _ = reader.read_line(&mut line);
        let cap = line.trim().parse::<usize>().expect("Bad parse");
        let mut keys = Vec::new();
        let mut vals = Vec::new();

        for line in reader.lines() {
            let kv : Vec<i32> = line
                .expect("Could not read line")
                .split(" ")
                .map(|x| x.trim().parse::<i32>()
                     .expect("Bad parse"))
                .collect();
            keys.push(kv[0] as usize);
            vals.push(kv[1]);
        }

        (cap, keys, vals)
    }

    fn benchmark_from_file<T>(fname: &str, b: &mut Bencher)
        where T: CumlMap<Key=usize,Value=i32>,
    {
        let (cap, keys, vals) = load_updates(fname);
        b.iter(|| {
            let mut cm = T::with_capacity(cap);
            for i in 1..keys.len() {
                cm.insert(keys[i], vals[i]);
            }
        })
    }

    #[bench]
    fn cft_bench_1_build(b: &mut Bencher) {
        benchmark_from_file::<CumlFreqTable<i32>>("src/bench_1", b);
    }

    #[bench]
    fn bix_bench_1_build(b: &mut Bencher) {
        benchmark_from_file::<BinaryIndexTree<i32>>("src/bench_1", b);
    }

    #[bench]
    fn dct_bench_1_build(b: &mut Bencher) {
        benchmark_from_file::<BoxedCumlTree<usize,i32>>("src/bench_1", b);
    }

    #[bench]
    fn act_bench_1_build(b: &mut Bencher) {
        benchmark_from_file::<ArenaCumlTree<usize,i32>>("src/bench_1", b);
    }
}

