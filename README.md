# Benchmarks for proposed two stage merge sort vs std TimSort

This repo includes all files used for benchmarking a proposed two stage merge sort with pre-sorted prefix optimization, vs Rust's current implementation of TimSort.  The two stages are:
* Top-down recursive depth-first merge, which helps data locality
* Small slices sort using a fast insertion sort, then merge

The new sort benefits from pre-sorted prefix optimization for forward and reverse sorted sub-slices.  The total running time is *O*(*n* \* log(*n*)) worst-case.

## TL;DR
The proposed two stage merge sort indicates a speedup of **13.1%** for random unsorted data, with **95%** of sort runs being faster.  For random data, including unsorted and forward / reverse sorted variants, the results indicate a speedup of **20.2%**, with **92%** of sort runs being faster.  Other data patterns are faster too, except for the `sawtooth` pattern, which is about the same.  The number of comparisons performed is **-2.92%** less over all tests.

## Background info

This benchmark is based on https://github.com/notriddle/quickersort/blob/master/examples/perf_txt.rs, 
(C) 2015 Michael Howell <michael@notriddle.com>

Updates to the benchmark include:
* test that new sort result is identical to standard sort (including stability)
* randomize vec sizes within decade (eg decade 100 has random sizes between 0 and 999)
* option for multiple benchmark runs
* option for quick eq testing with multiple vec sizes
* track comparison ratio between newsort and standard sort
* determine number of iterations using timer instead of fixed count
* add "reverse sorted" variant
* add parallel sort option
* call stdsort from a module or from lib-std

## How the becnhmark was run

The benchmark was run on six OS's / CPUs:
* Ubuntu 20.04.3 LTS / Intel(R) Core(TM) i7-8559U CPU @ 2.70GHz
* WIN 10 KVM on Ubuntu / Intel(R) Core(TM) i7-8559U CPU @ 2.70GHz
* macOS Big Sur Version 11.6 / MacBook Pro (16-inch, 2019) 2.4 GHz 8-Core Intel Core i9
* Debian Crostini Linux / Google PixelBook Intel(R) 7th Gen Core(TM) i7-7Y75
* Debian Crostini Linux / Asus Chromebook Flip CX5 Intel(R) 11th Gen Core(TM) i5-1130G7
* Debian Crostini Linux / HP Chromebook X2 11 Qualcomm(R) Snapdragon(TM) 7c

For each OS/CPU, the benchmark was run on vecs of `i16`, `i32`, `i64`, `i128`, and `String`.  The vecs were of random lengths from 0 to 10,000,000.

The results were collected in a separate log file for each OS/CPU/vec type combination, and then collected into a single `all` log file for each OS/CPU combination.  For instance, this command and its output, which was run on Ubuntu:

