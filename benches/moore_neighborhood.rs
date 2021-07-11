use life::moore_neighborhood_wrapping;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SIZE: (usize, usize) = (20, 20);

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Calculate Moore neighborhood wrapping", |b| {
        b.iter(|| {
            for i in 0..SIZE.0 {
                for j in 0..SIZE.1 {
                    moore_neighborhood_wrapping(black_box((i, j)), black_box(SIZE));
                }
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
