use crate::transactions::license::{LicenseCreation, LicenseTemplate, LicenseTransfer};

pub mod license;
pub mod state;

pub enum Transaction {
    LicenseTemplate(LicenseTemplate),
    LicenseCreation(LicenseCreation),
    LicenseTransfer(LicenseTransfer),
}
