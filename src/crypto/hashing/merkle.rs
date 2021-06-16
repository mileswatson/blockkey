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
            for i in (0..prev_layer.len()).step_by(2) {
                let left = prev_layer[i];
                match prev_layer.get(i + 1).copied() {
                    Some(right) => {
                        nodes.push(MerkleNode::merge(&nodes, left, right));
                        current_layer.push(nodes.len() - 1);
                    }
                    None => {
                        current_layer.push(left);
                    }
                }
            }
            prev_layer.clear();
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
        println!("{:?}\n", MerkleTree::new(&Vec::<u8>::new()));
        println!("{:?}\n", MerkleTree::new::<u8>(&[1]));
        println!("{:?}\n", MerkleTree::new::<u8>(&[1, 2]));
        println!("{:?}\n", MerkleTree::new::<u8>(&[1, 2, 3]));
        println!("{:?}\n", MerkleTree::new::<u8>(&[1, 2, 3, 4]));
        println!("{:?}\n", MerkleTree::new::<u8>(&[1, 2, 3, 4, 5]));
    }
}