```
$ ./do-all-builds-std ub-ssf ressw2
i32
resstd/ub-ssf-i32.log
   Compiling newsort v1.0.0 (/home/wpwoodjr/rust/newsort)
    Finished release [optimized] target(s) in 9.80s
Std: Std, Newsort: Newsort
Range 10 to 1000000
String array size = 1095696
Running benchmark 1 of 1: strings... 10's... 100's... 1000's... 10000's... 100000's... 1000000's...
benchmark completed; new to standard comparisons ratio: 0.9609

real	8m19.979s
user	8m13.343s
sys	0m6.539s
Tue 02 Nov 2021 05:02:49 PM EDT
i16
resstd/ub-ssf-i16.log
   Compiling newsort v1.0.0 (/home/wpwoodjr/rust/newsort)
    Finished release [optimized] target(s) in 9.68s
Std: Std, Newsort: Newsort
Range 10 to 1000000
String array size = 1095696
Running benchmark 1 of 1: strings... 10's... 100's... 1000's... 10000's... 100000's... 1000000's...
benchmark completed; new to standard comparisons ratio: 0.9974

real	8m18.201s
user	8m13.943s
sys	0m4.164s
Tue 02 Nov 2021 05:16:16 PM EDT
i64
resstd/ub-ssf-i64.log
   Compiling newsort v1.0.0 (/home/wpwoodjr/rust/newsort)
    Finished release [optimized] target(s) in 9.62s
Std: Std, Newsort: Newsort
Range 10 to 1000000
String array size = 1095696
Running benchmark 1 of 1: strings... 10's... 100's... 1000's... 10000's... 100000's... 1000000's...
benchmark completed; new to standard comparisons ratio: 0.9800

real	8m57.096s
user	8m41.635s
sys	0m15.386s
Tue 02 Nov 2021 05:30:23 PM EDT
i128
resstd/ub-ssf-i128.log
   Compiling newsort v1.0.0 (/home/wpwoodjr/rust/newsort)
    Finished release [optimized] target(s) in 9.70s
Std: Std, Newsort: Newsort
Range 10 to 1000000
String array size = 1095696
Running benchmark 1 of 1: strings... 10's... 100's... 1000's... 10000's... 100000's... 1000000's...
benchmark completed; new to standard comparisons ratio: 0.9794

real	10m0.251s
user	9m27.921s
sys	0m32.248s
Tue 02 Nov 2021 05:45:33 PM EDT
$
```
resulted in these files:
```
$ ls -lat ressw2/ub*
-rw-rw-r-- 1 wpwoodjr wpwoodjr 807544 Oct 26 15:31 ressw2/ub-ssf-all.log
-rw-rw-r-- 1 wpwoodjr wpwoodjr 201886 Oct 26 15:31 ressw2/ub-ssf-i128.log
-rw-rw-r-- 1 wpwoodjr wpwoodjr 201886 Oct 26 15:16 ressw2/ub-ssf-i64.log
-rw-rw-r-- 1 wpwoodjr wpwoodjr 201886 Oct 26 15:03 ressw2/ub-ssf-i16.log
-rw-rw-r-- 1 wpwoodjr wpwoodjr 201886 Oct 26 14:49 ressw2/ub-ssf-i32.log
```

## Results

After running the benchmark on all OS/CPUs, the OS/CPU `all` files were collected into one big `all` file and statistics run as follows:
```
$ log=ressw2/all* ./do-stats
filter: /
all                               nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	 34584         33.1%           80%        -2.92%

forward sorted                    nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  4944         30.5%           87%         0.00%

not forward sorted or plateau     nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	 23736         14.4%           78%        -0.15%

reverse sorted:                   nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  4944         62.6%           87%       -14.27%

rand                              nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  6888         20.2%           92%        -2.14%

rand, not forw/rev sorted:        nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  4920         13.1%           95%        -0.67%

shuffle ident:                    nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   984         11.4%           70%         0.80%

sawtooth ident:                   nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   984          3.9%           49%         4.76%

stagger ident:                    nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   984          7.6%           75%         3.40%

strings:                          nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   144          6.6%           78%        -3.61%

strings, not forward sorted:      nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   120          6.6%           78%        -4.33%
```

From the above benchmark, the proposed two stage merge sort indicates a speedup of **13.1%** for random unsorted data, with **95%** of sort runs being faster.  For random data, including unsorted and forward / reverse sorted variants, the results indicate a speedup of **20.2%**, with **92%** of sort runs being faster.  Other data patterns are faster too, except for the `sawtooth` pattern, which is about the same.  The number of comparisons performed is **-2.92%** less over all tests.

