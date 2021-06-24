use crate::crypto::contracts::Contract;
use crate::crypto::contracts::PublicKey;
use crate::crypto::hashing::*;

pub struct UnsignedLicenseTemplate {
    seed: u64,
}

impl Hashable for UnsignedLicenseTemplate {
    fn hash(&self) -> Hash<Self> {
        self.seed.hash().cast()
    }
}

pub struct UnsignedLicenseCreation {
    template: Hash<Contract<UnsignedLicenseTemplate>>,
}

impl Hashable for UnsignedLicenseCreation {
    fn hash(&self) -> Hash<Self> {
        self.template.cast()
    }
}

pub struct UnsignedLicenseTransfer {
    license: Hash<Contract<UnsignedLicenseCreation>>,
    recipient: Hash<PublicKey>,
}

impl Hashable for UnsignedLicenseTransfer {
    fn hash(&self) -> Hash<Self> {
        hash![self.license, self.recipient]
    }
}
