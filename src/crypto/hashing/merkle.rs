use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleNode {
    pub value: Hash,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub size: usize,
}

impl MerkleNode {
    pub fn merge(tree: &[MerkleNode], left: usize, right: usize) -> MerkleNode {
        let size = tree[left].size + tree[right].size;
        let value = hash![tree[left].value, tree[right].value, size.hash()];
        MerkleNode {
            value,
            left: Some(left),
            right: Some(right),
            size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub tree: Vec<MerkleNode>,
    pub root: Option<usize>,
    pub size: usize, // Number of leaves
}

impl MerkleTree {
    pub fn new<H: Hashable>(leaves: &[H]) -> MerkleTree {
        // Return empty tree if there is no leaf
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
                    current_layer.push(tree.len() - 1);
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
        self.root.map(|index| self.tree[index].value)
    }
}

impl PartialEq for MerkleTree {
    fn eq(&self, rhs: &Self) -> bool {
        self.get_root_hash() == rhs.get_root_hash() && self.size == rhs.size
    }
}

impl Eq for MerkleTree {}