These stats compare results by pattern / variant:
```
$ log=ressw2/all* ./do-stats-by-pattern 
filter: /
sawtooth                                          nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  6888         15.4%           68%         1.76%
  ident          ressw2/all-ssf-all.log:   	   984          3.9%           49%         4.76%
  reverse        ressw2/all-ssf-all.log:   	   984         -0.5%           44%        11.76%
  reverse_front  ressw2/all-ssf-all.log:   	   984         14.0%           54%         2.07%
  reverse_back   ressw2/all-ssf-all.log:   	   984          4.2%           54%         4.04%
  sorted         ressw2/all-ssf-all.log:   	   984         30.6%           88%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   984         42.3%           89%        -9.85%
  dither         ressw2/all-ssf-all.log:   	   984         13.4%           95%        -0.47%
rand                                              nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  6888         20.2%           92%        -2.14%
  ident          ressw2/all-ssf-all.log:   	   984         12.9%           95%        -0.73%
  reverse        ressw2/all-ssf-all.log:   	   984         13.1%           95%        -0.63%
  reverse_front  ressw2/all-ssf-all.log:   	   984         13.3%           95%        -0.87%
  reverse_back   ressw2/all-ssf-all.log:   	   984         12.8%           95%        -0.36%
  sorted         ressw2/all-ssf-all.log:   	   984         29.7%           86%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   984         45.9%           82%       -11.61%
  dither         ressw2/all-ssf-all.log:   	   984         13.5%           96%        -0.77%
stagger                                           nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  6888         14.2%           82%         1.15%
  ident          ressw2/all-ssf-all.log:   	   984          7.6%           75%         3.40%
  reverse        ressw2/all-ssf-all.log:   	   984          8.3%           78%         2.26%
  reverse_front  ressw2/all-ssf-all.log:   	   984          8.6%           77%         2.63%
  reverse_back   ressw2/all-ssf-all.log:   	   984         10.3%           86%         0.60%
  sorted         ressw2/all-ssf-all.log:   	   984         31.2%           88%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   984         21.5%           82%        -3.11%
  dither         ressw2/all-ssf-all.log:   	   984         11.8%           91%         2.29%
plateau                                           nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  6888         99.0%           83%       -14.15%
  ident          ressw2/all-ssf-all.log:   	   984         26.5%           82%         1.87%
  reverse        ressw2/all-ssf-all.log:   	   984        183.4%           94%       -40.39%
  reverse_front  ressw2/all-ssf-all.log:   	   984        245.3%           97%       -38.97%
  reverse_back   ressw2/all-ssf-all.log:   	   984         23.6%           82%         2.00%
  sorted         ressw2/all-ssf-all.log:   	   984         30.9%           87%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   984        186.3%           98%       -41.32%
  dither         ressw2/all-ssf-all.log:   	   984         -3.1%           39%        17.73%
shuffle                                           nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  6888         17.2%           76%        -1.21%
  ident          ressw2/all-ssf-all.log:   	   984         11.4%           70%         0.80%
  reverse        ressw2/all-ssf-all.log:   	   984          8.4%           63%        -0.42%
  reverse_front  ressw2/all-ssf-all.log:   	   984         30.9%           74%        -4.91%
  reverse_back   ressw2/all-ssf-all.log:   	   984          8.4%           64%         1.39%
  sorted         ressw2/all-ssf-all.log:   	   984         30.6%           87%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   984         18.0%           82%        -5.53%
  dither         ressw2/all-ssf-all.log:   	   984         12.5%           95%         0.19%
strings                                           nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	   144          6.6%           78%        -3.61%
  ident          ressw2/all-ssf-all.log:   	    24          4.8%           83%        -2.96%
  reverse        ressw2/all-ssf-all.log:   	    24          3.0%           71%        -2.45%
  reverse_front  ressw2/all-ssf-all.log:   	    24          3.4%           67%        -3.68%
  reverse_back   ressw2/all-ssf-all.log:   	    24          1.2%           71%        -1.07%
  sorted         ressw2/all-ssf-all.log:   	    24          6.4%           79%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	    24         20.7%          100%       -11.48%
  dither         ressw2/all-ssf-all.log:   	     0
```

