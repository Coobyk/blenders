//! Port of `sort.cc` — `BLI_qsort_r` (Bentley–McIlroy 3-way qsort over
//! raw elements) and `parallel_sort` (serial `sort_unstable` here).

/// `BLI_sort_cmp_t` — returns <0, 0, >0.
pub type SortCmp<'a> = &'a mut dyn FnMut(&[u8], &[u8]) -> std::cmp::Ordering;

/// `BLI_qsort_r` — sorts `n` elements of `es` bytes at `a`.
/// Faithful port of the C 3-way quicksort.
pub unsafe fn qsort_r(a: *mut u8, mut n: usize, es: usize, mut cmp: SortCmp, _thunk: usize) {
    unsafe {
        let mut base = a;
        let mut nn = n;
        'outer: loop {
            n = nn;
            let mut swap_cnt = 0i32;
            if n < 7 {
                // insertion sort
                for i in 1..n {
                    let mut pm = base.add(i * es);
                    while pm > base
                        && cmp(
                            std::slice::from_raw_parts(pm.sub(es), es),
                            std::slice::from_raw_parts(pm, es),
                        ) == std::cmp::Ordering::Greater
                    {
                        swapn(pm, pm.sub(es), es);
                        pm = pm.sub(es);
                    }
                }
                return;
            }
            let mut pm = base.add((n / 2) * es);
            if n > 7 {
                let mut pl = base;
                let mut pn = base.add((n - 1) * es);
                if n > 40 {
                    let d = (n / 8) * es;
                    pl = med3(pl, pl.add(d), pl.add(2 * d), es, &mut cmp);
                    pm = med3(pm.sub(d), pm, pm.add(d), es, &mut cmp);
                    pn = med3(pn.sub(2 * d), pn.sub(d), pn, es, &mut cmp);
                }
                pm = med3(pl, pm, pn, es, &mut cmp);
            }
            // swap(a, pm) — move pivot to front
            swapn(base, pm, es);
            let mut pa = base.add(es);
            let mut pb = pa;
            let mut pc = base.add((n - 1) * es);
            let mut pd = pc;
            loop {
                while pb <= pc {
                    let r = cmp(
                        std::slice::from_raw_parts(pb, es),
                        std::slice::from_raw_parts(base, es),
                    );
                    if r == std::cmp::Ordering::Greater {
                        break;
                    }
                    if r == std::cmp::Ordering::Equal {
                        swap_cnt = 1;
                        swapn(pa, pb, es);
                        pa = pa.add(es);
                    }
                    pb = pb.add(es);
                }
                while pb <= pc {
                    let r = cmp(
                        std::slice::from_raw_parts(pc, es),
                        std::slice::from_raw_parts(base, es),
                    );
                    if r == std::cmp::Ordering::Less {
                        break;
                    }
                    if r == std::cmp::Ordering::Equal {
                        swap_cnt = 1;
                        swapn(pc, pd, es);
                        pd = pd.sub(es);
                    }
                    pc = pc.sub(es);
                }
                if pb > pc {
                    break;
                }
                swapn(pb, pc, es);
                swap_cnt = 1;
                pb = pb.add(es);
                pc = pc.sub(es);
            }
            if swap_cnt == 0 {
                // insertion sort fallback
                for i in 1..n {
                    let mut pm2 = base.add(i * es);
                    while pm2 > base
                        && cmp(
                            std::slice::from_raw_parts(pm2.sub(es), es),
                            std::slice::from_raw_parts(pm2, es),
                        ) == std::cmp::Ordering::Greater
                    {
                        swapn(pm2, pm2.sub(es), es);
                        pm2 = pm2.sub(es);
                    }
                }
                return;
            }
            let pn = base.add(n * es);
            let mut r = std::cmp::min(pa as usize - base as usize, pb as usize - pa as usize);
            vecswap(base, pb.sub(r), r, es);
            r = std::cmp::min(pd as usize - pc as usize, pn as usize - pd as usize - es);
            vecswap(pb, pn.sub(r), r, es);
            let r_pa = pb as usize - pa as usize;
            if r_pa > es {
                qsort_r(base, r_pa / es, es, &mut cmp, _thunk);
            }
            let r_pd = pd as usize - pc as usize;
            if r_pd > es {
                base = pc.add(es);
                nn = r_pd / es;
                continue 'outer;
            }
            return;
        }
    }
}

unsafe fn swapn(a: *mut u8, b: *mut u8, n: usize) {
    unsafe {
        std::ptr::swap_nonoverlapping(a, b, n);
    }
}
unsafe fn vecswap(a: *mut u8, b: *mut u8, n: usize, es: usize) {
    if n > 0 {
        unsafe {
            let (a, b) = (a, b);
            // swap n bytes starting at a/b in `es`-sized pieces
            let mut i = 0usize;
            while i < n {
                swapn(a.add(i), b.add(i), es.min(n - i));
                i += es;
            }
        }
    }
}
unsafe fn med3(
    a: *mut u8,
    b: *mut u8,
    c: *mut u8,
    es: usize,
    cmp: &mut SortCmp,
) -> *mut u8 {
    unsafe {
        let ab = cmp(
            std::slice::from_raw_parts(a, es),
            std::slice::from_raw_parts(b, es),
        );
        let bc = cmp(
            std::slice::from_raw_parts(b, es),
            std::slice::from_raw_parts(c, es),
        );
        let ac = cmp(
            std::slice::from_raw_parts(a, es),
            std::slice::from_raw_parts(c, es),
        );
        match ab {
            std::cmp::Ordering::Less => match bc {
                std::cmp::Ordering::Less => b,
                _ => {
                    if ac == std::cmp::Ordering::Less {
                        c
                    } else {
                        a
                    }
                }
            },
            _ => match bc {
                std::cmp::Ordering::Greater => b,
                _ => {
                    if ac == std::cmp::Ordering::Less {
                        a
                    } else {
                        c
                    }
                }
            },
        }
    }
}

/// `blender::parallel_sort` — sort_unstable (serial).
pub fn parallel_sort<T: Ord>(v: &mut [T]) {
    v.sort_unstable();
}
/// `parallel_sort` with comparator.
pub fn parallel_sort_by<T, F: FnMut(&T, &T) -> std::cmp::Ordering>(v: &mut [T], cmp: F) {
    v.sort_unstable_by(cmp);
}
