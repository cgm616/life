use life::{ConwaysLife, LifeLike};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let machine = LifeLike::new("B3/S23").unwrap();

    let mut group = c.benchmark_group("Logic vs Lookup (B3/S23)");

    group.bench_function("Logic", |b| {
        b.iter(|| {
            for i in 0..=8usize {
                ConwaysLife::simulate_with_logic(true, black_box(i));
                ConwaysLife::simulate_with_logic(false, black_box(i));
            }
        })
    });
    group.bench_function("Lookup", |b| {
        b.iter(|| {
            for i in 0..=8usize {
                machine.simulate(true, black_box(i));
                machine.simulate(false, black_box(i));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
