use crate::crypto::contracts::UserId;
use crate::crypto::hashing::{Hash, Hashable};
use crate::transactions::Transaction::{self, *};
use crate::transactions::{
    CurrencyTransfer, LicenseId, LicenseListing, LicenseOrder, LicensePurchase, LicenseTransfer,
    SelfListing,
};
use im_rc::HashMap;
use im_rc::HashSet;

#[derive(Clone)]
pub struct UserState {
    /// The current balance of the user (default=0)
    pub balance: u64,
    /// Cost to order a license. When price=0 (default), the
    /// license cannot be purchased.
    pub price: u64,
    /// Licenses owned by the user.
    pub licenses: HashSet<LicenseId>,
    /// Licenses up for resale.
    pub listings: HashMap<LicenseId, u64>,
}

impl Default for UserState {
    fn default() -> UserState {
        UserState {
            balance: 0,
            price: 0,
            licenses: HashSet::<LicenseId>::new(),
            listings: HashMap::<LicenseId, u64>::new(),
        }
    }
}

impl UserState {
    /// Adds the given amount to the user's balance, checking for overflow.
    fn deposit(&self, amount: u64) -> Option<UserState> {
        Some(UserState {
            // Use checked add to prevent overflow attack
            balance: self.balance.checked_add(amount)?,
            ..self.clone()
        })
    }

    /// Subtracts the given amount from the user's balance if funds are available.
    fn withdraw(&self, amount: u64) -> Option<UserState> {
        if amount > self.balance {
            None
        } else {
            Some(UserState {
                balance: self.balance - amount,
                ..self.clone()
            })
        }
    }

    /// Sets the price to order a license from a user.
    fn set_price(&self, amount: u64) -> Option<UserState> {
        Some(UserState {
            price: amount,
            ..self.clone()
        })
    }

    /// Adds a license to the user's collection.
    fn add_license(&self, license: LicenseId) -> Option<UserState> {
        Some(UserState {
            licenses: self.licenses.update(license),
            ..self.clone()
        })
    }

    /// Removes a license from the user's collection.
    fn remove_license(&self, license: LicenseId) -> Option<UserState> {
        if !self.licenses.contains(&license) {
            None
        } else {
            Some(UserState {
                licenses: self.licenses.without(&license),
                ..self.clone()
            })
        }
    }

    /// Adds a license to the user's listing.
    /// WARNING: DOES NOT REMOVE FROM COLLECTION.
    fn add_listing(&self, license: LicenseId, price: u64) -> Option<UserState> {
        Some(UserState {
            listings: self.listings.update(license, price),
            ..self.clone()
        })
    }

    /// Removes a license from the user's listing.
    /// WARNING: DOES NOT ADD BACK TO COLLECTION.
    fn remove_listing(&self, license: LicenseId) -> Option<UserState> {
        if !self.listings.contains_key(&license) {
            None
        } else {
            Some(UserState {
                listings: self.listings.without(&license),
                ..self.clone()
            })
        }
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub transactions: HashSet<Hash<Transaction>>,
    pub users: HashMap<UserId, UserState>,
}

impl State {
    /// Gets the state of a user by ID, or creates a default user if one doesn't exist.
    fn get_user(&self, user_id: UserId) -> UserState {
        self.users.get(&user_id).cloned().unwrap_or_default()
    }

    /// Applies a function to the state of a user.
    fn update_user<F>(&self, user_id: UserId, transform: F) -> Option<State>
    where
        F: FnOnce(UserState) -> Option<UserState>,
    {
        let user = self.get_user(user_id);
        Some(State {
            users: self.users.update(user_id, transform(user)?),
            ..self.clone()
        })
    }

    /// Records a transaction, asserting that it hasn't already been processed.
    fn record_transaction(&self, transaction: Hash<Transaction>) -> Option<State> {
        if self.transactions.contains(&transaction) {
            None
        } else {
            Some(State {
                transactions: self.transactions.update(transaction),
                ..self.clone()
            })
        }
    }

    /// Transfers an amount from one account to another (if funds are available).
    fn _transfer_currency(&self, from: UserId, to: UserId, amount: u64) -> Option<State> {
        if from == to {
            Some(self.clone())
        } else {
            self.update_user(from, |sender| sender.withdraw(amount))?
                .update_user(to, |recipient| recipient.deposit(amount))
        }
    }

    /// Applies a CurrencyTransfer transaction.
    fn transfer_currency(&self, transfer: &CurrencyTransfer) -> Option<State> {
        self.record_transaction(transfer.hash().cast())?
            ._transfer_currency(
                transfer.signee.hash(),
                transfer.content.recipient,
                transfer.content.amount,
            )
    }

    /// Applies a SelfListing transaction.
    fn list_self(&self, valuation: &SelfListing) -> Option<State> {
        let self_id = valuation.signee.hash();
        let price = valuation.content.price;
        self.record_transaction(valuation.hash().cast())?
            .update_user(self_id, |user| user.set_price(price))
    }

    /// Applies a LicenseOrder transaction.
    fn order_license(&self, valuation: &LicenseOrder) -> Option<State> {
        let seller_id = valuation.content.seller;
        let buyer_id = valuation.signee.hash();
        let price = valuation.content.price;
        let license = valuation.hash();

        if seller_id == buyer_id || (price != 0 && price == self.get_user(seller_id).price) {
            self.record_transaction(license.cast())?
                ._transfer_currency(buyer_id, seller_id, price)?
                .update_user(buyer_id, |buyer| buyer.add_license(license))
        } else {
            None
        }
    }

    /// Applies a LicenseListing transaction.
    fn list_license(&self, listing: &LicenseListing) -> Option<State> {
        let seller_id = listing.signee.hash();
        let license = listing.content.license;
        let price = listing.content.price;
        if price == 0 {
            None
        } else {
            self.record_transaction(listing.hash().cast())?
                .update_user(seller_id, |user| {
                    user.remove_license(license)?.add_listing(license, price)
                })
        }
    }

    /// Applies a LicensePurchase transaction.
    fn purchase_license(&self, purchase: &LicensePurchase) -> Option<State> {
        let seller_id = purchase.content.seller;
        let buyer_id = purchase.signee.hash();
        let price = purchase.content.price;
        let license = purchase.content.license;

        if seller_id == buyer_id || self.get_user(seller_id).listings.get(&license) == Some(&price)
        {
            self.record_transaction(purchase.hash().cast())?
                ._transfer_currency(seller_id, buyer_id, price)?
                .update_user(seller_id, |seller| seller.remove_listing(license))?
                .update_user(buyer_id, |buyer| buyer.add_license(license))
        } else {
            None
        }
    }

    /// Applies a LicenseTransfer transaction.
    fn transfer_license(&self, transfer: &LicenseTransfer) -> Option<State> {
        let license = transfer.content.license;
        let sender_id = transfer.signee.hash();
        let recipient_id = transfer.content.recipient;
        self.record_transaction(transfer.hash().cast())?
            .update_user(sender_id, |sender| sender.remove_license(license))?
            .update_user(recipient_id, |recipient| recipient.add_license(license))
    }

    /// Applies a transaction.
    pub fn apply(&self, transaction: &Transaction) -> Option<State> {
        match transaction {
            CurrencyTransfer(transfer) => self.transfer_currency(transfer),
            SelfListing(listing) => self.list_self(listing),
            LicenseOrder(order) => self.order_license(order),
            LicenseListing(listing) => self.list_license(listing),
            LicensePurchase(purchase) => self.purchase_license(purchase),
            LicenseTransfer(transfer) => self.transfer_license(transfer),
        }
    }
}

#[cfg(test)]
mod test {}
