use merkle_tree_lib;
use rocket::serde::{json::Json, Serialize};
use rocket::State;

#[macro_use]
extern crate rocket;

#[get("/proof")]
fn proof_all_users(state: &State<AppState>) -> String {
    state.tree.root().unwrap()
}

#[get("/proof/mermaid")]
fn proof_all_users_display_mermaid_diagram(state: &State<AppState>) -> String {
    state.tree.display_mermaid_diagram()
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct MerkleProof {
    user_balance: u32,
    proof: Vec<(String, u8)>,
}

#[get("/proof/<user_id>")]
fn proof_by_user_id(state: &State<AppState>, user_id: &str) -> Json<MerkleProof> {
    let (node, path) = state
        .tree
        .search_with_path(|user_data| user_data.user_id == user_id.parse::<u32>().unwrap())
        .unwrap();

    Json(MerkleProof {
        user_balance: node.user_data.as_ref().unwrap().user_balance,
        proof: path.to_vec(),
    })
}

struct AppState {
    tree: merkle_tree_lib::MerkleTree,
}

#[launch]
fn rocket() -> _ {
    let user_data = vec![
        (1, 1111),
        (2, 2222),
        (3, 3333),
        (4, 4444),
        (5, 5555),
        (6, 6666),
        (7, 7777),
        (8, 8888),
    ];

    let tag_leaf = "ProofOfReserve_Leaf";
    let tag_branch = "ProofOfReserve_Branch";

    let tree = merkle_tree_lib::MerkleTree::build(tag_leaf, tag_branch, &user_data);

    rocket::build().manage(AppState { tree }).mount(
        "/",
        routes![
            proof_all_users,
            proof_all_users_display_mermaid_diagram,
            proof_by_user_id
        ],
    )
}
