use crate::crypto::contracts::Address;
use crate::crypto::hashing::*;

struct LicenseTemplate {
    seed: u64,
}

struct LicenseCreation {
    template: Hash,
}

struct LicenceTransfer {
    license: Hash,
    recipient: Address,
    product_id: u64,
    licence: u64,
}
