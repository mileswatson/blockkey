use crate::transactions::license::{LicenseCreation, LicenseTransfer};

pub mod license;
pub mod state;

pub enum Transaction {
    LicenseCreation(LicenseCreation),
    LicenseTransfer(LicenseTransfer),
}
