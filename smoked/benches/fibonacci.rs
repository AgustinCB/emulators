use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smoked::cpu::VM;

fn fibonacci(index: usize) {
    let codes = vec![
        44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, index as u8, 0, 0, 0, 0, 0, 0, 0,
        5, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0,
        0, 0, 0, 0, 0, 25, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 14, 22, 2, 0,
        0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 3, 1, 3, 0, 0, 0, 0, 0, 0, 0, 25, 20, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0,
        0, 0, 0, 3, 1, 3, 0, 0, 0, 0, 0, 0, 0, 25, 2, 0
    ];
    let mut vm = VM::from(&codes[..]);
    while !vm.is_done() {
        vm.execute().unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fibonacci 5", |b| {
        b.iter(|| fibonacci(black_box(5)))
    });
    c.bench_function("fibonacci 10", |b| {
        b.iter(|| fibonacci(black_box(10)))
    });
    c.bench_function("fibonacci 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });
    //c.bench_function("fibonacci 40", |b| {
    //    b.iter(|| fibonacci(black_box(40)))
    //});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);