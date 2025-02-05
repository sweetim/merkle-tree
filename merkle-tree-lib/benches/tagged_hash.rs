use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use merkle_tree_lib;

fn bench_tagged_hash(c: &mut Criterion) {
    let tag_leaf = "ProofOfReserve_Leaf";

    let mut group = c.benchmark_group("merkle_tree_lib::tagged_hash");

    for max_range in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(max_range), max_range, |b, &max_range| {
            b.iter(|| {
                std::hint::black_box({
                    for _ in 0..max_range {
                        merkle_tree_lib::tagged_hash(&tag_leaf, "aaa".as_bytes());
                    }
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_tagged_hash);
criterion_main!(benches);
