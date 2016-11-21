use super::PersistentVec;

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
