use crate::crypto::contracts::{Contract, UserId};
use crate::crypto::hashing::*;

pub struct UnsignedLicenseTemplate {
    seed: u64,
}

pub struct UnsignedLicenseCreation {
    template: LicenseTemplateId,
}

pub struct UnsignedLicenseTransfer {
    license: LicenseId,
    recipient: UserId,
}

pub type LicenseTemplate = Contract<UnsignedLicenseTemplate>;
pub type LicenseCreation = Contract<UnsignedLicenseCreation>;
pub type LicenseTransfer = Contract<UnsignedLicenseTransfer>;
pub type LicenseTemplateId = Hash<LicenseTemplate>;
pub type LicenseId = Hash<LicenseCreation>;

impl Hashable for UnsignedLicenseTemplate {
    fn hash(&self) -> Hash<Self> {
        self.seed.hash().cast()
    }
}

impl Hashable for UnsignedLicenseCreation {
    fn hash(&self) -> Hash<Self> {
        self.template.cast()
    }
}

impl Hashable for UnsignedLicenseTransfer {
    fn hash(&self) -> Hash<Self> {
        hash![self.license, self.recipient]
    }
}
