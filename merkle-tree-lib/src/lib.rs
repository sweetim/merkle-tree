use sha2::{Digest, Sha256};
use std::fmt;

pub mod util;

#[derive(Clone, Default)]
pub struct MerkleNode<T> {
    hash: Vec<u8>,
    left: Option<Box<MerkleNode<T>>>,
    right: Option<Box<MerkleNode<T>>>,
    pub user_data: Option<T>,
}

impl<T> MerkleNode<T>
where
    T: Clone + fmt::Debug,
{
    /// Creates a new leaf node with the given hash and user data.
    ///
    /// # Arguments
    ///
    /// * `hash`: The hash of the leaf node's data.
    /// * `user_data`: The user data associated with the leaf node.
    fn new_leaf(hash: Vec<u8>, user_data: Option<T>) -> Self {
        MerkleNode {
            hash,
            left: None,
            right: None,
            user_data,
        }
    }

    /// Creates a new branch node with the given left and right children and tag.
    /// The hash of the branch node is calculated by concatenating the hashes of its children
    /// and applying the `tagged_hash` function witsh the provided tag.
    ///
    /// # Arguments
    ///
    /// * `left`: The left child node.
    /// * `right`: The right child node.
    /// * `tag`: The tag used for calculating the branch node's hash.
    fn new_branch(left: MerkleNode<T>, right: MerkleNode<T>, tag: &str) -> Self {
        let combined = vec![left.hash.clone(), right.hash.clone()].concat();
        let hash = tagged_hash(tag, &combined);
        MerkleNode {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            user_data: None,
        }
    }
}

impl<T> fmt::Display for MerkleNode<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .user_data
            .as_ref()
            .map_or(hex::encode(&self.hash), |user_data| {
                format!("{} ({})", hex::encode(&self.hash), user_data)
            });

        write!(f, "{}", formatted)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeDirection {
    Left,
    Right,
    Root,
}

impl NodeDirection {
    fn value(&self) -> u8 {
        match self {
            NodeDirection::Left => 0,
            NodeDirection::Right => 1,
            NodeDirection::Root => 2,
        }
    }
}

impl fmt::Display for NodeDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeDirection::Left => write!(f, "Left"),
            NodeDirection::Right => write!(f, "Right"),
            NodeDirection::Root => write!(f, "Root"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraversePath {
    pub hashes: Vec<String>,
    pub directions: Vec<NodeDirection>,
}

impl TraversePath {
    fn new() -> Self {
        TraversePath {
            hashes: Vec::new(),
            directions: Vec::new(),
        }
    }

    /// Adds a step to the `TraversePath`.
    ///
    /// # Arguments
    ///
    /// * `hash`: The hash of the node visited in this step.
    /// * `direction`: The direction taken to reach the node (Left or Right).
    fn add_step(&mut self, hash: String, direction: NodeDirection) {
        self.hashes.push(hash);
        self.directions.push(direction);
    }

    /// Converts the `TraversePath` to a vector of (hash, direction) tuples.
    /// The direction is represented as a `u8` (0 for Left, 1 for Right, 2 for Root).
    ///
    /// # Returns
    ///
    /// A `Vec<(String, u8)>` representing the path.
    pub fn to_vec(&self) -> Vec<(String, u8)> {
        self.hashes
            .iter()
            .zip(self.directions.iter())
            .map(|(hash, direction)| (hash.to_string(), direction.value()))
            .collect()
    }
}

pub struct MerkleTree<T> {
    root: Option<Box<MerkleNode<T>>>,
}

struct TraverseStep<'a, T> {
    parent_node: Option<&'a MerkleNode<T>>,
    current_node: &'a MerkleNode<T>,
    level: u32,
    direction: NodeDirection,
}

pub trait MerkleTreeData {
    fn serialize(&self) -> Vec<u8>;
    fn mermaid_node_label(&self) -> String;
}

