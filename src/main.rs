// newsort benchmark vs standard sort

// (C) 2015 Michael Howell <michael@notriddle.com>
// This file is licensed under the same terms as Rust itself.

// Updated by Bill Wood Sept 2021
//     * test that new sort result is identical to standard sort (including stability)
//     * randomize array sizes within decade (eg decade 100 has random sizes between 0 and 999)
//     * option for multiple benchmark runs
//     * option for quick eq testing with multiple array sizes
//     * track relative comparison counts between newsort and standard sort
//     * determine number of iterations using timer instead of fixed count
//     * add "reverse sorted" variant
//     * add parallel sort option
//     * call stdsort from a module or from stdlib

mod newsort;
mod stdsort;
mod par_newsort;
use std::cmp::Ordering;

#[derive(Debug,Clone,Copy,Eq)]
struct Test<T: Ord>(T, u32);
impl<T: Ord> Ord for Test<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T: Ord> PartialOrd for Test<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord> PartialEq for Test<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn test_sort_eq<T>(alg_std: Algorithm, alg_ns: Algorithm, v: &[T]) -> (u64, u64, bool)
where
    T: Ord + Clone + Copy + std::fmt::Debug + Send + Sync,
{
    let mut v_ns: Vec<Test<T>> = v.iter().enumerate().map(|(i, &x)| Test(x, i as u32)).collect();
    let mut v_std = v_ns.clone();
    let (mut cnt_std, mut cnt_ns) = (0, 0);
    match alg_std {
        Algorithm::Std => stdsort::sort_by(&mut v_std, |a, b| { cnt_std += 1; a.cmp(b) }),
        Algorithm::LibStd => v_std.sort_by(|a, b| { cnt_std += 1; a.cmp(b) }),
        Algorithm::Newsort => newsort::sort_by(&mut v_std, |a, b| { cnt_std += 1; a.cmp(b) }),
        Algorithm::ParStd => v_std.par_sort_by(|a, b| a.cmp(b)),
        Algorithm::ParNewsort => par_newsort::par_sort_by(&mut v_std, |a, b| a.cmp(b)),
    }
    match alg_ns {
        Algorithm::Std => stdsort::sort_by(&mut v_ns, |a, b| { cnt_ns += 1; a.cmp(b) }),
        Algorithm::LibStd => v_ns.sort_by(|a, b| { cnt_ns += 1; a.cmp(b) }),
        Algorithm::Newsort => newsort::sort_by(&mut v_ns, |a, b| { cnt_ns += 1; a.cmp(b) }),
        Algorithm::ParStd => v_ns.par_sort_by(|a, b| a.cmp(b)),
        Algorithm::ParNewsort => par_newsort::par_sort_by(&mut v_ns, |a, b| a.cmp(b)),
    }
    let mut failed = false;
    v_ns.iter().zip(v_std.iter()).enumerate()
        .for_each(|(i, (x, y))|
            if ! failed {
                if x.0 != y.0 {
                    failed = true;
                    println!("v[{}] {:?} != {:?}", i, x, y);
                } else if x.1 != y.1 {
                    failed = true;
                    println!("v[{}] {:?} swap {:?}", i, x, y);
                }
                if failed {
                    println!("{:?}", v_ns.iter().map(|x| x.0).collect::<Vec<_>>());
                    println!("{:?}", v_std.iter().map(|x| x.0).collect::<Vec<_>>());
                }
            }
        );
    (cnt_std, cnt_ns, failed)
}

use std::cmp::min;
use std::fmt::{self, Display, Formatter};

#[derive(Copy,Clone,Debug)]
enum Algorithm {
    Std,
    LibStd,
    Newsort,
    ParStd,
    ParNewsort,
}

#[derive(Copy,Clone)]
enum Pattern {
    Sawtooth,
    Rand,
    Stagger,
    Plateau,
    Shuffle,
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Pattern::Sawtooth => "sawtooth",
            Pattern::Rand => "rand",
            Pattern::Stagger => "stagger",
            Pattern::Plateau => "plateau",
            Pattern::Shuffle => "shuffle",
        }.fmt(f)
    }
}

#[derive(Copy,Clone)]
enum Variant {
    Ident,
    Reverse,
    ReverseFront,
    ReverseBack,
    Sorted,
    ReverseSorted,
    Dither,
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Variant::Ident => "ident",
            Variant::Reverse => "reverse",
            Variant::ReverseFront => "reverse_front",
            Variant::ReverseBack => "reverse_back",
            Variant::Sorted => "sorted",
            Variant::ReverseSorted => "reverse_sorted",
            Variant::Dither => "dither",
        }.fmt(f)
    }
}

