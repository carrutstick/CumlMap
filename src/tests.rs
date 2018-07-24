extern crate test;
use self::test::Bencher;
use super::*;

macro_rules! impl_test {
    ($basen:ident, $testn:ident, $type:ty) => {
        #[test]
        fn $testn() {
            $basen::<$type>();
        }
    };
}

macro_rules! impl_bench_file {
    ($basen:ident, $testn:ident, $file:expr, $type:ty) => {
        #[bench]
        fn $testn(b: &mut Bencher) {
            $basen::<$type>($file, b)
        }
    };
}

macro_rules! impl_bench {
    ($basen:ident, $testn:ident, $type:ty) => {
        #[bench]
        fn $testn(b: &mut Bencher) {
            $basen::<$type>(b)
        }
    };
}

fn test_trivial<T>()
where
    T: CumlMap<Key = usize, Value = i32>,
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
    assert_eq!(t.get_quantile(11), Some(3));
    assert_eq!(t.get_quantile(12), None);
}

impl_test!(test_trivial, cft_trivial, CumlFreqTable<i32>);
impl_test!(test_trivial, bix_trivial, BinaryIndexTree<i32>);
impl_test!(test_trivial, dct_trivial, BoxedCumlTree<usize, i32>);
impl_test!(test_trivial, act_trivial, ArenaCumlTree<usize, i32>);
impl_test!(test_trivial, aat_trivial, AACumlTree<usize, i32>);
impl_test!(test_trivial, art_trivial, AARCumlTree<usize, i32>);

fn test_small_neg_mono<T>()
where
    T: CumlMap<Key = usize, Value = i32>,
{
    let mut t = T::with_capacity(4);
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

impl_test!(test_small_neg_mono, cft_small_neg_mono, CumlFreqTable<i32>);
impl_test!(test_small_neg_mono, bix_small_neg_mono, BinaryIndexTree<i32>);
impl_test!(test_small_neg_mono, dct_small_neg_mono, BoxedCumlTree<usize, i32>);
impl_test!(test_small_neg_mono, act_small_neg_mono, ArenaCumlTree<usize, i32>);
impl_test!(test_small_neg_mono, aat_small_neg_mono, AACumlTree<usize, i32>);
impl_test!(test_small_neg_mono, art_small_neg_mono, AARCumlTree<usize, i32>);

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

impl_bench_file!(benchmark_build, cft_build_1, "src/bench_1", CumlFreqTable<i32>);
impl_bench_file!(benchmark_build, bix_build_1, "src/bench_1", BinaryIndexTree<i32>);
impl_bench_file!(benchmark_build, dct_build_1, "src/bench_1", BoxedCumlTree<usize, i32>);
impl_bench_file!(benchmark_build, act_build_1, "src/bench_1", ArenaCumlTree<usize, i32>);
impl_bench_file!(benchmark_build, aat_build_1, "src/bench_1", AACumlTree<usize, i32>);
impl_bench_file!(benchmark_build, art_build_1, "src/bench_1", AARCumlTree<usize, i32>);

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

impl_bench!(benchmark_degen, cft_build_degen, CumlFreqTable<i32>);
impl_bench!(benchmark_degen, bix_build_degen, BinaryIndexTree<i32>);
impl_bench!(benchmark_degen, dct_build_degen, BoxedCumlTree<usize, i32>);
impl_bench!(benchmark_degen, act_build_degen, ArenaCumlTree<usize, i32>);
impl_bench!(benchmark_degen, aat_build_degen, AACumlTree<usize, i32>);
impl_bench!(benchmark_degen, art_build_degen, AARCumlTree<usize, i32>);
