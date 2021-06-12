
use data_encoding::HEXUPPER;
use sha2::{Sha256, Digest};
use std::convert::TryInto;

#[derive(Debug, Clone, Copy)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn from_bytes(msg: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(msg);
        let result = hasher.finalize();
        Hash(result.as_slice().try_into().unwrap())
    }

    pub fn to_string(&self) -> String {
        return HEXUPPER.encode(&self.0)
    }

    pub fn concat(lhs: &Hash, rhs: &Hash) -> Hash {
        let mut result = Vec::new();
        result.extend_from_slice(&lhs.0);
        result.extend_from_slice(&rhs.0);
        let result_slice: &[u8] = &result;
        Hash::from_bytes(&result_slice)
    }
}

pub trait Hashable {
    fn hash(&self) -> Hash;
}

#[derive(Debug, Clone)]
pub struct MerkleNode {
    value: Hash,
    left: MerkleTree,
    right: MerkleTree,
}

pub type MerkleTree = Option<Box<MerkleNode>>;

pub fn build_merkle_tree<H: Hashable>(leaves: &[H]) -> MerkleTree {
    let mut prev_layer = Vec::<MerkleNode>::new();
    let mut current_layer = Vec::<MerkleNode>::new();

    for leaf_value in leaves.iter() {
        let leaf_node = MerkleNode {
            value: leaf_value.hash(),
            left: None,
            right: None,
        };
        prev_layer.push(leaf_node);
    }

    while prev_layer.len() != 1 {
        while !prev_layer.is_empty() {
            if prev_layer.len() > 1 {
                let left = prev_layer.pop().unwrap();
                let right = prev_layer.pop().unwrap();
                let value = Hash::concat(&left.value, &right.value);
                current_layer.push(MerkleNode {
                    value,
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                });
            } else {
                current_layer.push(prev_layer.pop().unwrap());
            }
        }
        prev_layer = current_layer.clone();
        current_layer.clear();
    }
    
    Some(Box::new(prev_layer[0].clone()))
}