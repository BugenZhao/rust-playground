use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_empty(c: &mut Criterion) {
    let empty = std::iter::empty::<i32>();

    c.bench_function("empty", |b| {
        b.iter_batched(
            || empty.clone(),
            |empty| {
                black_box(empty.count());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_empty_array(c: &mut Criterion) {
    let empty = <[i32; 0]>::into_iter([]);

    c.bench_function("empty_array", |b| {
        b.iter_batched(
            || empty.clone(),
            |empty| {
                black_box(empty.count());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_once(c: &mut Criterion) {
    let once = std::iter::once(1);

    c.bench_function("once", |b| {
        b.iter_batched(
            || once.clone(),
            |once| {
                black_box(once.count());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_once_array(c: &mut Criterion) {
    let once = <[i32; 1]>::into_iter([1]);

    c.bench_function("once_array", |b| {
        b.iter_batched(
            || once.clone(),
            |once| {
                black_box(once.count());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_empty,
    bench_empty_array,
    bench_once,
    bench_once_array
);
criterion_main!(benches);
