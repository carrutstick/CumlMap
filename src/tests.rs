extern crate test;
use self::test::Bencher;
use super::*;

macro_rules! test_trivial {
    ($testn:ident, $type:expr) => {
        #[test]
        fn $testn() {
            let mut t = $type;
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

test_trivial!(bix_trivial, BinaryIndexTree::with_capacity(5));
test_trivial!(eix_trivial, ExtensibleBinaryIndexTree::new());
test_trivial!(dct_trivial, BoxedCumlTree::new());
test_trivial!(act_trivial, ArenaCumlTree::new());
test_trivial!(aat_trivial, AACumlTree::new());
test_trivial!(art_trivial, AARCumlTree::new());
test_trivial!(rbt_trivial, RBCumlTree::new());
test_trivial!(rct_trivial, RCCumlTree::new());

macro_rules! test_small_neg_mono {
    ($testn:ident, $type:expr) => {
        #[test]
        fn $testn() {
            let mut t = $type;
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

test_small_neg_mono!(bix_small_neg_mono, BinaryIndexTree::with_capacity(5));
test_small_neg_mono!(eix_small_neg_mono, ExtensibleBinaryIndexTree::new());
test_small_neg_mono!(dct_small_neg_mono, BoxedCumlTree::new());
test_small_neg_mono!(act_small_neg_mono, ArenaCumlTree::new());
test_small_neg_mono!(aat_small_neg_mono, AACumlTree::new());
test_small_neg_mono!(art_small_neg_mono, AARCumlTree::new());
test_small_neg_mono!(rbt_small_neg_mono, RBCumlTree::new());
test_small_neg_mono!(rct_small_neg_mono, RCCumlTree::new());

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

bench_build!(bix_build_1, usize, "src/bench_1", BinaryIndexTree::with_capacity(1000));
bench_build!(eix_build_1, i64,   "src/bench_1", ExtensibleBinaryIndexTree::new());
bench_build!(cix_build_1, i64,   "src/bench_1", ExtensibleBinaryIndexTree::with_capacity(1000));
bench_build!(dct_build_1, i64,   "src/bench_1", BoxedCumlTree::new());
bench_build!(act_build_1, i64,   "src/bench_1", ArenaCumlTree::new());
bench_build!(aat_build_1, i64,   "src/bench_1", AACumlTree::new());
bench_build!(art_build_1, i64,   "src/bench_1", AARCumlTree::new());
bench_build!(rbt_build_1, i64,   "src/bench_1", RBCumlTree::new());
bench_build!(rct_build_1, i64,   "src/bench_1", RCCumlTree::new());

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

bench_getc!(bix_getc_1, usize, "src/bench_1", BinaryIndexTree::with_capacity(1000));
bench_getc!(eix_getc_1, i64,   "src/bench_1", ExtensibleBinaryIndexTree::new());
bench_getc!(cix_getc_1, i64,   "src/bench_1", ExtensibleBinaryIndexTree::with_capacity(1000));
bench_getc!(dct_getc_1, i64,   "src/bench_1", BoxedCumlTree::new());
bench_getc!(act_getc_1, i64,   "src/bench_1", ArenaCumlTree::new());
bench_getc!(aat_getc_1, i64,   "src/bench_1", AACumlTree::new());
bench_getc!(art_getc_1, i64,   "src/bench_1", AARCumlTree::new());
bench_getc!(rbt_getc_1, i64,   "src/bench_1", RBCumlTree::new());
bench_getc!(rct_getc_1, i64,   "src/bench_1", RCCumlTree::new());

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

bench_degen!(bix_build_degen, usize, BinaryIndexTree::with_capacity(1000));
bench_degen!(eix_build_degen, i64,   ExtensibleBinaryIndexTree::new());
bench_degen!(cix_build_degen, i64,   ExtensibleBinaryIndexTree::with_capacity(1000));
bench_degen!(dct_build_degen, i64,   BoxedCumlTree::new());
bench_degen!(act_build_degen, i64,   ArenaCumlTree::new());
bench_degen!(aat_build_degen, i64,   AACumlTree::new());
bench_degen!(art_build_degen, i64,   AARCumlTree::new());
bench_degen!(rbt_build_degen, i64,   RBCumlTree::new());
bench_degen!(rct_build_degen, i64,   RCCumlTree::new());

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

bench_getc_degen!(bix_getc_degen, usize, BinaryIndexTree::with_capacity(1000));
bench_getc_degen!(eix_getc_degen, i64,   ExtensibleBinaryIndexTree::new());
bench_getc_degen!(cix_getc_degen, i64,   ExtensibleBinaryIndexTree::with_capacity(1000));
bench_getc_degen!(dct_getc_degen, i64,   BoxedCumlTree::new());
bench_getc_degen!(act_getc_degen, i64,   ArenaCumlTree::new());
bench_getc_degen!(aat_getc_degen, i64,   AACumlTree::new());
bench_getc_degen!(art_getc_degen, i64,   AARCumlTree::new());
bench_getc_degen!(rbt_getc_degen, i64,   RBCumlTree::new());
bench_getc_degen!(rct_getc_degen, i64,   RCCumlTree::new());

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

test_neg_key!(eix_neg_key, ExtensibleBinaryIndexTree::new());
test_neg_key!(dct_neg_key, BoxedCumlTree::new());
test_neg_key!(act_neg_key, ArenaCumlTree::new());
test_neg_key!(aat_neg_key, AACumlTree::new());
test_neg_key!(art_neg_key, AARCumlTree::new());
test_neg_key!(rbt_neg_key, RBCumlTree::new());
test_neg_key!(rct_neg_key, RCCumlTree::new());
