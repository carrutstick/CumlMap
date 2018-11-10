extern crate test;
use self::test::Bencher;
use super::*;

macro_rules! test_trivial {
    ($testn:ident, $init:expr) => {
        #[test]
        fn $testn() {
            let mut t = $init;
            t.insert(0, 1);
            t.insert(1, 2);
            t.insert(2, 3);
            t.insert(4, 5);

            assert_eq!(t.get_single(0), 1);
            assert_eq!(t.get_single(1), 2);
            assert_eq!(t.get_single(2), 3);
            assert_eq!(t.get_single(4), 5);

            assert_eq!(t.get_cuml(0), 1);
            assert_eq!(t.get_cuml(1), 3);
            assert_eq!(t.get_cuml(2), 6);
            assert_eq!(t.get_cuml(4), 11);

            assert_eq!(t.get_quantile(1), Some(0));
            assert_eq!(t.get_quantile(2), Some(1));
            assert_eq!(t.get_quantile(3), Some(1));
            assert_eq!(t.get_quantile(4), Some(2));
            assert_eq!(t.get_quantile(6), Some(2));
            assert_eq!(t.get_quantile(10), Some(4));
            assert_eq!(t.get_quantile(11), Some(4));
            assert_eq!(t.get_quantile(12), None);
        }
    };
}

test_trivial!(ftf_trivial, FenwickTree::with_capacity(5));
test_trivial!(fte_trivial, ExtensibleFenwickTree::new());
test_trivial!(rbt_trivial, CumlTree::new());
test_trivial!(aav_trivial, AAVLTree::new());

macro_rules! test_small_neg_mono {
    ($testn:ident, $init:expr) => {
        #[test]
        fn $testn() {
            let mut t = $init;
            t.insert(0, -3);
            t.insert(1, -1);
            t.insert(2, 3);
            t.insert(3, 1);

            assert_eq!(t.get_single(0), -3);
            assert_eq!(t.get_single(1), -1);
            assert_eq!(t.get_single(2), 3);
            assert_eq!(t.get_single(3), 1);

            assert_eq!(t.get_cuml(0), -3);
            assert_eq!(t.get_cuml(1), -4);
            assert_eq!(t.get_cuml(2), -1);
            assert_eq!(t.get_cuml(3), 0);

            /* quantiles are weird with negative sizes...
            assert_eq!(t.get_quantile(-2), Some(0));
            assert_eq!(t.get_quantile(-4), Some(1));
            */
        }
    };
}

test_small_neg_mono!(ftf_small_neg_mono, FenwickTree::with_capacity(5));
test_small_neg_mono!(fte_small_neg_mono, ExtensibleFenwickTree::new());
test_small_neg_mono!(rbt_small_neg_mono, CumlTree::new());
test_small_neg_mono!(aav_small_neg_mono, AAVLTree::new());

fn load_updates(fname: &str) -> (usize, Vec<i64>, Vec<i64>) {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let fp = File::open(fname).expect("Could not open file");
    let mut reader = BufReader::new(fp);
    let mut line = String::new();

    let _ = reader.read_line(&mut line);
    let cap = line.trim().parse::<usize>().expect("Bad parse");
    let mut keys = Vec::new();
    let mut vals = Vec::new();

    for line in reader.lines() {
        let kv: Vec<i64> = line
            .expect("Could not read line")
            .split(" ")
            .map(|x| x.trim().parse::<i64>().expect("Bad parse"))
            .collect();
        keys.push(kv[0]);
        vals.push(kv[1]);
    }

    (cap, keys, vals)
}

macro_rules! bench_build {
    ($testn:ident, $k:ty, $fname:expr, $init:expr) => {
        #[bench]
        fn $testn(b: &mut Bencher) {
            let (_cap, keys, vals) = load_updates($fname);
            b.iter(|| {
                let mut cm = $init;
                for i in 0..keys.len() {
                    cm.insert(keys[i] as $k, vals[i]);
                }
            });

            let mut cm = $init;
            let mut c50 = 0;
            let mut c1 = 0;
            for i in 0..keys.len() {
                cm.insert(keys[i] as $k, vals[i]);
                if keys[i] <= 50 {
                    c50 += vals[i]
                }
                if keys[i] <= 1 {
                    c1 += vals[i]
                }
            }
            assert_eq!(cm.get_cuml(50), c50);
            assert_eq!(cm.get_cuml(1), c1);
        }
    };
}

bench_build!(ftf_build_1, usize, "src/bench_1", FenwickTree::with_capacity(1000));
bench_build!(fte_build_1, i64,   "src/bench_1", ExtensibleFenwickTree::new());
bench_build!(ftc_build_1, i64,   "src/bench_1", ExtensibleFenwickTree::with_capacity(1000));
bench_build!(rbt_build_1, i64,   "src/bench_1", CumlTree::new());
bench_build!(aav_build_1, i64,   "src/bench_1", AAVLTree::new());

macro_rules! bench_getc {
    ($testn:ident, $k:ty, $fname:expr, $init:expr) => {
        #[bench]
        fn $testn(b: &mut Bencher) {
            let (_cap, keys, vals) = load_updates($fname);
            let mut cm = $init;
            for i in 0..keys.len() {
                cm.insert(keys[i] as $k, vals[i]);
            }

            let mx = *keys.iter().max().unwrap();
            let mut c = 0;
            b.iter(|| {
                for i in 0..mx {
                    let j = test::black_box(i);
                    c = cm.get_cuml(j as $k);
                }
            });
        }
    };
}

