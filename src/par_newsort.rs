// par_newsort-5a

use std::mem::{size_of};

macro_rules! lt {
    ($v: ident, $left: expr, $right: expr, $is_less: ident) => {
        // $cmp(&$v[$left], &$v[$right])
        $is_less(unsafe {&$v.get_unchecked($left)}, unsafe {&$v.get_unchecked($right)})
    };
}
macro_rules! gt {
    ($v: ident, $left: expr, $right: expr, $is_less: ident) => {
        lt!($v, $right, $left, $is_less)
    };
}
macro_rules! le {
    ($v: ident, $left: expr, $right: expr, $is_less: ident) => {
        ! gt!($v, $left, $right, $is_less)
    };
}

#[inline]
pub fn par_sort<T>(v: &mut [T])
where
    T: Ord + Send + Sync,
{
    merge_sort(v, |a, b| a.lt(b));
}

use std::cmp::Ordering::{self, Less};
#[inline]
pub fn par_sort_by<T, F>(v: &mut [T], compare: F)
where
    T: Send + Sync,
    F: Fn(&T, &T) -> Ordering + Sync,
{
    merge_sort(v, |a, b| compare(a, b) == Less);
}

const SMALL_CHUNK_SIZE: usize = 10;
const PAR_CHUNK_SIZE: usize = SMALL_CHUNK_SIZE*256;

