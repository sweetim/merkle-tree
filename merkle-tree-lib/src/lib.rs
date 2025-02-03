use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone, Debug)]
pub struct UserData {
    pub user_id: u32,
    pub user_balance: u32,
}

impl UserData {
    fn new(user_id: u32, user_balance: u32) -> Self {
        UserData {
            user_id,
            user_balance,
        }
    }
}

#[derive(Clone, Default)]
pub struct MerkleNode {
    hash: Vec<u8>,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    pub user_data: Option<UserData>,
}

impl MerkleNode {
    /// Creates a new leaf node with the given hash and user data.
    ///
    /// # Arguments
    ///
    /// * `hash`: The hash of the leaf node's data.
    /// * `user_data`: The user data associated with the leaf node.
    fn new_leaf(hash: Vec<u8>, user_data: Option<UserData>) -> Self {
        MerkleNode {
            hash,
            left: None,
            right: None,
            user_data,
        }
    }

    /// Creates a new branch node with the given left and right children and tag.
    /// The hash of the branch node is calculated by concatenating the hashes of its children
    /// and applying the `tagged_hash` function with the provided tag.
    ///
    /// # Arguments
    ///
    /// * `left`: The left child node.
    /// * `right`: The right child node.
    /// * `tag`: The tag used for calculating the branch node's hash.
    fn new_branch(left: MerkleNode, right: MerkleNode, tag: &str) -> Self {
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

impl fmt::Display for MerkleNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .user_data
            .as_ref()
            .map_or(hex::encode(&self.hash), |user_data| {
                format!(
                    "{} (User ID: {}, Balance: {})",
                    hex::encode(&self.hash),
                    user_data.user_id,
                    user_data.user_balance
                )
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

pub struct MerkleTree {
    root: Option<Box<MerkleNode>>,
}

struct TraverseStep<'a> {
    parent_node: Option<&'a MerkleNode>,
    current_node: &'a MerkleNode,
    level: u32,
    direction: NodeDirection,
}

impl MerkleTree {
    /// Builds a Merkle Tree from the given user data.
    ///
    /// # Arguments
    ///
    /// * `tag_leaf`: The tag used for hashing leaf nodes.
    /// * `tag_branch`: The tag used for hashing branch nodes.
    /// * `user_data`: A slice of tuples, where each tuple contains a user ID and balance.
    pub fn build(tag_leaf: &str, tag_branch: &str, user_data: &[(u32, u32)]) -> Self {
        if user_data.is_empty() {
            return MerkleTree { root: None };
        }

        let mut nodes: Vec<MerkleNode> = user_data
            .iter()
            .map(|&(user_id, user_balance)| {
                let user_data = UserData::new(user_id, user_balance);
                let serialized = format!("({},{})", user_id, user_balance);
                MerkleNode::new_leaf(
                    tagged_hash(tag_leaf, serialized.as_bytes()),
                    Some(user_data),
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
    fn iterate_tree(&self, map_fn: fn(&TraverseStep) -> String) -> Option<Vec<String>> {
        self.root.as_ref().map(|root| {
            let mut output = Vec::new();

            let mut stack: Vec<TraverseStep> = vec![TraverseStep {
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
            let current_node_label =
                (step.current_node.user_data.as_ref()).map_or(String::from(""), |item| {
                    format!(
                        "<br>User ID: {}<br>Balance: {}",
                        item.user_id, item.user_balance
                    )
                });

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
    pub fn search_with_path<F>(&self, predicate: F) -> Option<(&MerkleNode, TraversePath)>
    where
        F: Fn(&UserData) -> bool,
    {
        self.root.as_ref().and_then(|root| {
            let mut stack = vec![(root, TraversePath::new())];

            while let Some((node, path)) = stack.pop() {
                if let Some(user_data) = &node.user_data {
                    if predicate(user_data) {
                        return Some((node.as_ref(), path));
                    }
                }

                if let Some(left) = &node.left {
                    let mut left_path = path.clone();
                    left_path.add_step(hex::encode(&node.hash), NodeDirection::Left);
                    stack.push((left, left_path));
                }

                if let Some(right) = &node.right {
                    let mut right_path = path.clone();
                    right_path.add_step(hex::encode(&node.hash), NodeDirection::Right);
                    stack.push((right, right_path));
                }
            }

            None
        })
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

    #[test]
    fn it_can_build_a_tree() {
        let user_data = vec![(1, 1111), (2, 2222), (3, 3333), (4, 4444), (5, 5555)];
        let tag_leaf = "ProofOfReserve_Leaf";
        let tag_branch = "ProofOfReserve_Branch";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);

        assert_eq!(
            tree.root().unwrap(),
            "e752d40ca9a0626be5fea078ef35216a9c50554934a54dfbe2eb60195af66c85"
        );
    }

    #[test]
    fn it_can_search_with_path() {
        let user_data = vec![(1, 1111), (2, 2222), (3, 3333), (4, 4444), (5, 5555)];
        let tag_leaf = "ProofOfReserve_Leaf";
        let tag_branch = "ProofOfReserve_Branch";

        let tree = MerkleTree::build(tag_leaf, tag_branch, &user_data);
        let user_id = "3";
        let (_node, path) = tree
            .search_with_path(|user_data| user_data.user_id == user_id.parse::<u32>().unwrap())
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
