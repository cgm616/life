use life::{Automata, LifeLike};

use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("Calculate changes");
    for size in [10usize, 100, 200, 500, 1000].iter() {
        let world_size = size * size;
        let mut buffer1: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
        let mut buffer2: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
        let mut change_buffer: BitVec<Lsb0, usize> = BitVec::with_capacity(world_size);
        buffer1.resize(world_size, false);
        buffer2.resize(world_size, false);
        buffer1.iter_mut().for_each(|i| i.set(rng.gen()));
        change_buffer.extend(
            buffer1
                .as_raw_slice()
                .iter()
                .zip(buffer2.as_raw_slice().iter())
                .map(|(&a, &b)| a ^ b),
        );
        let machine = LifeLike::new("B3/S23").unwrap();
        machine.update(&mut buffer1, &mut buffer2, &change_buffer, (*size, *size));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched_ref(
                || (buffer1.clone(), buffer2.clone(), change_buffer.clone()),
                |(buffer1, buffer2, change_buffer)| {
                    change_buffer.clear();
                    change_buffer.extend(
                        buffer2
                            .as_raw_slice()
                            .iter()
                            .zip(buffer1.as_raw_slice().iter())
                            .map(|(&a, &b)| a ^ b),
                    );
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
