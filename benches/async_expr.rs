use std::rc::Rc;
use std::sync::Arc;

use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use futures::{FutureExt, Stream, StreamExt};
use itertools::{repeat_n, Itertools};
use tokio::runtime::Runtime;

type Array = Rc<Vec<i32>>;

fn add(a: Array, b: Array) -> Array {
    a.iter()
        .zip(b.iter())
        .map(|(&a, &b)| a + b)
        .collect_vec()
        .into()
}

async fn add_async(a: Array, b: Array) -> Array {
    add(a, b)
}

fn chunks() -> impl Stream<Item = Array> {
    futures::stream::iter(repeat_n(Rc::new(vec![233; 20480]), 128))
}

fn runtime() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

fn bench_sync(c: &mut Criterion) {
    c.bench_function("sync", |b| {
        b.to_async(runtime()).iter_batched(
            chunks,
            |mut chunks| async move {
                while let Some(chunk) = chunks.next().await {
                    black_box(add(chunk.clone(), chunk));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_async(c: &mut Criterion) {
    c.bench_function("async", |b| {
        b.to_async(runtime()).iter_batched(
            chunks,
            |mut chunks| async move {
                while let Some(chunk) = chunks.next().await {
                    black_box(add_async(chunk.clone(), chunk).await);
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_async_buffered(c: &mut Criterion) {
    c.bench_function("async_buffered", |b| {
        b.to_async(runtime()).iter_batched(
            chunks,
            |chunks| async move {
                let mut stream = chunks
                    .map(|chunk| add_async(chunk.clone(), chunk))
                    .buffered(8);

                while let Some(chunk) = stream.next().await {
                    black_box(chunk);
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_sync, bench_async, bench_async_buffered);
criterion_main!(benches);
