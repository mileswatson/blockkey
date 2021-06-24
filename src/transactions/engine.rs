use crate::crypto::contracts::Contract;
use crate::transactions::license::{
    UnsignedLicenseCreation, UnsignedLicenseTemplate, UnsignedLicenseTransfer,
};

pub enum Transaction {
    LicenseTemplate(Contract<UnsignedLicenseTemplate>),
    LicenseCreation(Contract<UnsignedLicenseCreation>),
    LicenseTransfer(Contract<UnsignedLicenseTransfer>),
}
