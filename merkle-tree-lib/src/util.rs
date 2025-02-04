use crate::MerkleTreeData;

#[derive(Debug, Default, Clone)]
pub struct UserData {
    pub id: u32,
    pub balance: u32,
}

impl MerkleTreeData for UserData {
    fn serialize(&self) -> Vec<u8> {
        format!("{},{}", self.id, self.balance).as_bytes().to_vec()
    }

    fn mermaid_node_label(&self) -> String {
        format!("<br>User ID: {}<br>Balance: {}", self.id, self.balance)
    }
}

pub fn generate_random_user_data(n: usize) -> Vec<UserData> {
    vec![0; n]
        .iter()
        .enumerate()
        .map(|(i, _v)| {
            let x = (i + 1) as u32;
            UserData {
                id: x,
                balance: x * 1000,
            }
        })
        .collect()
}
