use super::DVec;
use super::BRANCH_FACTOR;

#[test]
fn push_matches_len() {
    const N: usize = 5000;
    let mut pv = DVec::new();
    for i in 0..N {
        pv.push(i);
    }
    assert_eq!(pv.len(), N);

    for i in 0..N {
        assert_eq!(*pv.get(i).unwrap(), i);
    }
}

#[test]
fn push_matches_len_cloned() {
    const N: usize = 5000;
    let mut pv = DVec::new();
    for i in 0..N {
        pv.push(i);
    }
    let pv0 = pv.clone();
    assert_eq!(pv.len(), N);
    assert_eq!(pv0.len(), N);

    for i in 0..N {
        pv.push(i);
    }

    assert_eq!(pv.len(), 2 * N);
    assert_eq!(pv0.len(), N);

    for i in 0..N {
        assert_eq!(*pv.get(i).unwrap(), i);
        assert_eq!(*pv0.get(i).unwrap(), i);
    }

    for i in 0..N {
        assert_eq!(*pv.get(i + N).unwrap(), i);
    }
}

#[test]
fn push_matches_mutate_in_place() {
    const N: usize = BRANCH_FACTOR * 4;
    let mut pv = DVec::new();
    for i in 0..N {
        pv.push(i);
    }
    let pv0 = pv.clone();
    assert_eq!(pv.len(), N);
    assert_eq!(pv0.len(), N);

    for i in 0..(N / 2) {
        *pv.get_mut(i).unwrap() += 1;
    }

    assert_eq!(pv.len(), N);
    assert_eq!(pv0.len(), N);

    for i in 0..(N / 2) {
        assert_eq!(*pv.get(i).unwrap(), i + 1);
        assert_eq!(*pv0.get(i).unwrap(), i);
    }

    // the second half ought to be untouched
    for i in N / 2..N {
        assert_eq!(*pv.get(i).unwrap(), i);
        assert_eq!(*pv0.get(i).unwrap(), i);
        assert_eq!(pv.get(i).unwrap() as *const usize,
                   pv0.get(i).unwrap() as *const usize);
    }
}

macro_rules! push {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use DVec;
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                b.iter(|| {
                    let mut vec = DVec::new();
                    for i in 0 .. N {
                        vec.push(i);
                    }
                });
            }

            #[bench]
            fn standard(b: &mut test_crate::Bencher) {
                b.iter(|| {
                    let mut vec = Vec::new();
                    for i in 0 .. N {
                        vec.push(i);
                    }
                });
            }
        }
    }
}

push!(push_5000, 5000);
push!(push_50000, 50000);
push!(push_500000, 500000);

macro_rules! index_sequentially {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use DVec;
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                let mut vec = DVec::new();
                for i in 0 .. N {
                    vec.push(i * 2);
                }
                b.iter(|| {
                    for i in 0 .. N {
                        assert_eq!(vec[i], i * 2);
                    }
                });
            }

            #[bench]
            fn standard(b: &mut test_crate::Bencher) {
                let mut vec = Vec::new();
                for i in 0 .. N {
                    vec.push(i * 2);
                }
                b.iter(|| {
                    for i in 0 .. N {
                        assert_eq!(vec[i], i * 2);
                    }
                });
            }
        }
    }
}

index_sequentially!(index_sequentially_5000, 5000);
index_sequentially!(index_sequentially_50000, 50000);
index_sequentially!(index_sequentially_500000, 500000);

macro_rules! index_randomly {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use DVec;
            use rand::{Rng, SeedableRng, XorShiftRng};
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                let mut vec = DVec::new();
                for i in 0 .. N {
                    vec.push(i * 2);
                }
                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                b.iter(|| {
                    for _ in 0 .. N {
                        let j = (rng.next_u32() as usize) % N;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            }

            #[bench]
            fn standard(b: &mut test_crate::Bencher) {
                let mut vec = Vec::new();
                for i in 0 .. N {
                    vec.push(i * 2);
                }
                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                b.iter(|| {
                    for _ in 0 .. N {
                        let j = (rng.next_u32() as usize) % N;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            }
        }
    }
}

index_randomly!(index_randomly_5000, 5000);
index_randomly!(index_randomly_50000, 50000);
index_randomly!(index_randomly_500000, 500000);
