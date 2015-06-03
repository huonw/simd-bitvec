#![feature(core, test, collections)]

extern crate test;
use std::simd::u32x4;

pub struct BitVec {
    storage: Vec<u32x4>,
    nbits: usize
}

const ALL_SET: u32x4 = u32x4(!0, !0, !0, !0);
const BITS: usize = 32 * 4;

fn elems(n: usize) -> usize {
    (n + BITS - 1) / BITS
}
fn elems_u32(n: usize) -> usize {
    (n + 32 - 1) / 32
}
impl BitVec {
    pub fn new() -> BitVec {
        BitVec {
            storage: vec![],
            nbits: 0
        }
    }

    pub fn from_elem(n: usize, b: bool) -> BitVec {
        let elem = ALL_SET * u32x4(b as u32, b as u32, b as u32, b as u32);

        BitVec {
            storage: (0..elems(n)).map(|_| elem).collect(),
            nbits: n
        }
    }
    pub fn set_all(&mut self) {
        for x in self.as_u32_mut() {
            *x = !0
        }
    }
    fn as_u32_mut(&mut self) -> &mut [u32] {
        unsafe {
            let len = elems_u32(self.nbits);
            std::slice::from_raw_parts_mut(self.storage.as_mut_ptr() as *mut u32,
                                           len)
        }
    }
    fn as_u32(&self) -> &[u32] {
        unsafe {
            let len = elems_u32(self.nbits);
            std::slice::from_raw_parts(self.storage.as_ptr() as *const u32,
                                       len)
        }
    }

    pub fn len(&self) -> usize {
        self.nbits
    }

    fn process<F>(&mut self, other: &BitVec, mut op: F) -> bool where F: FnMut(u32x4, u32x4) -> u32x4 {
        assert_eq!(self.len(), other.len());

        let mut changed = u32x4(0, 0, 0, 0);
        for (a, b) in self.storage.iter_mut().zip(other.storage.iter()) {
            let aa = *a;
            let w = op(aa, *b);

            changed = changed | (aa ^ w);
            *a = w;
        }
        !(changed.0 == 0 && changed.1 == 0 && changed.2 == 0 && changed.3 == 0)
    }
    pub fn union(&mut self, other: &BitVec) -> bool {
        self.process(other, |a, b| a | b)
    }
    fn process_u32<F>(&mut self, other: &BitVec, mut op: F) -> bool where F: FnMut(u32, u32) -> u32 {
        assert_eq!(self.len(), other.len());

        let mut changed = 0;
        for (a, b) in test::black_box(self.as_u32_mut()).iter_mut().zip(test::black_box(other.as_u32()).iter().cloned()) {
            let aa = *a;
            let w = op(aa, b);

            changed = changed | (aa ^ w);
            *a = w;
        }
        changed != 0
    }
    pub fn union_u32(&mut self, other: &BitVec) -> bool {
        self.process_u32(other, |a, b| a | b)
    }
}

const N: usize = 1_000_000;
macro_rules! bench {
    ($p: path, $set_all: ident, $union: ident) => {
        #[bench]
        pub fn $set_all(b: &mut test::Bencher) {
            let mut bitv = <$p>::from_elem(N, true);

            b.iter(|| {
                bitv.set_all();
                test::black_box(&bitv);
            })
        }
        #[bench]
        pub fn $union(b: &mut test::Bencher) {
            let mut bitv = <$p>::from_elem(N, true);
            let bitv2 = <$p>::from_elem(N, true);

            b.iter(|| {
                test::black_box(&bitv);
                test::black_box(&bitv2);
                test::black_box(bitv.union(&bitv2));
                test::black_box(&bitv);
                test::black_box(&bitv2);
            })
        }
    }
}

bench!(BitVec, set_all_simd, union_simd);
bench!(std::collections::BitVec, set_all_std, union_std);
#[bench]
pub fn union_u32(b: &mut test::Bencher) {
    let mut bitv = BitVec::from_elem(N, true);
    let bitv2 = BitVec::from_elem(N, true);

    b.iter(|| {
        test::black_box(&bitv);
        test::black_box(&bitv2);
        test::black_box(bitv.union_u32(&bitv2));
        test::black_box(&bitv);
        test::black_box(&bitv2);
    })
}