These stats compare results by variant / pattern:
```
$ log=ressw2/all* ./do-stats-by-variant 
filter: /
ident                                             nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4944         12.4%           74%         2.00%
  sawtooth       ressw2/all-ssf-all.log:   	   984          3.9%           49%         4.76%
  rand           ressw2/all-ssf-all.log:   	   984         12.9%           95%        -0.73%
  stagger        ressw2/all-ssf-all.log:   	   984          7.6%           75%         3.40%
  plateau        ressw2/all-ssf-all.log:   	   984         26.5%           82%         1.87%
  shuffle        ressw2/all-ssf-all.log:   	   984         11.4%           70%         0.80%
  strings        ressw2/all-ssf-all.log:   	    24          4.8%           83%        -2.96%
reverse                                           nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4944         42.4%           75%        -5.47%
  sawtooth       ressw2/all-ssf-all.log:   	   984         -0.5%           44%        11.76%
  rand           ressw2/all-ssf-all.log:   	   984         13.1%           95%        -0.63%
  stagger        ressw2/all-ssf-all.log:   	   984          8.3%           78%         2.26%
  plateau        ressw2/all-ssf-all.log:   	   984        183.4%           94%       -40.39%
  shuffle        ressw2/all-ssf-all.log:   	   984          8.4%           63%        -0.42%
  strings        ressw2/all-ssf-all.log:   	    24          3.0%           71%        -2.45%
reverse_front                                     nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4944         62.1%           79%        -7.99%
  sawtooth       ressw2/all-ssf-all.log:   	   984         14.0%           54%         2.07%
  rand           ressw2/all-ssf-all.log:   	   984         13.3%           95%        -0.87%
  stagger        ressw2/all-ssf-all.log:   	   984          8.6%           77%         2.63%
  plateau        ressw2/all-ssf-all.log:   	   984        245.3%           97%       -38.97%
  shuffle        ressw2/all-ssf-all.log:   	   984         30.9%           74%        -4.91%
  strings        ressw2/all-ssf-all.log:   	    24          3.4%           67%        -3.68%
reverse_back                                      nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4944         11.8%           76%         1.52%
  sawtooth       ressw2/all-ssf-all.log:   	   984          4.2%           54%         4.04%
  rand           ressw2/all-ssf-all.log:   	   984         12.8%           95%        -0.36%
  stagger        ressw2/all-ssf-all.log:   	   984         10.3%           86%         0.60%
  plateau        ressw2/all-ssf-all.log:   	   984         23.6%           82%         2.00%
  shuffle        ressw2/all-ssf-all.log:   	   984          8.4%           64%         1.39%
  strings        ressw2/all-ssf-all.log:   	    24          1.2%           71%        -1.07%
sorted                                            nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4944         30.5%           87%         0.00%
  sawtooth       ressw2/all-ssf-all.log:   	   984         30.6%           88%         0.00%
  rand           ressw2/all-ssf-all.log:   	   984         29.7%           86%         0.00%
  stagger        ressw2/all-ssf-all.log:   	   984         31.2%           88%         0.00%
  plateau        ressw2/all-ssf-all.log:   	   984         30.9%           87%         0.00%
  shuffle        ressw2/all-ssf-all.log:   	   984         30.6%           87%         0.00%
  strings        ressw2/all-ssf-all.log:   	    24          6.4%           79%         0.00%
reverse_sorted                                    nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4944         62.6%           87%       -14.27%
  sawtooth       ressw2/all-ssf-all.log:   	   984         42.3%           89%        -9.85%
  rand           ressw2/all-ssf-all.log:   	   984         45.9%           82%       -11.61%
  stagger        ressw2/all-ssf-all.log:   	   984         21.5%           82%        -3.11%
  plateau        ressw2/all-ssf-all.log:   	   984        186.3%           98%       -41.32%
  shuffle        ressw2/all-ssf-all.log:   	   984         18.0%           82%        -5.53%
  strings        ressw2/all-ssf-all.log:   	    24         20.7%          100%       -11.48%
dither                                            nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4920          9.6%           83%         3.79%
  sawtooth       ressw2/all-ssf-all.log:   	   984         13.4%           95%        -0.47%
  rand           ressw2/all-ssf-all.log:   	   984         13.5%           96%        -0.77%
  stagger        ressw2/all-ssf-all.log:   	   984         11.8%           91%         2.29%
  plateau        ressw2/all-ssf-all.log:   	   984         -3.1%           39%        17.73%
  shuffle        ressw2/all-ssf-all.log:   	   984         12.5%           95%         0.19%
  strings        ressw2/all-ssf-all.log:   	     0
```
