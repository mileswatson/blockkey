use crate::crypto::contracts::Address;
use crate::hash;
use crate::crypto::hashing::*;

struct LicenceForging {
    forger: Address,
    product_id: u64,
    licence: u64,
}

impl Hashable for LicenceForging {
    fn hash(&self) -> Hash {
        hash![self.forger, self.product_id, self.licence]
    }
}

struct LicenceTransfer {
    sender: Address,
    recipient: Address,
    licence: Hash,
}

impl Hashable for LicenceTransfer {
    fn hash(&self) -> Hash {
        hash![self.sender, self.recipient, self.licence]
    }
}
