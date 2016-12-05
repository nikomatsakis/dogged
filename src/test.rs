use super::PersistentVec;
use super::BRANCH_FACTOR;

#[test]
fn push_matches_len() {
    const N: usize = 5000;
    let mut pv = PersistentVec::new();
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
    let mut pv = PersistentVec::new();
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
    let mut pv = PersistentVec::new();
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
            use PersistentVec;
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                b.iter(|| {
                    let mut vec = PersistentVec::new();
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

macro_rules! sum {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use PersistentVec;
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                let mut vec = PersistentVec::new();
                for i in 0 .. N {
                    vec.push(i);
                }

                let mut sum = 0;
                b.iter(|| {
                    sum = 0;
                    for i in 0 .. N {
                        sum += *vec.get(i).unwrap();
                    }
                });

                assert_eq!(sum, (0..N).sum());
            }

            #[bench]
            fn standard(b: &mut test_crate::Bencher) {
                let mut vec = Vec::new();
                for i in 0 .. N {
                    vec.push(i);
                }

                let mut sum = 0;
                b.iter(|| {
                    sum = 0;
                    for i in 0 .. N {
                        sum += vec[i];
                    }
                });

                assert_eq!(sum, (0..N).sum());
            }
        }
    }
}

sum!(sum_5000, 5000);
sum!(sum_50000, 50000);

macro_rules! inc {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use PersistentVec;
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                let mut vec = PersistentVec::new();
                for i in 0 .. N {
                    vec.push(i);
                }

                b.iter(|| {
                    for i in 0 .. N {
                        vec.get_mut(i).unwrap().wrapping_add(1);
                    }
                });
            }

            #[bench]
            fn standard(b: &mut test_crate::Bencher) {
                let mut vec = Vec::new();
                for i in 0 .. N {
                    vec.push(i);
                }

                b.iter(|| {
                    for i in 0 .. N {
                        vec.get_mut(i).unwrap().wrapping_add(1);
                    }
                });
            }
        }
    }
}

inc!(inc_5000, 5000);
inc!(inc_50000, 50000);