impl<T> MerkleTree<T>
where
    T: Clone + fmt::Debug + MerkleTreeData + Default,
{
    /// Builds a Merkle Tree from the given user data.
    ///
    /// # Arguments
    ///
    /// * `tag_leaf`: The tag used for hashing leaf nodes.
    /// * `tag_branch`: The tag used for hashing branch nodes.
    /// * `user_data`: A slice of tuples, where each tuple contains a user ID and balance.
    pub fn build(tag_leaf: &str, tag_branch: &str, input: &Vec<T>) -> Self {
        if input.is_empty() {
            return MerkleTree { root: None };
        }

        let mut nodes: Vec<MerkleNode<T>> = input
            .iter()
            .map(|data| {
                MerkleNode::new_leaf(
                    tagged_hash(tag_leaf, data.serialize().as_slice()),
                    Some(data.clone()),
                )
            })
            .collect();

        while nodes.len() > 1 {
            nodes = nodes
                .chunks_mut(2)
                .map(|pair| {
                    let [left, right] = match pair {
                        [l, r] => [std::mem::take(l), std::mem::take(r)],
                        [l] => [l.clone(), std::mem::take(l)],
                        _ => panic!(),
                    };

                    MerkleNode::new_branch(left, right, tag_branch)
                })
                .collect();
        }

        MerkleTree {
            root: Some(Box::new(nodes[0].clone())),
        }
    }

    /// Returns the hash of the root node of the Merkle Tree.
    pub fn root(&self) -> Option<String> {
        self.root.as_ref().map(|node| hex::encode(&node.hash))
    }

    /// Iterates over the tree level by level and applies the given function to each node.
    ///
    /// # Arguments
    ///
    /// * `map_fn`: A function that takes a `&TraverseStep` and returns a String.
    ///              This function is called for each node in the tree.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `Vec<String>` if the tree is not empty, `None` otherwise.
    /// Each string in the vector is the result of applying `map_fn` to a node.
    fn iterate_tree(&self, map_fn: fn(&TraverseStep<T>) -> String) -> Option<Vec<String>> {
        self.root.as_ref().map(|root| {
            let mut output = Vec::new();

            let mut stack: Vec<TraverseStep<T>> = vec![TraverseStep {
                parent_node: None,
                current_node: root,
                level: 0,
                direction: NodeDirection::Root,
            }];

            while let Some(step) = stack.pop() {
                output.push(map_fn(&step));

                if let Some(right) = &step.current_node.right {
                    stack.push(TraverseStep {
                        parent_node: Some(step.current_node),
                        current_node: right,
                        level: step.level + 1,
                        direction: NodeDirection::Right,
                    });
                }

                if let Some(left) = &step.current_node.left {
                    stack.push(TraverseStep {
                        parent_node: Some(step.current_node),
                        current_node: left,
                        level: step.level + 1,
                        direction: NodeDirection::Left,
                    });
                }
            }

            output
        })
    }

    /// Displays the Merkle Tree in an indented format.
    pub fn display_tree(&self) -> String {
        match self.iterate_tree(|step| {
            let indent = " ".repeat(step.level as usize);
            format!(
                "{}{}: {}",
                indent,
                step.direction,
                truncate_middle(hex::encode(&step.current_node.hash).as_str(), 10)
            )
        }) {
            Some(output) => output.join("\n"),
            None => format!("Tree is empty."),
        }
    }

    /// Displays the Merkle Tree as a Mermaid diagram.
    /// Use the mermaid editor to visualize the diagram https://mermaid.live/
    pub fn display_mermaid_diagram(&self) -> String {
        match self.iterate_tree(|step| {
            let current_node_hash = hex::encode(&step.current_node.hash);
            let truncated_current_node_hash = truncate_middle(current_node_hash.as_str(), 10);
            let current_node_label = (step.current_node.user_data.as_ref())
                .map_or(String::from(""), |item| item.mermaid_node_label());
            println!("{current_node_label} lable");
            let node_mermaid = format!(
                "Node_{current_node_hash}[{truncated_current_node_hash}{current_node_label}]",
            );

            let node_connection_mermaid = (step.direction != NodeDirection::Root)
                .then(|| {
                    let parent_node_hash = hex::encode(&step.parent_node.unwrap().hash);

                    format!("\nNode_{} --> Node_{}", parent_node_hash, current_node_hash)
                })
                .unwrap_or_default();

            format!("{node_mermaid}{node_connection_mermaid}")
        }) {
            Some(output) => format!("flowchart TD\n{}", output.join("\n")),
            None => format!("Tree is empty."),
        }
    }

    /// Searches for a user with the given predicate.
    ///
    /// # Arguments
    ///
    /// * `predicate`: A function that takes a `&UserData` and returns a boolean.
    ///              It returns true if the user data matches the search criteria, false otherwise.
    ///
    /// # Returns
    ///
    /// An `Option` containing a tuple of `(&MerkleNode, TraversePath)` if a matching user is found, `None` otherwise.
    pub fn search_with_path<F>(&self, predicate: F) -> Option<(&MerkleNode<T>, TraversePath)>
    where
        F: Fn(&T) -> bool,
    {
        if let Some(root) = &self.root {
            let mut path = TraversePath::new();
            Self::search_node_with_path(root, &predicate, &mut path)
        } else {
            None
        }
    }

    fn search_node_with_path<'a, F>(
        node: &'a MerkleNode<T>,
        predicate: &F,
        path: &mut TraversePath,
    ) -> Option<(&'a MerkleNode<T>, TraversePath)>
    where
        F: Fn(&T) -> bool,
    {
        if let Some(user_data) = &node.user_data {
            if predicate(user_data) {
                return Some((
                    node,
                    TraversePath {
                        directions: path.directions.clone(),
                        hashes: path.hashes.clone(),
                    },
                ));
            }
        }

        if let Some(left) = &node.left {
            path.add_step(hex::encode(&node.hash), NodeDirection::Left);
            if let Some(result) = Self::search_node_with_path(left, predicate, path) {
                return Some(result);
            }
            path.hashes.pop();
            path.directions.pop();
        }

        if let Some(right) = &node.right {
            path.add_step(hex::encode(&node.hash), NodeDirection::Right);
            if let Some(result) = Self::search_node_with_path(right, predicate, path) {
                return Some(result);
            }
            path.hashes.pop();
            path.directions.pop();
        }

        None
    }
}