fn generate_int(pattern: Pattern, variant: Variant, size: usize, rand_size: bool, m: usize, test_type: TestType, run: usize) -> Vec<i128> {
    use rand::prelude::*;
    let mut rng =
        if let TestType::BenchMark = test_type {
            StdRng::seed_from_u64((m*size + run) as u64)
        } else {
            StdRng::from_entropy()
        };
    let rsize = (rng.gen::<f32>()*10.0*size as f32) as usize;
    let size = if rand_size { rsize } else { size };
    let mut ret_val = Vec::with_capacity(size);
    let (mut j, mut k) = (0, 0);
    for i in 0 .. size {
        ret_val.push(match pattern {
            Pattern::Sawtooth => i % m,
            Pattern::Rand => rng.gen::<usize>(),
            Pattern::Stagger => (i*m + i) % size,
            Pattern::Plateau => min(i, m),
            Pattern::Shuffle => if rng.gen::<usize>() % m == 0 { j+=2; j } else { k += 2; k },
        } as i128);
    }
    match variant {
        Variant::Ident => (),
        Variant::Reverse => ret_val.reverse(),
        Variant::ReverseFront => ret_val[0 .. size / 2].reverse(),
        Variant::ReverseBack => ret_val[size / 2 .. ].reverse(),
        Variant::Sorted => ret_val.sort(),
        Variant::ReverseSorted => { ret_val.sort(); ret_val.reverse() },
        Variant::Dither => for x in &mut ret_val { *x %= 5; },
    }
    ret_val
}

fn run_test<T>(algorithm: Algorithm, v_once: &[T]) -> (f64, usize)
where
    T: Ord + Copy + Send + Sync,
{
    const MAX_TIME_MS: u128 = 125;
    let len = v_once.len();
    let mut v = v_once.to_vec();
    let mut trial_count = 0;
    let start_total = std::time::Instant::now();

    loop {
        match algorithm {
            Algorithm::Std => stdsort::sort(&mut v),
            Algorithm::LibStd => v.sort(),
            Algorithm::Newsort => newsort::sort(&mut v),
            Algorithm::ParStd => v.par_sort(),
            Algorithm::ParNewsort => par_newsort::par_sort(&mut v),
        }
        trial_count += 1;
        if start_total.elapsed().as_millis() >= MAX_TIME_MS {
            break;
        }
        v.copy_from_slice(v_once);
    }
    let mut elapsed = start_total.elapsed().as_secs_f64();
    (0..trial_count - 1).for_each(|_| v.copy_from_slice(v_once));
    elapsed -= start_total.elapsed().as_secs_f64() - elapsed;
    ((len*trial_count) as f64/1_000_000_f64/elapsed, trial_count)
}

#[derive(Copy, Clone)]
enum TestType {
    BenchMark,
    EqualityTest,
}

struct Arg<T>{
    default: T,
    value: Option<T>,
}
impl<T: Copy> Arg<T>{
    fn new(default: T) -> Arg<T> {
        Self { default: default, value: None }
    }
    fn get(&self) -> T {
        if let Some(val) = self.value {
            val
        } else {
            self.default
        }
    }
    fn set(&mut self, value: T) {
        self.value = Some(value);
    }
    fn set_default(&mut self, default: T) {
        self.default = default;
    }
    // fn get_default(&self) -> T {
    //     self.default
    // }
}

