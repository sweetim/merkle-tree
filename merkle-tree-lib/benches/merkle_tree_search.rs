use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use merkle_tree_lib;

fn bench_merkle_tree_lib_search_with_path(c: &mut Criterion) {
    let tag_leaf = "ProofOfReserve_Leaf";
    let tag_branch = "ProofOfReserve_Branch";
    let user_data_large = merkle_tree_lib::util::generate_random_user_data(1_000_000);
    let tree = merkle_tree_lib::MerkleTree::build(tag_leaf, tag_branch, &user_data_large);

    let mut group = c.benchmark_group("merkle_tree_lib::search_with_path");

    for id in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(id), id, |b, &id| {
            b.iter(|| {
                std::hint::black_box(tree.search_with_path(|user_data| user_data.id == id));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_merkle_tree_lib_search_with_path);
criterion_main!(benches);
