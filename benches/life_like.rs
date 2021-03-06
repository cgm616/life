use life::{Automata, LifeLike};

use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("LifeLike B3/S23 5gen");
    for size in [10usize, 100, 200, 500, 1000].iter() {
        let world_size = size * size;
        let mut buffer1: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
        let mut buffer2: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
        let mut change_buffer: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
        buffer1.resize(world_size, false);
        buffer2.resize(world_size, false);
        change_buffer.resize(world_size, false);
        buffer1.iter_mut().for_each(|i| i.set(rng.gen()));

        let machine = LifeLike::new("B3/S23").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched_ref(
                || (buffer1.clone(), buffer2.clone()),
                |(buffer1, buffer2)| {
                    let (mut fresh, mut stale): (
                        &mut BitSlice<Lsb0, usize>,
                        &mut BitSlice<Lsb0, usize>,
                    ) = (buffer1, buffer2);
                    for _ in 0..black_box(5) {
                        change_buffer.clear();
                        change_buffer.extend(
                            fresh
                                .as_raw_slice()
                                .iter()
                                .zip(stale.as_raw_slice().iter())
                                .map(|(&a, &b)| a ^ b),
                        );
                        machine.update(fresh, stale, &change_buffer, (size, size));
                        std::mem::swap(&mut fresh, &mut stale);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
