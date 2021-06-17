use super::*;

#[derive(Debug)]
struct MerkleNode {
    value: Hash,
    children: Option<(usize, usize)>,
    size: usize,
}

impl MerkleNode {
    pub fn new(x: &impl Hashable) -> MerkleNode {
        MerkleNode {
            value: x.hash(),
            children: None,
            size: 1,
        }
    }

    pub fn empty() -> MerkleNode {
        MerkleNode {
            value: Hash::empty(),
            children: None,
            size: 0,
        }
    }

    pub fn merge(tree: &[MerkleNode], left: usize, right: usize) -> MerkleNode {
        let size = tree[left].size + tree[right].size;
        MerkleNode {
            value: hash![tree[left], tree[right]],
            children: Some((left, right)),
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
    size: usize, // Number of leaves
}

impl MerkleTree {
    pub fn new<H: Hashable>(leaves: &[H]) -> MerkleTree {
        // Return empty tree if there is no leaf
        if leaves.is_empty() {
            return MerkleTree {
                nodes: vec![MerkleNode::empty()],
                root: 0,
                size: 0,
            };
        }
        // Create a MerkleNode for each leaf
        let mut nodes: Vec<MerkleNode> = leaves.iter().map(|leaf| MerkleNode::new(leaf)).collect();

        // Reference the indices of the leaves
        let mut current_layer: Vec<usize> = (0..nodes.len()).collect();

        // For keeping track of the previous layer
        let mut prev_layer = Vec::<usize>::new();

        while current_layer.len() != 1 {
            // prev_layer = current_layer, but reduces allocations
            std::mem::swap(&mut prev_layer, &mut current_layer);
            current_layer.clear();

            // Iterate through and merge all pairs from the previous layer
            for i in (0..prev_layer.len() - 1).step_by(2) {
                let left = prev_layer[i];
                let right = prev_layer[i + 1];

                nodes.push(MerkleNode::merge(&nodes, left, right));
                current_layer.push(nodes.len() - 1);
            }
            // If there's one node left over, add it to the end
            if prev_layer.len() % 2 == 1 {
                current_layer.push(prev_layer[prev_layer.len() - 1]);
            }
        }
        MerkleTree {
            nodes,
            root: current_layer[0],
            size: leaves.len(),
        }
    }

    /// Generate a proof that the given item is contained within the Merkle tree.
    pub fn construct_proof(&self, index: usize) -> Vec<Hash> {
        if index >= self.size {
            panic!()
        } else {
            let mut proof = vec![];
            let mut relative_index = index;
            let mut current = &self.nodes[self.root];
            while current.size > 1 {
                let children = current
                    .children
                    .map(|(left, right)| (&self.nodes[left], &self.nodes[right]))
                    .unwrap();
                if relative_index < children.0.size {
                    proof.push(children.1.value);
                    current = children.0;
                } else {
                    relative_index -= children.0.size;
                    proof.push(children.0.value);
                    current = children.1;
                }
            }
            proof
        }
    }

    /// Verify a proof that a given Merkle tree contains the given leaf node at the given index
    pub fn verify_proof<T: Hashable>(
        index: usize,
        size: usize,
        leaf: T,
        tree_hash: Hash,
        proof: &[Hash],
    ) -> bool {
        verify_proof_rec(index, size, leaf.hash(), proof)
            .map(|found| hash![found, size] == tree_hash)
            .unwrap_or(false)
    }
}

/// Underlying function to recursively verify a proof
fn verify_proof_rec(index: usize, size: usize, leaf: Hash, proof: &[Hash]) -> Option<Hash> {
    let left_size = left_child_size(size);
    if size == 1 {
        Some(leaf)
    } else if proof.is_empty() {
        None
    } else if index < left_size {
        verify_proof_rec(index, left_size, leaf, &proof[1..]).map(|left| hash![left, proof[0]])
    } else {
        let relative_index = index - left_size;
        verify_proof_rec(relative_index, size - left_size, leaf, &proof[1..])
            .map(|right| hash![proof[0], right])
    }
}

/// Gets the size of the left child, given the size of a parent MerkleNode
/// x & (!x + 1) returns the lowest significant bit of x
/// for signed integers, use x & -x as rust guarantees two's complement
/// left_child_size(size) = 2 to the power most sigificant bit of size - 1
fn left_child_size(size: usize) -> usize {
    match size {
        0 => 0,
        1 => 0,
        _ => {
            let x = (size - 1).reverse_bits();
            (x & (!x + 1)).reverse_bits()
        }
    }
}

impl Hashable for MerkleTree {
    fn hash(&self) -> Hash {
        hash![self.nodes[self.root], self.size]
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
    fn merkle_tree_construction() {
        MerkleTree::new(&Vec::<u8>::new());
        MerkleTree::new::<u8>(&[1]);
        MerkleTree::new::<u8>(&[1, 2]);
        MerkleTree::new::<u8>(&[1, 2, 3]);
        MerkleTree::new::<u8>(&[1, 2, 3, 4]);
        MerkleTree::new::<u8>(&[1, 2, 3, 4, 5]);
    }

    #[test]
    fn child_size() {
        assert_eq!(left_child_size(0), 0);
        assert_eq!(left_child_size(1), 0);
        assert_eq!(left_child_size(2), 1);
        assert_eq!(left_child_size(3), 2);
        assert_eq!(left_child_size(4), 2);
        assert_eq!(left_child_size(5), 4);
        assert_eq!(left_child_size(32), 16);
        assert_eq!(left_child_size(33), 32);
        assert_eq!(left_child_size(64), 32);
        assert_eq!(left_child_size(65), 64);
    }

    #[test]
    fn proof_test() {
        const SIZE: usize = 23;
        let elements: Vec<i32> = (0..SIZE as i32).collect();
        let tree = MerkleTree::new(&elements);
        let tree_hash = tree.hash();
        for (index, item) in elements.iter().enumerate() {
            let proof = tree.construct_proof(index);
            assert!(MerkleTree::verify_proof(
                index, SIZE, *item, tree_hash, &proof
            ));
        }
    }
}
