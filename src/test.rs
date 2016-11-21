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

    for i in 0..(N/2) {
        *pv.get_mut(i).unwrap() += 1;
    }

    assert_eq!(pv.len(), N);
    assert_eq!(pv0.len(), N);

    for i in 0..(N/2) {
        assert_eq!(*pv.get(i).unwrap(), i + 1);
        assert_eq!(*pv0.get(i).unwrap(), i);
    }

    // the second half ought to be untouched
    for i in N/2..N {
        assert_eq!(*pv.get(i).unwrap(), i);
        assert_eq!(*pv0.get(i).unwrap(), i);
        assert_eq!(pv.get(i).unwrap() as *const usize, pv0.get(i).unwrap() as *const usize);
    }
}
