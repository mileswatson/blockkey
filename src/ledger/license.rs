use crate::crypto::contracts::Contract;
use crate::crypto::contracts::PublicKey;
use crate::crypto::hashing::*;

struct LicenseTemplate {
    seed: u64,
}

impl Hashable for LicenseTemplate {
    type Input = Self;
    fn hash(&self) -> Hash<Self> {
        self.seed.hash().cast()
    }
}

struct LicenseCreation {
    template: Hash<Contract<LicenseTemplate>>,
}

impl Hashable for LicenseCreation {
    type Input = Self;
    fn hash(&self) -> Hash<Self> {
        self.template.cast()
    }
}

struct LicenseTransfer {
    license: Hash<Contract<LicenseCreation>>,
    recipient: Hash<PublicKey>,
}

impl Hashable for LicenseTransfer {
    type Input = Self;
    fn hash(&self) -> Hash<Self> {
        hash![self.license, self.recipient]
    }
}
