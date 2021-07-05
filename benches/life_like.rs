use life::{Automata, LifeLike};

use bitvec::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
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

        let machine = LifeLike::new("B3/S23").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let (mut fresh, mut stale) = (&mut buffer1, &mut buffer2);
                fresh.iter_mut().for_each(|i| i.set(rng.gen()));
                for _ in 0..black_box(5) {
                    change_buffer.clear();
                    change_buffer.extend(fresh.iter().zip(stale.iter()).map(|(a, b)| *a ^ *b));
                    machine.update(fresh, stale, &change_buffer, (size, size));
                    std::mem::swap(fresh, stale);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
