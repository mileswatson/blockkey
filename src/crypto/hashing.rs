
use data_encoding::HEXUPPER;
use sha2::{Sha256, Digest};
use std::convert::TryInto;
use std::ops;
use std::fmt;

// Definition of Hash struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    // Method that takes a byte array and return hash
    pub fn from_bytes(msg: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(msg);
        let result = hasher.finalize();
        Hash(result.as_slice().try_into().unwrap())
    }

    // Method that takes two Hash and return the Hash of their concatenation
    pub fn concat(lhs: &Hash, rhs: &Hash) -> Hash {
        let mut result = Vec::new();
        result.extend_from_slice(&lhs.0);
        result.extend_from_slice(&rhs.0);
        let result_slice: &[u8] = &result;
        Hash::from_bytes(&result_slice)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HEXUPPER.encode(&self.0))
    }
}

// Basically Hash::concat
impl ops::Add for Hash {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Hash::concat(&self, &rhs)
    }    
}

// End of Hash 

// Definition of Hashable
pub trait Hashable {
    fn hash(&self) -> Hash;
}
// End of hashable

// Definition of Merkle Tree Node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleNode {
    pub value: Hash,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub size: usize,
}

impl MerkleNode {
    pub fn merge(tree: &Vec::<MerkleNode>, left: usize, right: usize) -> MerkleNode {
        let size = tree[left].size + tree[right].size;
        let value = tree[left].value + tree[right].value + size.hash();
        MerkleNode {
            value,
            left: Some(left),
            right: Some(right),
            size,
        }
    }
}
// End of Merkle Tree Node


// Definition of Merkle Tree
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub tree: Vec::<MerkleNode>,
    pub root: Option<usize>,
    pub size: usize, // Number of leaves
}

impl MerkleTree {
    pub fn new<H: Hashable>(leaves: &[H]) -> MerkleTree {
        // Return empty tree if there is no leave
        if leaves.is_empty() {
            return MerkleTree {
                tree: Vec::<MerkleNode>::new(),
                root: None,
                size: 0,
            };
        }

        let mut tree = Vec::<MerkleNode>::new();
    
        let mut prev_layer = Vec::<usize>::new();
        let mut current_layer = Vec::<usize>::new();
    
        for (index, leaf_value) in leaves.iter().enumerate() {
            tree.push(MerkleNode {
                value: leaf_value.hash(),
                left: None,
                right: None,
                size: 1,
            });
            prev_layer.push(index as usize);
        }
    
        while prev_layer.len() != 1 {
            while !prev_layer.is_empty() {
                if prev_layer.len() > 1 {
                    let left = prev_layer.pop().unwrap();
                    let right = prev_layer.pop().unwrap();
                    tree.push(MerkleNode::merge(&tree, left, right));
                    current_layer.push(tree.len()-1);
                } else {
                    current_layer.push(prev_layer.pop().unwrap());
                }
            }
            std::mem::swap(&mut prev_layer, &mut current_layer);
            current_layer.clear();
        }
        
        MerkleTree {
            tree, 
            root: Some(prev_layer[0]),
            size: leaves.len(),
        }
    }

    pub fn get_root_hash(&self) -> Option<Hash> {
        match self.root {
            Some(index) => Some(self.tree[index].value.clone()),
            None => None
        }
    }
}

impl PartialEq for MerkleTree {
    fn eq(&self, rhs: &Self) -> bool {
        self.get_root_hash() == rhs.get_root_hash() &&
        self.size == rhs.size
    }
}

impl Eq for MerkleTree {}
// End of merkle tree

// impl Hashable trait for integers, floats, boolean, char, and string
impl Hashable for i8 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i16 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i32 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i64 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i128 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for isize {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u8 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u16 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u32 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u64 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u128 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for usize {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for f32 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
} 

impl Hashable for f64 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for char {
    fn hash(&self) -> Hash {
        let mut bytes = [0; 2];
        self.encode_utf8(&mut bytes);
        Hash::from_bytes(&bytes)
    }
}

// There is no to_be_bytes for bool, so it's kinda hacky
impl Hashable for bool {
    fn hash(&self) -> Hash {
        (*self as i32).hash()
    }
}

impl Hashable for String  {
    fn hash(&self) -> Hash {
        Hash::from_bytes(self.as_bytes())
    }
}

