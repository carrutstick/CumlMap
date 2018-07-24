extern crate test;
use self::test::Bencher;
use super::*;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
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
    test_trivial::<BoxedCumlTree<usize, i32>>();
}

#[test]
fn trivial_act() {
    test_trivial::<ArenaCumlTree<usize, i32>>();
}

#[test]
fn trivial_aat() {
    test_trivial::<AACumlTree<usize, i32>>();
}

#[test]
fn trivial_art() {
    test_trivial::<AARCumlTree<usize, i32>>();
}

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

fn benchmark_from_file<T>(fname: &str, b: &mut Bencher)
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
    benchmark_from_file::<BoxedCumlTree<usize, i32>>("src/bench_1", b);
}

#[bench]
fn act_bench_1_build(b: &mut Bencher) {
    benchmark_from_file::<ArenaCumlTree<usize, i32>>("src/bench_1", b);
}

#[bench]
fn aat_bench_1_build(b: &mut Bencher) {
    benchmark_from_file::<AACumlTree<usize, i32>>("src/bench_1", b);
}

#[bench]
fn art_bench_1_build(b: &mut Bencher) {
    benchmark_from_file::<AARCumlTree<usize, i32>>("src/bench_1", b);
}

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

#[bench]
fn cft_bench_degen_build(b: &mut Bencher) {
    benchmark_degen::<CumlFreqTable<i32>>(b);
}

#[bench]
fn bix_bench_degen_build(b: &mut Bencher) {
    benchmark_degen::<BinaryIndexTree<i32>>(b);
}

#[bench]
fn dct_bench_degen_build(b: &mut Bencher) {
    benchmark_degen::<BoxedCumlTree<usize, i32>>(b);
}

#[bench]
fn act_bench_degen_build(b: &mut Bencher) {
    benchmark_degen::<ArenaCumlTree<usize, i32>>(b);
}

#[bench]
fn aat_bench_degen_build(b: &mut Bencher) {
    benchmark_degen::<AACumlTree<usize, i32>>(b);
}

#[bench]
fn art_bench_degen_build(b: &mut Bencher) {
    benchmark_degen::<AARCumlTree<usize, i32>>(b);
}
