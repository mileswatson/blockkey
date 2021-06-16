use super::*;

#[derive(Debug, PartialEq, Eq)]
struct MerkleNode {
    value: Hash,
    left: Option<usize>,
    right: Option<usize>,
    size: usize,
}

impl MerkleNode {
    pub fn new(x: &impl Hashable) -> MerkleNode {
        MerkleNode {
            value: x.hash(),
            left: None,
            right: None,
            size: 1,
        }
    }

    pub fn empty() -> MerkleNode {
        MerkleNode {
            value: Hash::empty(),
            left: None,
            right: None,
            size: 0,
        }
    }

    pub fn merge(tree: &[MerkleNode], left: usize, right: usize) -> MerkleNode {
        let size = tree[left].size + tree[right].size;
        MerkleNode {
            value: hash![tree[left], tree[right], size],
            left: Some(left),
            right: Some(right),
            size,
        }
    }
}

impl Hashable for MerkleNode {
    fn hash(&self) -> Hash {
        self.value
    }
}

#[derive(Debug)]
pub struct MerkleTree {
    nodes: Vec<MerkleNode>,
    root: usize,
    leaves: usize, // Number of leaves
}

impl MerkleTree {
    pub fn new<H: Hashable>(leaves: &[H]) -> MerkleTree {
        // Return empty tree if there is no leaf
        if leaves.is_empty() {
            return MerkleTree {
                nodes: vec![MerkleNode::empty()],
                root: 0,
                leaves: 0,
            };
        }
        let mut nodes: Vec<MerkleNode> = leaves.iter().map(|leaf| MerkleNode::new(leaf)).collect();
        let mut prev_layer: Vec<usize> = (0..nodes.len()).collect();
        let mut current_layer = Vec::<usize>::new();

        while prev_layer.len() != 1 {
            while !prev_layer.is_empty() {
                if prev_layer.len() > 1 {
                    let left = prev_layer.pop().unwrap();
                    let right = prev_layer.pop().unwrap();
                    nodes.push(MerkleNode::merge(&nodes, left, right));
                    current_layer.push(nodes.len() - 1);
                } else {
                    current_layer.push(prev_layer.pop().unwrap());
                }
            }
            std::mem::swap(&mut prev_layer, &mut current_layer);
        }
        MerkleTree {
            nodes,
            root: prev_layer[0],
            leaves: leaves.len(),
        }
    }
}

impl Hashable for MerkleTree {
    fn hash(&self) -> Hash {
        self.nodes[self.root].hash()
    }
}

impl PartialEq for MerkleTree {
    fn eq(&self, rhs: &Self) -> bool {
        self.hash() == rhs.hash()
    }
}

impl Eq for MerkleTree {}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() {
        MerkleTree::new::<u8>(&[1]);
        MerkleTree::new::<u8>(&[1, 2]);
        MerkleTree::new::<u8>(&[1, 2, 3]);
        MerkleTree::new::<u8>(&[1, 2, 3, 4]);
        MerkleTree::new::<u8>(&[1, 2, 3, 4, 5]);
    }
}
