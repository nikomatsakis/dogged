extern crate test;
use self::test::Bencher;
use super::*;

const SCALE: usize = 1000;

#[bench]
fn unaliased_push(bencher: &mut Bencher) {
    bencher.iter(|| {
        let mut a = DVec::new();
        for i in 0..SCALE {
            a.push(i);
        }
    });
}

#[bench]
fn aliased_push(bencher: &mut Bencher) {
    bencher.iter(|| {
        let mut a = DVec::new();
        let b = a.clone();
        for i in 0..SCALE {
            a.push(i);
        }
    });
}

#[bench]
fn vec_push(bencher: &mut Bencher) {
    bencher.iter(|| {
        let mut a = Vec::new();
        for i in 0..SCALE {
            a.push(i);
        }
    });
}

#[bench]
fn vec_push2(bencher: &mut Bencher) {
    bencher.iter(|| {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in 0..SCALE {
            a.push(i);
            b.push(i);
        }
    });
}