/// Truncates a string in the middle if it exceeds the maximum length.
///
/// If the input string's length is less than or equal to `max_len`, it returns the original string.
/// Otherwise, it returns a new string with the first `max_len / 2` characters, an ellipsis ("..."),
/// and the last `max_len - (max_len / 2)` characters.
///
/// # Arguments
///
/// * `input`: The string to truncate.
/// * `max_len`: The maximum length of the string.
///
/// # Returns
///
/// A string of truncated text.
fn truncate_middle(input: &str, max_len: usize) -> String {
    let len = input.len();
    if len <= max_len {
        return input.to_string();
    }

    let half_len = max_len / 2;
    let start = &input[..half_len];
    let end = &input[len - (max_len - half_len)..];

    format!("{}...{}", start, end)
}

/// Calculates a tagged hash using SHA256.
///
/// This function takes a tag and an input byte slice, calculates the SHA256 hash of the tag,
/// then calculates the SHA256 hash of the concatenation of the tag's hash (twice) and the input.
///
/// # Arguments
///
/// * `tag`: The tag string.
/// * `input`: The input byte slice.
///
/// # Returns
///
/// The tagged SHA256 hash as a `Vec<u8>`.
fn tagged_hash(tag: &str, input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    let tag_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&tag_hash);
    hasher.update(&tag_hash);
    hasher.update(input);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("abcdefghijklmnopqrstuvwxyz", 10, "abcde...vwxyz")]
    #[case("abcdefghijklmnopqrstuvwxyz", 5, "ab...xyz")]
    #[case("abcdefghijklmnopqrstuvwxyz", 2, "a...z")]
    #[case("abcdefghijklmnopqrstuvwxyz", 1, "...z")]
    fn it_can_truncate_middle(#[case] input: &str, #[case] max_len: usize, #[case] expected: &str) {
        let actual = super::truncate_middle(input, max_len);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(
        "Bitcoin_Transaction",
        "aaa",
        "d2d838724571ff750eb7f498a667c32f522efae2b403eae6f678207ac6f978de"
    )]
    #[case(
        "Bitcoin_Transaction",
        "bbb",
        "7cdf701413062eaba020af83441a6762ee2910e36b1805bad072103b0257f441"
    )]
    #[case(
        "hello",
        "aaa",
        "aa7deacc6231c611d10b4a2b14bec43c30251b977610fd5a322550003f2b216b"
    )]
    fn tagged_hash(#[case] tag: &str, #[case] input: &str, #[case] expected: &str) {
        let actual = super::tagged_hash(tag, input.as_bytes());
        assert_eq!(hex::encode(actual), expected);
    }

    #[derive(Clone, Debug, Default)]
    #[allow(non_camel_case_types)]
    pub struct UserItem_A {
        value: String,
    }

    impl MerkleTreeData for UserItem_A {
        fn serialize(&self) -> Vec<u8> {
            format!("{}", self.value).as_bytes().to_vec()
        }

        fn mermaid_node_label(&self) -> String {
            format!("<br>{}", self.value)
        }
    }

    fn generate_user_item_a() -> Vec<UserItem_A> {
        vec!["aaa", "bbb", "ccc", "ddd", "eee"]
            .into_iter()
            .map(|v| UserItem_A {
                value: String::from(v),
            })
            .collect()
    }

    #[derive(Clone, Debug, Default)]
    #[allow(non_camel_case_types)]
    pub struct UserItem_B {
        pub id: u32,
        pub balance: u32,
    }

    impl MerkleTreeData for UserItem_B {
        fn serialize(&self) -> Vec<u8> {
            format!("({},{})", self.id, self.balance)
                .as_bytes()
                .to_vec()
        }

        fn mermaid_node_label(&self) -> String {
            format!("<br>User ID: {}<br>Balance: {}", self.id, self.balance)
        }
    }

    fn generate_user_item_b() -> Vec<UserItem_B> {
        vec![(1, 1111), (2, 2222), (3, 3333), (4, 4444), (5, 5555)]
            .into_iter()
            .map(|(id, balance)| UserItem_B { id, balance })
            .collect()
    }

    #[test]
    fn it_can_build_a_tree_with_empty_input() {
        let input: Vec<UserItem_A> = vec![];

        let tag_leaf = "Bitcoin_Transaction";
        let tag_branch = "Bitcoin_Transaction";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &input);

        assert!(tree.root().is_none());
    }

    #[test]
    fn it_can_build_a_tree_user_item_a() {
        let user_data = generate_user_item_a();

        let tag_leaf = "Bitcoin_Transaction";
        let tag_branch = "Bitcoin_Transaction";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);

        assert_eq!(
            tree.root().unwrap(),
            "4aa906745f72053498ecc74f79813370a4fe04f85e09421df2d5ef760dfa94b5"
        );
    }

    #[test]
    fn it_can_build_a_tree_user_item_b() {
        let user_data = generate_user_item_b();

        let tag_leaf = "ProofOfReserve_Leaf";
        let tag_branch = "ProofOfReserve_Branch";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);

        assert_eq!(
            tree.root().unwrap(),
            "e752d40ca9a0626be5fea078ef35216a9c50554934a54dfbe2eb60195af66c85"
        );
    }

    #[test]
    fn it_can_search_with_path_user_item_a() {
        let user_data = generate_user_item_a();

        let tag_leaf = "Bitcoin_Transaction";
        let tag_branch = "Bitcoin_Transaction";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);
        let user_id = "aaa";
        let (_node, path) = tree
            .search_with_path(|user_data| user_data.value == user_id)
            .unwrap();

        assert_eq!(
            path.to_vec(),
            vec![
                (
                    "4aa906745f72053498ecc74f79813370a4fe04f85e09421df2d5ef760dfa94b5".to_string(),
                    0u8
                ),
                (
                    "718b18c132f71dad76a3977a587e40c876bab3436b0f9a0446dbfadca2c13ea3".to_string(),
                    0u8
                ),
                (
                    "631bae42ba587408a741fa7d482a955d059caa471c5d66548d44a6ed234e782c".to_string(),
                    0u8
                )
            ]
        );
    }

    #[test]
    fn it_can_search_with_path_user_item_b() {
        let user_data = generate_user_item_b();

        let tag_leaf = "ProofOfReserve_Leaf";
        let tag_branch = "ProofOfReserve_Branch";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);
        let user_id = 3u32;
        let (_node, path) = tree
            .search_with_path(|user_data| user_data.id == user_id)
            .unwrap();

        assert_eq!(
            path.to_vec(),
            vec![
                (
                    "e752d40ca9a0626be5fea078ef35216a9c50554934a54dfbe2eb60195af66c85".to_string(),
                    0u8
                ),
                (
                    "fafe4ecc00e37d340d72f581fbbda4e179ad24bdc2f45713dcc2a38ebfc30439".to_string(),
                    1u8
                ),
                (
                    "d185af244042b0fecba7ee16c9933d73b10c5482104538274dd777b6b120eae1".to_string(),
                    0u8
                )
            ]
        );
    }
}
