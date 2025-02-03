use criterion::{criterion_group, criterion_main, Criterion};

use merkle_tree_lib;

fn bench_merkle_tree_lib_build(c: &mut Criterion) {
    c.bench_function("merkle_tree_lib::build", |b| {
        b.iter(|| {
            std::hint::black_box({
                let user_data_large: Vec<(u32, u32)> = vec![0; 100_000]
                    .iter()
                    .enumerate()
                    .map(|(i, _v)| {
                        let x = (i + 1) as u32;
                        (x, x * 1000)
                    })
                    .collect();

                let tag_leaf = "ProofOfReserve_Leaf";
                let tag_branch = "ProofOfReserve_Branch";

                merkle_tree_lib::MerkleTree::build(tag_leaf, tag_branch, &user_data_large);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_merkle_tree_lib_build,
);
criterion_main!(benches);
