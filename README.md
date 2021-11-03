# Benchmarks for proposed two stage merge sort vs std TimSort

This repo includes all files used for benchmarking a proposed two stage merge sort with pre-sorted prefix optimization, vs Rust's current implementation of TimSort.  The two stages are:
* Top-down recursive depth-first merge, which helps data locality
* Small slices sort using a fast insertion sort, then merge
The total running time is *O*(*n* \* log(*n*)) worst-case.


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

After running the benchmark on all OS/CPUs, the OS/CPU `all` files were collected into one big `all` file and statistics run as follows:
```
$ log="ressw2/all*" ./do-stats
filter: 00 /
all                               nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	 29544         31.9%           78%        -3.41%

forward sorted                    nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  4224         28.0%           85%         0.00%

not forward sorted or plateau     nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	 20280         12.1%           76%        -0.89%

reverse sorted:                   nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  4224         62.3%           85%       -14.07%

rand                              nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  5880         18.4%           91%        -2.65%

rand, not forw/rev sorted:        nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	  4200         10.8%           95%        -0.99%

shuffle ident:                    nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   840          8.5%           67%        -0.22%

sawtooth ident:                   nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   840         -0.3%           41%         5.91%

stagger ident:                    nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   840          6.2%           74%         1.93%

strings:                          nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   144          6.6%           78%        -3.61%

strings, not forward sorted:      nrec     % speedup     is faster     cmp ratio
  ressw2/all-ssf-all.log:   	   120          6.6%           78%        -4.33%
```

From the above, benchmarking the proposed two stage merge sort vs TimSort indicates a speedup of **10.8%** for random unsorted data, with **95%** of sort runs being faster.  For random sorted data, the results indicate a speedup of **18.4%**, with **91%** of sort runs being faster.  Other data patterns are faster too, except for the `sawtooth` pattern, which has a slight penalty of **-0.3%**.

