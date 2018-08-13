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

/*
fn load_updates(fname: &str) -> (usize, Vec<usize>, Vec<i32>) {
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
        let kv: Vec<i32> = line
            .expect("Could not read line")
            .split(" ")
            .map(|x| x.trim().parse::<i32>().expect("Bad parse"))
            .collect();
        keys.push(kv[0] as usize);
        vals.push(kv[1]);
    }

    (cap, keys, vals)
}

fn benchmark_build<T>(fname: &str, b: &mut Bencher)
where
    T: CumlMap<Key = usize, Value = i32>,
{
    let (cap, keys, vals) = load_updates(fname);
    b.iter(|| {
        let mut cm = T::with_capacity(cap);
        for i in 0..keys.len() {
            cm.insert(keys[i], vals[i]);
        }
    });

    let mut cm = T::with_capacity(cap);
    let mut c50 = 0;
    let mut c1 = 0;
    for i in 0..keys.len() {
        cm.insert(keys[i], vals[i]);
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

impl_bench_file!(benchmark_build, bix_1_build, "src/bench_1", BinaryIndexTree<i32>);
impl_bench_file!(benchmark_build, dct_1_build, "src/bench_1", BoxedCumlTree<usize, i32>);
impl_bench_file!(benchmark_build, act_1_build, "src/bench_1", ArenaCumlTree<usize, i32>);
impl_bench_file!(benchmark_build, aat_1_build, "src/bench_1", AACumlTree<usize, i32>);
impl_bench_file!(benchmark_build, art_1_build, "src/bench_1", AARCumlTree<usize, i32>);
impl_bench_file!(benchmark_build, rbt_1_build, "src/bench_1", RBCumlTree<usize, i32>);

fn benchmark_get_cuml<T>(fname: &str, b: &mut Bencher)
where
    T: CumlMap<Key = usize, Value = i32>,
{
    let (cap, keys, vals) = load_updates(fname);
    let mut cm = T::with_capacity(cap);
    for i in 0..keys.len() {
        cm.insert(keys[i], vals[i]);
    }

    let mx = *keys.iter().max().unwrap();
    let mut c = 0;
    b.iter(|| {
        for i in 0..mx {
            let j = test::black_box(i);
            c = cm.get_cuml(j);
        }
    });
}

impl_bench_file!(benchmark_get_cuml, bix_1_getc, "src/bench_1", BinaryIndexTree<i32>);
impl_bench_file!(benchmark_get_cuml, dct_1_getc, "src/bench_1", BoxedCumlTree<usize, i32>);
impl_bench_file!(benchmark_get_cuml, act_1_getc, "src/bench_1", ArenaCumlTree<usize, i32>);
impl_bench_file!(benchmark_get_cuml, aat_1_getc, "src/bench_1", AACumlTree<usize, i32>);
impl_bench_file!(benchmark_get_cuml, art_1_getc, "src/bench_1", AARCumlTree<usize, i32>);
impl_bench_file!(benchmark_get_cuml, rbt_1_getc, "src/bench_1", RBCumlTree<usize, i32>);

fn benchmark_degen<T>(b: &mut Bencher)
where
    T: CumlMap<Key = usize, Value = i32>,
{
    let n = 1000;
    b.iter(|| {
        let mut cm = T::with_capacity(n);
        for i in 1..n {
            cm.insert(i, i as i32);
        }
    });

    let mut cm = T::with_capacity(n);
    let mut tot = 0;
    for i in 1..n {
        cm.insert(i, i as i32);
        tot += i;
        assert_eq!(cm.get_cuml(i), tot as i32);
        assert_eq!(cm.get_single(i), i as i32);
    }
}

impl_bench!(benchmark_degen, bix_build_degen, BinaryIndexTree<i32>);
impl_bench!(benchmark_degen, dct_build_degen, BoxedCumlTree<usize, i32>);
impl_bench!(benchmark_degen, act_build_degen, ArenaCumlTree<usize, i32>);
impl_bench!(benchmark_degen, aat_build_degen, AACumlTree<usize, i32>);
impl_bench!(benchmark_degen, art_build_degen, AARCumlTree<usize, i32>);
impl_bench!(benchmark_degen, rbt_build_degen, RBCumlTree<usize, i32>);

fn benchmark_degen_get_cuml<T>(b: &mut Bencher)
where
    T: CumlMap<Key = usize, Value = i32>,
{
    let n = 1000;
    let mut cm = T::with_capacity(n);
    for i in 1..n {
        cm.insert(i, i as i32);
    }

    let mut c = 0;
    b.iter(|| {
        for i in 0..n {
            let j = test::black_box(i);
            c = cm.get_cuml(j);
        }
    });
}

impl_bench!(benchmark_degen_get_cuml, bix_getc_degen, BinaryIndexTree<i32>);
impl_bench!(benchmark_degen_get_cuml, dct_getc_degen, BoxedCumlTree<usize, i32>);
impl_bench!(benchmark_degen_get_cuml, act_getc_degen, ArenaCumlTree<usize, i32>);
impl_bench!(benchmark_degen_get_cuml, aat_getc_degen, AACumlTree<usize, i32>);
impl_bench!(benchmark_degen_get_cuml, art_getc_degen, AARCumlTree<usize, i32>);
impl_bench!(benchmark_degen_get_cuml, rbt_getc_degen, RBCumlTree<usize, i32>);

fn test_neg_key<T>()
where
    T: CumlMap<Key = i64, Value = i32>,
{
    let mut t = T::with_capacity(4);
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

impl_test!(test_neg_key, eix_neg_key, ExtensibleBinaryIndexTree<i32>);
impl_test!(test_neg_key, dct_neg_key, BoxedCumlTree<i64, i32>);
impl_test!(test_neg_key, act_neg_key, ArenaCumlTree<i64, i32>);
impl_test!(test_neg_key, aat_neg_key, AACumlTree<i64, i32>);
impl_test!(test_neg_key, art_neg_key, AARCumlTree<i64, i32>);
impl_test!(test_neg_key, rbt_neg_key, RBCumlTree<i64, i32>);
*/
