use crate::crypto::contracts::{Contract, UserId};
use crate::crypto::hashing::*;
use serde::{Deserialize, Serialize};

pub mod state;

#[derive(Serialize, Deserialize)]
pub enum Transaction {
    CurrencyTransfer(CurrencyTransfer),
    SelfListing(SelfListing),
    LicenseOrder(LicenseOrder),
    LicenseListing(LicenseListing),
    LicensePurchase(LicensePurchase),
    LicenseTransfer(LicenseTransfer),
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedCurrencyTransfer {
    pub amount: u64,
    pub recipient: UserId,
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedSelfListing {
    pub price: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedLicenseOrder {
    pub seller: UserId,
    pub price: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedLicenseListing {
    pub license: LicenseId,
    pub price: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedLicensePurchase {
    pub seller: UserId,
    pub license: LicenseId,
    pub price: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedLicenseTransfer {
    pub license: LicenseId,
    pub recipient: UserId,
}

pub type CurrencyTransfer = Contract<UnsignedCurrencyTransfer>;
pub type SelfListing = Contract<UnsignedSelfListing>;
pub type LicenseOrder = Contract<UnsignedLicenseOrder>;
pub type LicenseListing = Contract<UnsignedLicenseListing>;
pub type LicensePurchase = Contract<UnsignedLicensePurchase>;
pub type LicenseTransfer = Contract<UnsignedLicenseTransfer>;

pub type LicenseId = Hash<LicenseOrder>;

impl Hashable for UnsignedCurrencyTransfer {
    fn hash(&self) -> Hash<Self> {
        hash![self.amount, self.recipient]
    }
}

impl Hashable for UnsignedSelfListing {
    fn hash(&self) -> Hash<Self> {
        hash![self.price]
    }
}

impl Hashable for UnsignedLicenseOrder {
    fn hash(&self) -> Hash<Self> {
        hash![self.seller, self.price]
    }
}

impl Hashable for UnsignedLicenseListing {
    fn hash(&self) -> Hash<Self> {
        hash![self.license, self.price]
    }
}

impl Hashable for UnsignedLicensePurchase {
    fn hash(&self) -> Hash<Self> {
        hash![self.seller, self.license, self.price]
    }
}

impl Hashable for UnsignedLicenseTransfer {
    fn hash(&self) -> Hash<Self> {
        hash![self.license, self.recipient]
    }
}
