use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use merkle_tree_lib;

fn bench_merkle_tree_lib_build(c: &mut Criterion) {
    let tag_leaf = "ProofOfReserve_Leaf";
    let tag_branch = "ProofOfReserve_Branch";

    let mut group = c.benchmark_group("merkle_tree_lib::build");

    for i in [10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(i), i, |b, &i| {
            let user_data_large = merkle_tree_lib::util::generate_random_user_data(i);

            b.iter(|| {
                std::hint::black_box({
                    merkle_tree_lib::MerkleTree::build(tag_leaf, tag_branch, &user_data_large);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_merkle_tree_lib_build);
criterion_main!(benches);
