use crate::crypto::contracts::{PrivateKey, UserId};
use crate::network::{self, Network};
use crate::transactions::{
    LicenseId, Transaction, UnsignedCurrencyTransfer, UnsignedLicenseListing, UnsignedLicenseOrder,
    UnsignedLicensePurchase, UnsignedLicenseTransfer, UnsignedSelfListing,
};

const TRANSACTIONS_TOPIC: &str = "transactions";

#[allow(dead_code)]
struct Client {
    private: PrivateKey,
    network: Network,
}

#[allow(dead_code)]
impl Client {
    async fn new(private: PrivateKey) -> Self {
        Client {
            private,
            network: network::create(TRANSACTIONS_TOPIC).await.unwrap(),
        }
    }

    async fn send_transaction(&mut self, transaction: Transaction) {
        // Serialization should probably be changed to some other format later,
        // but json might be easier to debug for now.
        let message = serde_json::to_string(&transaction).unwrap();
        self.network.broadcast(message.as_bytes()).await.unwrap();
    }

    async fn make_currency_transfer(&mut self, amount: u64, recipient: UserId) {
        let transfer = self
            .private
            .sign(UnsignedCurrencyTransfer { amount, recipient });
        self.send_transaction(Transaction::CurrencyTransfer(transfer))
            .await;
    }

    async fn make_self_listing(&mut self, price: u64) {
        let listing = self.private.sign(UnsignedSelfListing { price });
        self.send_transaction(Transaction::SelfListing(listing))
            .await;
    }

    async fn make_license_order(&mut self, seller: UserId, price: u64) {
        let order = self.private.sign(UnsignedLicenseOrder { seller, price });
        self.send_transaction(Transaction::LicenseOrder(order))
            .await;
    }

    async fn make_license_listing(&mut self, license: LicenseId, price: u64) {
        let listing = self.private.sign(UnsignedLicenseListing { license, price });
        self.send_transaction(Transaction::LicenseListing(listing))
            .await;
    }

    async fn make_license_purchase(&mut self, seller: UserId, license: LicenseId, price: u64) {
        let purchase = self.private.sign(UnsignedLicensePurchase {
            seller,
            license,
            price,
        });
        self.send_transaction(Transaction::LicensePurchase(purchase))
            .await;
    }

    async fn make_license_transfer(&mut self, license: LicenseId, recipient: UserId) {
        let transfer = self
            .private
            .sign(UnsignedLicenseTransfer { license, recipient });
        self.send_transaction(Transaction::LicenseTransfer(transfer))
            .await;
    }
}