bench_getc!(ftf_getc_1, usize, "src/bench_1", FenwickTree::with_capacity(1000));
bench_getc!(fte_getc_1, i64,   "src/bench_1", ExtensibleFenwickTree::new());
bench_getc!(ftc_getc_1, i64,   "src/bench_1", ExtensibleFenwickTree::with_capacity(1000));
bench_getc!(rbt_getc_1, i64,   "src/bench_1", CumlTree::new());
bench_getc!(aav_getc_1, i64,   "src/bench_1", AAVLTree::new());

macro_rules! bench_degen {
    ($testn:ident, $k:ty, $init:expr) => {
        #[bench]
        fn $testn(b: &mut Bencher)
        {
            let n = 1000;
            b.iter(|| {
                let mut cm = $init;
                for i in 1..n {
                    cm.insert(i as $k, i);
                }
            });

            let mut cm = $init;
            let mut tot = 0;
            for i in 1..n {
                cm.insert(i as $k, i);
                tot += i;
                assert_eq!(cm.get_cuml(i), tot);
                assert_eq!(cm.get_single(i), i);
            }
        }
    };
}

bench_degen!(ftf_build_degen, usize, FenwickTree::with_capacity(1000));
bench_degen!(fte_build_degen, i64,   ExtensibleFenwickTree::new());
bench_degen!(ftc_build_degen, i64,   ExtensibleFenwickTree::with_capacity(1000));
bench_degen!(rbt_build_degen, i64,   CumlTree::new());
bench_degen!(aav_build_degen, i64,   AAVLTree::new());

macro_rules! bench_getc_degen {
    ($testn:ident, $k:ty, $init:expr) => {
        #[bench]
        fn $testn(b: &mut Bencher) {
            let n = 1000;
            let mut cm = $init;
            for i in 1..n {
                cm.insert(i as $k, i);
            }

            let mut c = 0;
            b.iter(|| {
                for i in 0..n {
                    let j = test::black_box(i);
                    c = cm.get_cuml(j as $k);
                }
            });
        }
    };
}

bench_getc_degen!(ftf_getc_degen, usize, FenwickTree::with_capacity(1000));
bench_getc_degen!(fte_getc_degen, i64,   ExtensibleFenwickTree::new());
bench_getc_degen!(ftc_getc_degen, i64,   ExtensibleFenwickTree::with_capacity(1000));
bench_getc_degen!(rbt_getc_degen, i64,   CumlTree::new());
bench_getc_degen!(aav_getc_degen, i64,   AAVLTree::new());

macro_rules! test_neg_key {
    ($testn:ident, $init:expr) => {
        #[test]
        fn $testn() {
            let mut t = $init;
            t.insert(-2, 1);
            t.insert(1, 2);
            t.insert(-1, 3);
            t.insert(3, 5);

            assert_eq!(t.get_single(-2), 1);
            assert_eq!(t.get_single(-1), 3);
            assert_eq!(t.get_single(0), 0);
            assert_eq!(t.get_single(1), 2);
            assert_eq!(t.get_single(3), 5);

            assert_eq!(t.get_cuml(-2), 1);
            assert_eq!(t.get_cuml(-1), 4);
            assert_eq!(t.get_cuml(0), 4);
            assert_eq!(t.get_cuml(1), 6);
            assert_eq!(t.get_cuml(3), 11);

            assert_eq!(t.get_quantile(1), Some(-2));
            assert_eq!(t.get_quantile(2), Some(-1));
            assert_eq!(t.get_quantile(3), Some(-1));
            assert_eq!(t.get_quantile(4), Some(-1));
            assert_eq!(t.get_quantile(6), Some(1));
            assert_eq!(t.get_quantile(10), Some(3));
            assert_eq!(t.get_quantile(11), Some(3));
            assert_eq!(t.get_quantile(12), None);
        }
    };
}

test_neg_key!(fte_neg_key, ExtensibleFenwickTree::new());
test_neg_key!(rbt_neg_key, CumlTree::new());
test_neg_key!(aav_neg_key, AAVLTree::new());

// ExtensibleFenwickTree specific tests

#[test]
fn fte_oob_1() {
    let mut t = ExtensibleFenwickTree::with_capacity(10);
    t.insert(5, 5);
    assert_eq!(t.get_cuml(-10), 0);
    assert_eq!(t.get_single(-10), 0);
    assert_eq!(t.get_cuml(10), 5);
    assert_eq!(t.get_single(10), 0);
}

macro_rules! test_oob_query {
    ($testn:ident, $init:expr) => {
        #[test]
        fn $testn() {
            let mut t = $init;
            t.insert(3,2);

            assert_eq!(t.get_cuml(15), 2);
            assert_eq!(t.get_single(15), 0);

            t.insert(9, 5);

            // seven being the sum of the only two elements actually in the structure
            assert_eq!(t.get_cuml(15), 7);
            assert_eq!(t.get_single(15), 0);
        }
    }
}

test_oob_query!(fwt_oob_query, FenwickTree::with_capacity(10));
test_oob_query!(eft_oob_query, ExtensibleFenwickTree::with_capacity(10));