fn merge_sort<T, F>(v: &mut [T], is_less: F)
where
    T: Send + Sync,
    F: Fn(&T, &T) -> bool + Sync,
{
    // Slices of up to this length get sorted using insertion sort.
    const MAX_INSERTION: usize = 20;

    // Sorting has no meaningful behavior on zero-sized types.
    if size_of::<T>() == 0 {
        return;
    }

    let len = v.len();
    let is_less = &is_less;

    if len <= MAX_INSERTION {
        // Short arrays get sorted in-place via insertion sort to avoid allocations.
        if len > 1 {
            for i in (0..len - 1).rev() {
                insert_head(&mut v[i..], is_less);
            }
        }
        return;
    }

    // Allocate a buffer to use as scratch memory. We keep the length 0 so we can keep in it
    // shallow copies of the contents of `v` without risking the dtors running on copies if
    // `is_less` panics. When merging two sorted runs, this buffer holds a copy of the shorter run,
    // which will always have length at most `len / 2`.
    let mut buf = Vec::with_capacity(len);
    let num_threads = rayon::current_num_threads();

    unsafe { buf.set_len(len) }
    par_chunks_sort(v, num_threads, 0, &mut buf, is_less);
    unsafe { buf.set_len(0) }

    fn par_chunks_sort<T, F>(v: &mut [T], num_threads: usize, depth: usize, buf: &mut [T], is_less: &F) -> bool
    where
        T: Send + Sync,
        F: Fn(&T, &T) -> bool + Sync,
    {
        let len = v.len();

        if len < PAR_CHUNK_SIZE || num_threads < 2 {
            large_chunks_sort(v, 0, buf.as_mut_ptr(), is_less);
            false
        } else {
            let mid = (len + 1)/2;
            let (lo, hi) = v.split_at_mut(mid);
            let (buf_lo, buf_hi) = buf.split_at_mut(mid);
            let (v, buf, swapped) = match rayon::join(
                    || par_chunks_sort(lo, num_threads/2, depth + 1, buf_lo, is_less),
                    || par_chunks_sort(hi, num_threads/2, depth + 1, buf_hi, is_less)) {
                (false, false) => {
                    (v, buf, false)
                }
                (false, true) => {
                    unsafe { std::ptr::copy_nonoverlapping(buf_hi.as_ptr(), hi.as_mut_ptr(), len - mid) }
                    (v, buf, false)
                }
                (true, false) => {
                    unsafe { std::ptr::copy_nonoverlapping(buf_lo.as_ptr(), lo.as_mut_ptr(), mid) }
                    (v, buf, false)
                }
                (true, true) => {
                    (buf, v, true)
                }
            };
            if depth == 0 && swapped {
                par_merge(v, mid, buf, is_less, num_threads);
                false
            } else if gt!(v, mid - 1, mid, is_less) {
                if gt!(v, 0, len - 1, is_less) {  // strictly reverse sorted?
                    swap_buf(v, mid, buf.as_mut_ptr());
                    swapped
                } else if depth > 0 && len >= PAR_CHUNK_SIZE*2 {
                    par_merge(v, mid, buf, is_less, num_threads);
                    ! swapped
                } else {
                    merge(v, mid, buf.as_mut_ptr(), is_less);
                    swapped
                }
            } else {
                swapped
            }
        }
    }

    fn large_chunks_sort<T, F>(v: &mut [T], mut sorted: usize, buf_ptr: *mut T, is_less: &F)
    where
        F: Fn(&T, &T) -> bool + Sync,
    {
        let len = v.len();
        if sorted == 0 {
            sorted = check_prefix_sort(v, is_less);
        }

        if sorted < len {
            debug_assert!(len > 2);
            if len <= SMALL_CHUNK_SIZE + 2 {
                for i in (0..len - 1).rev() {
                    insert_head(&mut v[i..len], is_less);
                }
            } else {
                let mid;

                if len > SMALL_CHUNK_SIZE*2 {
                    mid = (len + 1)/2;
                    if sorted < mid {
                        large_chunks_sort(&mut v[..mid], sorted, buf_ptr, is_less);
                        large_chunks_sort(&mut v[mid..], 0, buf_ptr, is_less);        
                    } else {
                        large_chunks_sort(&mut v[mid..], sorted - mid, buf_ptr, is_less);
                    }
                } else {
                    mid = SMALL_CHUNK_SIZE;
                    if sorted < SMALL_CHUNK_SIZE {
                        for i in (0..SMALL_CHUNK_SIZE - 1).rev() {
                            insert_head(&mut v[i..SMALL_CHUNK_SIZE], is_less);
                        }
                    }
                    for i in (SMALL_CHUNK_SIZE..len - 1).rev() {
                        insert_head(&mut v[i..], is_less);
                    }
                }
                if gt!(v, mid - 1, mid, is_less) {
                    if gt!(v, 0, len - 1, is_less) {  // strictly reverse sorted?
                        swap_buf(v, mid, buf_ptr);
                    } else {
                        merge(v, mid, buf_ptr, is_less);
                    }
                }
            }
        }
    }

    struct MergePart<'a, T>(&'a [T], &'a [T], &'a mut [T]);
    unsafe impl<T> Send for MergePart<'_, T> {}
    
    use rayon::prelude::*;
    fn par_merge<T, F>(v: &mut [T], mid: usize, buf: &mut [T], is_less: &F, num_threads: usize)
    where
        T: Sync,
        F: Fn(&T, &T) -> bool + Sync,
    {
        let (mut a, mut b) = v.split_at(mid);
        // let mut c = buf;
        let (_, mut c) = buf.split_at_mut(0); // ???

        // let num_threads = 1.max(rayon::current_num_threads()/2);
        // num_threads = num_threads.max(2);
        let psize = PAR_CHUNK_SIZE.max((b.len() + num_threads - 1)/num_threads);
        let p: Vec<usize> = b.par_chunks(psize)
            .map(|bpar| binary_search(a, &bpar[bpar.len() - 1], is_less))
            .collect();
    
        let mut parts: Vec<MergePart<T>> = vec![];
        let mut j = 0;
        for i in 0..p.len() {
            debug_assert!(p[i] >= j);
            let split = p[i] - j;
            let (al, ar) = a.split_at(split);
            j += split;
            a = ar;
            let (bl, br) = b.split_at(psize.min(b.len()));
            b = br;
            let (cl, cr) = c.split_at_mut(al.len() + bl.len());
            c = cr;
            parts.push(MergePart(al, bl, cl));
        }
        if a.len() != 0 || b.len() != 0 {
            parts.push(MergePart(a, b, c));
        }

        // const PART_RESPLIT_FACTOR: usize = 2;
        parts.par_iter_mut().for_each(|part|
            // if part.0.len() <= PART_RESPLIT_FACTOR*psize || part.1.len() == 0 {
                seq_merge(part.0, part.1, part.2, is_less));
            // } else {
            //     // dbg!(part.0.len(), part.1.len(), psize);
            //     par_merge(part.1, part.0, part.2, is_less, psize);
            // });
        // let len = v.len();
        // let (a, b) = v.split_at_mut(mid);
        // unsafe {
        //     // let (b1, b2) = split_at_mut_unchecked(buf, mid);
        //     // rayon::join(|| std::ptr::copy_nonoverlapping(b1.as_ptr(), a.as_mut_ptr(), mid),
        //     //             || std::ptr::copy_nonoverlapping(b2.as_ptr(), b.as_mut_ptr(), len - mid));
        //     std::ptr::copy_nonoverlapping(buf.as_ptr(), v.as_mut_ptr(), v.len());
        // }
    }
    
    fn seq_merge<T, F>(a: &[T], b: &[T], c: &mut [T], is_less: &F)
    where
        F: Fn(&T, &T) -> bool,
    {
        // println!("<- {:?} {:?}", a, b);
        let (mut l, mut r, alen, blen, mut clen) = (0, 0, a.len(), b.len(), 0);
        let (a_ptr, b_ptr, c_mut_ptr) = (a.as_ptr(), b.as_ptr(), c.as_mut_ptr());
        if r < blen {
            while l < alen {
                if clen > 0 || is_less(&b[r], &a[l]) {
                    let mut n = 1;
                    while r + n < blen && is_less(&b[r + n], &a[l]) {
                        n += 1;
                    }
                    unsafe {
                        std::ptr::copy_nonoverlapping(b_ptr.add(r), c_mut_ptr.add(clen), n);
                    }
                    r += n;
                    clen += n;
                    if r >= blen {
                        break;
                    }
                }
                let mut n = 1;
                while l + n < alen && ! is_less(&b[r], &a[l + n]) {
                    n += 1;
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(a_ptr.add(l), c_mut_ptr.add(clen), n);
                }
                l += n;
                clen += n;
            }
        }
        unsafe {
            if r < blen {
                std::ptr::copy_nonoverlapping(b_ptr.add(r), c_mut_ptr.add(clen), blen - r);
            } else {
                std::ptr::copy_nonoverlapping(a_ptr.add(l), c_mut_ptr.add(clen), alen - l);
            }
        }
        // println!("-> {:?}", c);
    }
    
    fn binary_search<T, F>(v: &[T], x: &T, is_less: &F) -> usize
    where
        F: Fn(&T, &T) -> bool,
    {
        debug_assert!(v.len() > 0);
        let mid = v.len()/2;
        if mid == 0 {
            if is_less(x, &v[0]) {
                0
            } else {
                1
            }
        } else {
            if is_less(x, &v[mid]) {
                binary_search(&v[..mid], x, is_less)
            } else {
                mid + binary_search(&v[mid..], x, is_less)
            }
        }
    }

    // find length of sorted prefix and reverse it if strictly descending
    #[inline(always)]
    fn check_prefix_sort<T, F>(v: &mut [T], is_less: &F) -> usize
    where
        F: Fn(&T, &T) -> bool,
    {
        let len = v.len();
        if len <= 1 {
            len
        } else if gt!(v, 0, 1, is_less) { // strictly descending
            let mut i = 2;
            while i < len && gt!(v, i - 1, i, is_less) {
                i += 1;
            }
            v[..i].reverse();
            i
        } else {
            let mut i = 2;
            while i < len && le!(v, i - 1, i, is_less) {
                i += 1;
            }
            i
        }
    }

    use std::ptr;
    fn swap_buf<T>(v: &mut [T], mid: usize, buf_ptr: *mut T) {
        let rlen = v.len() - mid;
        let v_ptr = v.as_mut_ptr();
        unsafe {
            ptr::copy_nonoverlapping(v_ptr.add(mid), buf_ptr, rlen);
            ptr::copy(v_ptr, v_ptr.add(rlen), mid);
            ptr::copy_nonoverlapping(buf_ptr, v_ptr, rlen);
        }
    }

    unsafe fn _split_at_mut_unchecked<T>(buf: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
        let buf_ptr = buf.as_mut_ptr();
        (std::slice::from_raw_parts_mut(buf_ptr, 0), std::slice::from_raw_parts_mut(buf_ptr.add(mid), 0))
    }

    /// Merges non-decreasing runs `v[..mid]` and `v[mid..]` using `buf_ptr` as temporary storage, and
    /// stores the result into `v[..]`.
    ///
    /// # Safety
    ///
    /// The two slices must be non-empty and `mid` must be in bounds and >= `v.len() - mid`. Buffer `buf_ptr` must be
    /// long enough to hold a copy of the right-hand slice. Also, `T` must not be a zero-sized type.
    fn merge<T, F>(v: &mut [T], mid: usize, buf_ptr: *mut T, is_less: &F)
    where
        F: Fn(&T, &T) -> bool,
    {
        let len = v.len();
        let rlen = len - mid;
        debug_assert!(mid >= rlen);
        let v = v.as_mut_ptr();
        let (v_mid, v_end) = unsafe { (v.add(mid), v.add(len)) };
    
        // The merge process first copies the right side into `buf_ptr`. Then it traces the newly copied
        // run and the longer run forwards (or backwards), comparing their next unconsumed elements and
        // copying the lesser (or greater) one into `v`.
        //
        // As soon as the shorter run is fully consumed, the process is done. If the longer run gets
        // consumed first, then we must copy whatever is left of the shorter run into the remaining
        // hole in `v`.
        //
        // Intermediate state of the process is always tracked by `hole`, which serves two purposes:
        // 1. Protects integrity of `v` from panics in `is_less`.
        // 2. Fills the remaining hole in `v` if the longer run gets consumed first.
        //
        // Panic safety:
        //
        // If `is_less` panics at any point during the process, `hole` will get dropped and fill the
        // hole in `v` with the unconsumed range in `buf_ptr`, thus ensuring that `v` still holds every
        // object it initially held exactly once.
        let mut hole;

        unsafe {
            ptr::copy_nonoverlapping(v_mid, buf_ptr, rlen);
            hole = MergeHole { start: buf_ptr, end: buf_ptr.add(rlen), dest: v_mid };
        }

        // Initially, these pointers point past the ends of their arrays.
        let left = &mut hole.dest;
        let right = &mut hole.end;
        let mut out = v_end;

        while v < *left && buf_ptr < *right {
            // Consume the greater side.
            // If equal, prefer the right run to maintain stability.
            unsafe {
                let to_copy = if is_less(&*right.offset(-1), &*left.offset(-1)) {
                    decrement_and_get(left)
                } else {
                    decrement_and_get(right)
                };
                ptr::copy_nonoverlapping(to_copy, decrement_and_get(&mut out), 1);
            }
        }

        // Finally, `hole` gets dropped. If the shorter run was not fully consumed, whatever remains of
        // it will now be copied into the hole in `v`.

        #[inline(always)]
        fn decrement_and_get<T>(ptr: &mut *mut T) -> *mut T {
            *ptr = unsafe { ptr.offset(-1) };
            *ptr
        }

        // When dropped, copies the range `start..end` into `dest..`.
        struct MergeHole<T> {
            start: *mut T,
            end: *mut T,
            dest: *mut T,
        }
    
        impl<T> Drop for MergeHole<T> {
            fn drop(&mut self) {
                // `T` is not a zero-sized type, so it's okay to divide by its size.
                // let len = (self.end as usize - self.start as usize) / size_of::<T>();
                unsafe {
                    let len = self.end.offset_from(self.start) as usize;
                    ptr::copy_nonoverlapping(self.start, self.dest, len);
                }
            }
        }
    }

    /// Inserts `v[0]` into pre-sorted sequence `v[1..]` so that whole `v[..]` becomes sorted.
    ///
    /// This is the integral subroutine of insertion sort.
    #[inline(always)]
    fn insert_head<T, F>(v: &mut [T], is_less: &F)
    where
        F: Fn(&T, &T) -> bool,
    {
        if v.len() >= 2 && is_less(&v[1], &v[0]) {
            unsafe {
                // There are three ways to implement insertion here:
                //
                // 1. Swap adjacent elements until the first one gets to its final destination.
                //    However, this way we copy data around more than is necessary. If elements are big
                //    structures (costly to copy), this method will be slow.
                //
                // 2. Iterate until the right place for the first element is found. Then shift the
                //    elements succeeding it to make room for it and finally place it into the
                //    remaining hole. This is a good method.
                //
                // 3. Copy the first element into a temporary variable. Iterate until the right place
                //    for it is found. As we go along, copy every traversed element into the slot
                //    preceding it. Finally, copy data from the temporary variable into the remaining
                //    hole. This method is very good. Benchmarks demonstrated slightly better
                //    performance than with the 2nd method.
                //
                // All methods were benchmarked, and the 3rd showed best results. So we chose that one.
                let mut tmp = core::mem::ManuallyDrop::new(ptr::read(&v[0]));
    
                // Intermediate state of the insertion process is always tracked by `hole`, which
                // serves two purposes:
                // 1. Protects integrity of `v` from panics in `is_less`.
                // 2. Fills the remaining hole in `v` in the end.
                //
                // Panic safety:
                //
                // If `is_less` panics at any point during the process, `hole` will get dropped and
                // fill the hole in `v` with `tmp`, thus ensuring that `v` still holds every object it
                // initially held exactly once.
                let mut hole = InsertionHole { src: &mut *tmp, dest: &mut v[1] };
                ptr::copy_nonoverlapping(&v[1], &mut v[0], 1);
    
                for i in 2..v.len() {
                    if !is_less(&v[i], &*tmp) {
                        break;
                    }
                    ptr::copy_nonoverlapping(&v[i], &mut v[i - 1], 1);
                    hole.dest = &mut v[i];
                }
                // `hole` gets dropped and thus copies `tmp` into the remaining hole in `v`.
            }
        }
    
        // When dropped, copies from `src` into `dest`.
        struct InsertionHole<T> {
            src: *mut T,
            dest: *mut T,
        }
    
        impl<T> Drop for InsertionHole<T> {
            fn drop(&mut self) {
                unsafe {
                    ptr::copy_nonoverlapping(self.src, self.dest, 1);
                }
            }
        }
    }
}
