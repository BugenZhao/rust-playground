#![allow(dead_code)]

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

use std::iter::repeat;

use bytes::Buf;
use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use tikv_jemallocator::Jemalloc;

const VNODE: &[u8] = &(233u8).to_be_bytes();
const TABLE_ID: &[u8] = b"t2333";
const EPOCH: &[u8] = &(666u64).to_be_bytes();

mod buf {
    use super::*;

    pub fn write(b: impl Buf) {
        let b = Buf::chain(VNODE, b);
        vnode_write(b)
    }

    fn vnode_write(b: impl Buf) {
        let b = Buf::chain(TABLE_ID, b);
        raw_write(b)
    }

    fn raw_write(b: impl Buf) {
        let b = Buf::chain(b, EPOCH);
        full_key_write(b)
    }

    fn full_key_write(mut b: impl Buf) {
        let bytes = b.copy_to_bytes(b.remaining());
        black_box(bytes);
    }
}

mod v {
    use super::*;

    pub fn write(b: Vec<u8>) {
        let b = [VNODE, b.as_ref()].concat();
        vnode_write(b)
    }

    fn vnode_write(b: Vec<u8>) {
        let b = [TABLE_ID, b.as_ref()].concat();
        raw_write(b)
    }

    fn raw_write(mut b: Vec<u8>) {
        b.extend_from_slice(EPOCH);
        full_key_write(b)
    }

    fn full_key_write(b: Vec<u8>) {
        black_box(b);
    }
}

type Key = Vec<u8>;

fn keys() -> Vec<Key> {
    (0..10000u64)
        .map(|i| repeat(i.to_be_bytes()).take(4).flatten().collect())
        .collect()
}

fn bench_buf_write(c: &mut Criterion) {
    let keys = keys();

    c.bench_function("buf_write", |b| {
        b.iter_batched(
            || keys.clone(),
            |keys| {
                for key in keys {
                    buf::write(key.as_ref());
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_vec_write(c: &mut Criterion) {
    let keys = keys();

    c.bench_function("vec_write", |b| {
        b.iter_batched(
            || keys.clone(),
            |keys| {
                for key in keys {
                    v::write(key);
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_buf_write, bench_vec_write);
criterion_main!(benches);