These stats compare results by pattern / variant:
```
$ log="ressw2/all*" ./do-stats-by-pattern
filter: 00 /
sawtooth                                      nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  5880         12.4%           63%         1.31%
  ident          ressw2/all-ssf-all.log:   	   840         -0.3%           41%         5.91%
  reverse        ressw2/all-ssf-all.log:   	   840         -3.6%           37%         8.60%
  reverse_front  ressw2/all-ssf-all.log:   	   840         12.4%           47%         0.62%
  reverse_back   ressw2/all-ssf-all.log:   	   840          0.6%           47%         4.03%
  sorted         ressw2/all-ssf-all.log:   	   840         28.1%           86%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   840         38.8%           87%        -8.78%
  dither         ressw2/all-ssf-all.log:   	   840         10.9%           94%        -1.21%
rand                                          nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  5880         18.4%           91%        -2.65%
  ident          ressw2/all-ssf-all.log:   	   840         10.5%           94%        -1.07%
  reverse        ressw2/all-ssf-all.log:   	   840         10.8%           94%        -0.94%
  reverse_front  ressw2/all-ssf-all.log:   	   840         10.9%           95%        -0.90%
  reverse_back   ressw2/all-ssf-all.log:   	   840         10.8%           95%        -1.03%
  sorted         ressw2/all-ssf-all.log:   	   840         27.2%           83%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   840         47.2%           80%       -13.60%
  dither         ressw2/all-ssf-all.log:   	   840         11.2%           95%        -1.03%
stagger                                       nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  5880         12.1%           81%         0.37%
  ident          ressw2/all-ssf-all.log:   	   840          6.2%           74%         1.93%
  reverse        ressw2/all-ssf-all.log:   	   840          7.0%           77%         1.42%
  reverse_front  ressw2/all-ssf-all.log:   	   840          7.0%           76%         0.90%
  reverse_back   ressw2/all-ssf-all.log:   	   840          8.4%           85%         0.65%
  sorted         ressw2/all-ssf-all.log:   	   840         28.7%           85%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   840         17.4%           79%        -3.37%
  dither         ressw2/all-ssf-all.log:   	   840         10.2%           92%         1.05%
plateau                                       nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  5880        102.6%           81%       -14.05%
  ident          ressw2/all-ssf-all.log:   	   840         23.4%           79%         2.19%
  reverse        ressw2/all-ssf-all.log:   	   840        191.9%           93%       -39.06%
  reverse_front  ressw2/all-ssf-all.log:   	   840        265.4%           97%       -40.65%
  reverse_back   ressw2/all-ssf-all.log:   	   840         19.9%           79%         2.34%
  sorted         ressw2/all-ssf-all.log:   	   840         28.5%           85%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   840        195.4%           98%       -40.15%
  dither         ressw2/all-ssf-all.log:   	   840         -6.0%           35%        17.01%
shuffle                                       nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  5880         14.6%           73%        -2.02%
  ident          ressw2/all-ssf-all.log:   	   840          8.5%           67%        -0.22%
  reverse        ressw2/all-ssf-all.log:   	   840          6.1%           60%        -2.90%
  reverse_front  ressw2/all-ssf-all.log:   	   840         29.2%           70%        -6.38%
  reverse_back   ressw2/all-ssf-all.log:   	   840          5.5%           59%         0.49%
  sorted         ressw2/all-ssf-all.log:   	   840         28.2%           84%         0.00%
  reverse_sorted ressw2/all-ssf-all.log:   	   840         14.0%           79%        -4.54%
  dither         ressw2/all-ssf-all.log:   	   840         10.5%           94%        -0.58%
strings                                       nrec     % speedup     is faster     cmp ratio
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
$ log="ressw2/all*" ./do-stats-by-variant
filter: 00 /
ident                                         nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4224          9.6%           71%         1.72%
  sawtooth       ressw2/all-ssf-all.log:   	   840         -0.3%           41%         5.91%
  rand           ressw2/all-ssf-all.log:   	   840         10.5%           94%        -1.07%
  stagger        ressw2/all-ssf-all.log:   	   840          6.2%           74%         1.93%
  plateau        ressw2/all-ssf-all.log:   	   840         23.4%           79%         2.19%
  shuffle        ressw2/all-ssf-all.log:   	   840          8.5%           67%        -0.22%
  strings        ressw2/all-ssf-all.log:   	    24          4.8%           83%        -2.96%
reverse                                       nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4224         42.2%           72%        -6.55%
  sawtooth       ressw2/all-ssf-all.log:   	   840         -3.6%           37%         8.60%
  rand           ressw2/all-ssf-all.log:   	   840         10.8%           94%        -0.94%
  stagger        ressw2/all-ssf-all.log:   	   840          7.0%           77%         1.42%
  plateau        ressw2/all-ssf-all.log:   	   840        191.9%           93%       -39.06%
  shuffle        ressw2/all-ssf-all.log:   	   840          6.1%           60%        -2.90%
  strings        ressw2/all-ssf-all.log:   	    24          3.0%           71%        -2.45%
reverse_front                                 nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4224         64.6%           77%        -9.25%
  sawtooth       ressw2/all-ssf-all.log:   	   840         12.4%           47%         0.62%
  rand           ressw2/all-ssf-all.log:   	   840         10.9%           95%        -0.90%
  stagger        ressw2/all-ssf-all.log:   	   840          7.0%           76%         0.90%
  plateau        ressw2/all-ssf-all.log:   	   840        265.4%           97%       -40.65%
  shuffle        ressw2/all-ssf-all.log:   	   840         29.2%           70%        -6.38%
  strings        ressw2/all-ssf-all.log:   	    24          3.4%           67%        -3.68%
reverse_back                                  nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4224          9.0%           73%         1.28%
  sawtooth       ressw2/all-ssf-all.log:   	   840          0.6%           47%         4.03%
  rand           ressw2/all-ssf-all.log:   	   840         10.8%           95%        -1.03%
  stagger        ressw2/all-ssf-all.log:   	   840          8.4%           85%         0.65%
  plateau        ressw2/all-ssf-all.log:   	   840         19.9%           79%         2.34%
  shuffle        ressw2/all-ssf-all.log:   	   840          5.5%           59%         0.49%
  strings        ressw2/all-ssf-all.log:   	    24          1.2%           71%        -1.07%
sorted                                        nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4224         28.0%           85%         0.00%
  sawtooth       ressw2/all-ssf-all.log:   	   840         28.1%           86%         0.00%
  rand           ressw2/all-ssf-all.log:   	   840         27.2%           83%         0.00%
  stagger        ressw2/all-ssf-all.log:   	   840         28.7%           85%         0.00%
  plateau        ressw2/all-ssf-all.log:   	   840         28.5%           85%         0.00%
  shuffle        ressw2/all-ssf-all.log:   	   840         28.2%           84%         0.00%
  strings        ressw2/all-ssf-all.log:   	    24          6.4%           79%         0.00%
reverse_sorted                                nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4224         62.3%           85%       -14.07%
  sawtooth       ressw2/all-ssf-all.log:   	   840         38.8%           87%        -8.78%
  rand           ressw2/all-ssf-all.log:   	   840         47.2%           80%       -13.60%
  stagger        ressw2/all-ssf-all.log:   	   840         17.4%           79%        -3.37%
  plateau        ressw2/all-ssf-all.log:   	   840        195.4%           98%       -40.15%
  shuffle        ressw2/all-ssf-all.log:   	   840         14.0%           79%        -4.54%
  strings        ressw2/all-ssf-all.log:   	    24         20.7%          100%       -11.48%
dither                                        nrec     % speedup     is faster     cmp ratio
  all            ressw2/all-ssf-all.log:   	  4200          7.4%           82%         3.05%
  sawtooth       ressw2/all-ssf-all.log:   	   840         10.9%           94%        -1.21%
  rand           ressw2/all-ssf-all.log:   	   840         11.2%           95%        -1.03%
  stagger        ressw2/all-ssf-all.log:   	   840         10.2%           92%         1.05%
  plateau        ressw2/all-ssf-all.log:   	   840         -6.0%           35%        17.01%
  shuffle        ressw2/all-ssf-all.log:   	   840         10.5%           94%        -0.58%
  strings        ressw2/all-ssf-all.log:   	     0
```
