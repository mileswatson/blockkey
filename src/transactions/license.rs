use crate::crypto::contracts::{Contract, UserId};
use crate::crypto::hashing::*;

pub struct UnsignedLicenseCreation {
    pub seed: u64,
}

pub struct UnsignedLicenseTransfer {
    pub license: LicenseId,
    pub recipient: UserId,
}

pub type LicenseCreation = Contract<UnsignedLicenseCreation>;
pub type LicenseTransfer = Contract<UnsignedLicenseTransfer>;
pub type LicenseId = Hash<LicenseCreation>;

impl Hashable for UnsignedLicenseCreation {
    fn hash(&self) -> Hash<Self> {
        hash![self.seed]
    }
}

impl Hashable for UnsignedLicenseTransfer {
    fn hash(&self) -> Hash<Self> {
        hash![self.license, self.recipient]
    }
}
