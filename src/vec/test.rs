use super::*;

#[test]
fn set() {
    let mut a = DVec::with(vec![1, 2, 3]);
    let b = a.clone();
    a.set(0, 10);
    a.set(1, 20);
    a.set(2, 30);

    assert_eq!(a.depth(), 0);
    assert_eq!(b.depth(), 3);

    assert_eq!(a.get(0), 10);
    assert_eq!(a.get(1), 20);
    assert_eq!(a.get(2), 30);

    assert_eq!(a.depth(), 0);
    assert_eq!(b.depth(), 3);

    assert_eq!(b.get(0), 1);
    assert_eq!(b.get(1), 2);
    assert_eq!(b.get(2), 3);

    assert_eq!(a.depth(), 3);
    assert_eq!(b.depth(), 0);

    assert_eq!(a.get(0), 10);
    assert_eq!(a.get(1), 20);
    assert_eq!(a.get(2), 30);

    assert_eq!(a.depth(), 0);
    assert_eq!(b.depth(), 3);
}

#[test]
fn push() {
    let mut a = DVec::with(vec![1]);
    let b = a.clone();
    a.push(20);
    a.push(30);

    assert_eq!(a.get(0), 1);
    assert_eq!(a.get(1), 20);
    assert_eq!(a.get(2), 30);
    assert_eq!(a.len(), 3);

    assert_eq!(b.get(0), 1);
    assert_eq!(b.len(), 1);

    assert_eq!(a.get(0), 1);
    assert_eq!(a.get(1), 20);
    assert_eq!(a.get(2), 30);
    assert_eq!(a.len(), 3);
}