use rayon::prelude::*;
fn main() {
    let mut test_type = Ok(TestType::BenchMark);
    let (mut verbose, mut n_runs) = (false, Arg::new(1));
    let (mut alg_std, mut alg_ns) = (Arg::new(Algorithm::Std), Arg::new(Algorithm::Newsort));
    let (mut min, mut max) = (Arg::new(1), Arg::new(6));
    let mut rand_sizes = true;
    let mut iter = std::env::args().skip(1);

    while let Some(arg) = iter.next() {
        match &arg[..] {
            "-v" | "--verbose" =>
                verbose = true,
            "benchmark" => {
                test_type = Ok(TestType::BenchMark);
                max.set_default(6);
                },
            "eq" => {
                test_type = Ok(TestType::EqualityTest);
                max.set_default(4);
                n_runs.set_default(20);
                },
            "-n" | "--nruns" =>
                if let Some(arg2) = iter.next() {
                    if let Ok(n) = arg2.parse::<usize>() {
                        n_runs.set(if n == 0 { <usize>::MAX } else { n });
                    } else {
                        test_type = Err("invalid number");
                        break;
                    }
                } else {
                    test_type = Err("number not provided");
                    break;
                },
            "-p" | "--parallel" => {
                alg_std.set_default(Algorithm::ParStd);
                alg_ns.set_default(Algorithm::ParNewsort);
                },
            "--std" | "--new" =>
                if let Some(arg2) = iter.next() {
                    let alg = if arg == "--std" { &mut alg_std } else { &mut alg_ns };
                    match &arg2[..] {
                        "std" => alg.set(Algorithm::Std),
                        "lib-std" => alg.set(Algorithm::LibStd),
                        "new" => alg.set(Algorithm::Newsort),
                        "par-std" => alg.set(Algorithm::ParStd),
                        "par-new" => alg.set(Algorithm::ParNewsort),
                        _ => { test_type = Err("invalid algorithm"); break },
                    }
                } else {
                    test_type = Err("algorithm not provided");
                },
            "--max" | "--min" =>
                if let Some(arg2) = iter.next() {
                    if let Ok(n) = arg2.parse::<u32>() {
                        let val = if arg == "--max" { &mut max } else { &mut min };
                        val.set(n);
                    } else {
                        test_type = Err("invalid number");
                        break;
                    }
                } else {
                    test_type = Err("number not provided");
                },
            "--no-rand-sizes" =>
                rand_sizes = false,
            _ => {
                test_type = Err("unknown option");
                break;
                }
        }
    }

    if let Err(s) = test_type {
        eprintln!("error: {}", s);
        eprintln!("usage: newsort [ benchmark ] [ eq ] [ -n n | --nruns n ] [ -v | --verbose ]");
        eprintln!("               [ --max n ] [ --min n ] [ -p | --parallel ] [ --no-rand-sizes ]");
        eprintln!("               [ --std std | new | par-std | par-new ] [ --new std | new | par-std | par-new ]");
        std::process::exit(1);
    }

    let test_type = test_type.unwrap();
    let n_runs = n_runs.get();
    let (alg_std, alg_ns) = (alg_std.get(), alg_ns.get());
    let (mut cmp_count_ns_total, mut cmp_count_std_total) = (0, 0);

    let strings = {
        use std::io::Read;
        let path = std::path::Path::new("./strings.txt");
        let mut file = std::fs::File::open(&path).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        s
    };
    let strings = strings.split_whitespace().collect::<Vec::<_>>();

    eprintln!("Std: {:?}, Newsort: {:?}", alg_std, alg_ns);
    match (alg_std, alg_ns) {
        (Algorithm::ParStd, _) | (_, Algorithm::ParNewsort) | (Algorithm::ParNewsort, _) | (_, Algorithm::ParStd) =>
            eprintln!("Running in parallel with {} cpus", rayon::current_num_threads()),
        _ => (),
    }
    eprintln!("Range {} to {}", 10_usize.pow(min.get()), 10_usize.pow(max.get()));
    eprintln!("String array size = {}", strings.len());
    if let TestType::BenchMark = test_type {
        println!("{: >7} / {: >7} {: >7}{: >15}{: >15}{: >12}{: >12}{: >12}{: >12}{: >12}{: >12}{: >12}",
            "decade", "size", "m", "pattern", "variant", "new-runs", "std-runs", "new-cmp-cnt", "std-cmp-cnt", "new-M/sec", "std-M/sec", "new/std");
    } else {
        eprintln!("Running sort equality test with {} runs...", n_runs);
        if verbose {
            println!("{: >7} / {: >7} {: >7}{: >15}{: >15}{: >12}{: >12}{: >12}{: >12}",
                "decade", "size", "m", "pattern", "variant", "new-cmp-cnt", "std-cmp-cnt", "new/std", "tot-new/std");
        }
    }
    for run in 0..n_runs {
        if let TestType::EqualityTest = test_type {
            if run > 0 && run % 20 == 0 {
                eprintln!("{} equality test runs completed", run);
            }
        } else {
            eprint!("Running benchmark {} of {}:", run + 1, n_runs);
        }

        // string sort test
        let mut size = 10;
        while size < strings.len() {
            size *= 10;
        }
        if if let TestType::BenchMark = test_type {
            eprint!(" strings...");
            true
        } else {
            run == 0
        } {
            for &variant in &[Variant::Ident, Variant::Reverse, Variant::ReverseFront, Variant::ReverseBack, Variant::Sorted, Variant::ReverseSorted, ] {
                let mut strings = strings.clone();
                let len = strings.len();
                match variant {
                    Variant::Ident => (),
                    Variant::Reverse => strings.reverse(),
                    Variant::ReverseFront => strings[0 .. len / 2].reverse(),
                    Variant::ReverseBack => strings[len / 2 .. ].reverse(),
                    Variant::Sorted => strings.sort(),
                    Variant::ReverseSorted => { strings.sort(); strings.reverse() },
                    _ => (),
                }
                let (cmp_count_std, cmp_count_ns, failed) = test_sort_eq(alg_std, alg_ns, &strings);
                if failed {
                    println!("{: >7} / {: >7}", size, strings.len());
                    panic!("failed!");
                }
                cmp_count_ns_total += cmp_count_ns;
                cmp_count_std_total += cmp_count_std;
                if let TestType::EqualityTest = test_type {
                    if verbose {
                        println!("{: >7} / {: >7} {: >7}{: >15}{: >15}{: >12.0}{: >12.0}{: >12.4}{: >12.4}",
                            size, strings.len(), 0, "strings", variant,
                            cmp_count_ns, cmp_count_std,
                            (cmp_count_ns as f64)/(cmp_count_std as f64),
                            (cmp_count_ns_total as f64)/(cmp_count_std_total as f64));
                    }
                } else {
                    let (throughput_std, trial_count_std) = run_test(alg_std, &strings);
                    let (throughput_ns, trial_count_ns) = run_test(alg_ns, &strings);
                    println!("{: >7} / {: >7} {: >7}{: >15}{: >15}{: >12.0}{: >12.0}{: >12.0}{: >12.0}{: >12.1}{: >12.1}{: >12.2}",
                        size, strings.len(), 0, "strings", variant,
                        trial_count_ns, trial_count_std,
                        cmp_count_ns, cmp_count_std,
                        throughput_ns, throughput_std,
                        throughput_ns / throughput_std);
                }
            }
        }

        // i128 sort test
        for size_pow in min.get()..=max.get() {
            let size = 10_usize.pow(size_pow);
            if let TestType::BenchMark = test_type {
                eprint!(" {}'s...", size);
            }
            let incr = size/5;
            for m in (0..=size + incr).step_by(incr) {
                if m == 2 { continue; }
                let m = if m == 0 { 2 } else { m };
                for &pattern in &[Pattern::Sawtooth, Pattern::Rand, Pattern::Stagger, Pattern::Plateau, Pattern::Shuffle] {
                    for &variant in &[Variant::Ident, Variant::Reverse, Variant::ReverseFront, Variant::ReverseBack, Variant::Sorted, Variant::ReverseSorted, Variant::Dither] {
                        let v = generate_int(pattern, variant, size, rand_sizes, m, test_type, run);
                        let (cmp_count_std, cmp_count_ns, failed) = test_sort_eq(alg_std, alg_ns, &v);
                        if failed {
                            println!("{: >7} / {: >7} {: >7}{: >15}{: >15}",
                                size, v.len(), m, pattern, variant);
                            panic!("failed!");
                        }
                        cmp_count_ns_total += cmp_count_ns;
                        cmp_count_std_total += cmp_count_std;
                        if let TestType::EqualityTest = test_type {
                            if verbose {
                                println!("{: >7} / {: >7} {: >7}{: >15}{: >15}{: >12.0}{: >12.0}{: >12.4}{: >12.4}",
                                    size, v.len(), m, pattern, variant,
                                    cmp_count_ns, cmp_count_std,
                                    (cmp_count_ns as f64)/(cmp_count_std as f64),
                                    (cmp_count_ns_total as f64)/(cmp_count_std_total as f64));
                            }
                        } else {
                            let (throughput_std, trial_count_std) = run_test(alg_std, &v);
                            let (throughput_ns, trial_count_ns) = run_test(alg_ns, &v);
                            println!("{: >7} / {: >7} {: >7}{: >15}{: >15}{: >12.0}{: >12.0}{: >12.0}{: >12.0}{: >12.1}{: >12.1}{: >12.2}",
                                size, v.len(), m, pattern, variant,
                                trial_count_ns, trial_count_std,
                                cmp_count_ns, cmp_count_std,
                                throughput_ns, throughput_std,
                                throughput_ns / throughput_std);
                        }
                    }
                }
            }
        }
        if let TestType::BenchMark = test_type {
            eprintln!();
        }
    }
    if let TestType::BenchMark = test_type {
        eprintln!("benchmark completed; new to standard comparisons ratio: {:.4}",
            (cmp_count_ns_total as f64)/(cmp_count_std_total as f64));
    } else {
        eprintln!("{} equality test runs completed; new to standard comparisons ratio: {:.4}",
            n_runs, (cmp_count_ns_total as f64)/(cmp_count_std_total as f64));
    }
}
