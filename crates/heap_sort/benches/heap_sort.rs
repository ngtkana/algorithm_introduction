use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::prelude::*;
use std::iter::repeat_with;

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("Build");
    let mut rng = StdRng::seed_from_u64(42);

    group.bench_function("Binary Heap", |b| {
        b.iter_batched_ref(
            || {
                let n = rng.gen_range(90_000, 100_000);
                let binary_heap = repeat_with(|| rng.gen_range(0, std::u32::MAX))
                    .take(n)
                    .collect::<Vec<u32>>();
                binary_heap
            },
            |a| heap_sort::BinaryHeap::build(a),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("Tertiary Heap", |b| {
        b.iter_batched_ref(
            || {
                let n = rng.gen_range(90_000, 100_000);
                let a = repeat_with(|| rng.gen_range(0, std::u32::MAX))
                    .take(n)
                    .collect::<Vec<u32>>();
                a
            },
            |a| heap_sort::DArrayHeap::build(a, 3),
            BatchSize::SmallInput,
        )
    });
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("Insersion");
    let mut rng = StdRng::seed_from_u64(42);

    group.bench_function("Binary Heap", |b| {
        let n = rng.gen_range(90_000, 100_000);
        let a = repeat_with(|| rng.gen_range(0, std::u32::MAX))
            .take(n)
            .collect::<Vec<u32>>();
        let mut binary_heap = heap_sort::BinaryHeap::build(&a);
        b.iter_batched_ref(
            || {
                let x = rng.gen_range(0, std::u32::MAX);
                x
            },
            |x| binary_heap.insert(*x),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("Tertiary Heap", |b| {
        let n = rng.gen_range(90_000, 100_000);
        let a = repeat_with(|| rng.gen_range(0, std::u32::MAX))
            .take(n)
            .collect::<Vec<u32>>();
        let mut tertiary_heap = heap_sort::DArrayHeap::build(&a, 3);

        b.iter_batched_ref(
            || {
                let x = rng.gen_range(0, std::u32::MAX);
                x
            },
            |x| tertiary_heap.insert(*x),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_build, bench_insert);
criterion_main!(benches);
