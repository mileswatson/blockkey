use crate::crypto::contracts::UserId;
use crate::crypto::hashing::Hashable;
use crate::transactions::Transaction::{self, *};
use crate::transactions::{
    LicenseId, LicenseListing, LicenseOrder, LicensePurchase, LicenseTransfer, SelfListing,
};
use im_rc::HashMap;
use im_rc::HashSet;

#[derive(Clone)]
pub struct UserState {
    pub balance: u64,
    pub price: u64,
    pub created: HashSet<LicenseId>,
    pub licenses: HashSet<LicenseId>,
    pub listings: HashMap<LicenseId, u64>,
}

impl Default for UserState {
    fn default() -> UserState {
        UserState {
            balance: 0,
            price: 0,
            created: HashSet::<LicenseId>::new(),
            licenses: HashSet::<LicenseId>::new(),
            listings: HashMap::<LicenseId, u64>::new(),
        }
    }
}

impl UserState {
    fn deposit(&self, amount: u64) -> Option<UserState> {
        Some(UserState {
            balance: self.balance.checked_add(amount)?,
            ..self.clone()
        })
    }

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

    fn set_price(&self, amount: u64) -> Option<UserState> {
        Some(UserState {
            price: amount,
            ..self.clone()
        })
    }

    fn create_license(&self, license: LicenseId) -> Option<UserState> {
        if self.created.contains(&license) {
            None
        } else {
            Some(UserState {
                created: self.created.update(license),
                ..self.clone()
            })
        }
    }

    fn add_license(&self, license: LicenseId) -> Option<UserState> {
        Some(UserState {
            licenses: self.licenses.update(license),
            ..self.clone()
        })
    }

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

    fn add_listing(&self, license: LicenseId, price: u64) -> Option<UserState> {
        Some(UserState {
            listings: self.listings.update(license, price),
            ..self.clone()
        })
    }

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
    pub users: HashMap<UserId, UserState>,
}

impl State {
    fn get_user(&self, user_id: UserId) -> UserState {
        self.users.get(&user_id).cloned().unwrap_or_default()
    }

    fn update_user<F>(&self, user_id: UserId, transform: F) -> Option<State>
    where
        F: Fn(UserState) -> Option<UserState>,
    {
        let user = self.get_user(user_id);
        Some(State {
            users: self.users.update(user_id, transform(user)?),
        })
    }

    fn transfer_currency(&self, from: UserId, to: UserId, amount: u64) -> Option<State> {
        if from == to {
            Some(self.clone())
        } else {
            self.update_user(from, |sender| sender.withdraw(amount))
                .and_then(|state| state.update_user(to, |recipient| recipient.deposit(amount)))
        }
    }

    fn list_self(&self, valuation: &SelfListing) -> Option<State> {
        let self_id = valuation.signee.hash();
        let price = valuation.content.price;
        self.update_user(self_id, |user| user.set_price(price))
    }

    fn order_license(&self, valuation: &LicenseOrder) -> Option<State> {
        let seller_id = valuation.content.seller;
        let buyer_id = valuation.signee.hash();
        let price = valuation.content.price;
        let license = valuation.hash();

        if seller_id == buyer_id || (price != 0 && price == self.get_user(seller_id).price) {
            self.transfer_currency(buyer_id, seller_id, price)
                .and_then(|state| {
                    state.update_user(seller_id, |seller| seller.create_license(license))
                })
                .and_then(|state| state.update_user(buyer_id, |buyer| buyer.add_license(license)))
        } else {
            None
        }
    }

    fn list_license(&self, listing: &LicenseListing) -> Option<State> {
        let seller_id = listing.signee.hash();
        let license = listing.content.license;
        let price = listing.content.price;
        if price == 0 {
            None
        } else {
            self.update_user(seller_id, |user| {
                user.remove_license(license)
                    .and_then(|user| user.add_listing(license, price))
            })
        }
    }

    fn purchase_license(&self, purchase: &LicensePurchase) -> Option<State> {
        let seller_id = purchase.content.seller;
        let buyer_id = purchase.signee.hash();
        let price = purchase.content.price;
        let license = purchase.content.license;

        if seller_id == buyer_id || self.get_user(seller_id).listings.get(&license) == Some(&price)
        {
            self.transfer_currency(seller_id, buyer_id, price)
                .and_then(|state| {
                    state.update_user(seller_id, |seller| seller.remove_listing(license))
                })
                .and_then(|state| state.update_user(buyer_id, |buyer| buyer.add_license(license)))
        } else {
            None
        }
    }

    fn transfer_license(&self, transfer: &LicenseTransfer) -> Option<State> {
        let license = transfer.content.license;
        let sender_id = transfer.signee.hash();
        let recipient_id = transfer.content.recipient;
        self.update_user(sender_id, |sender| sender.remove_license(license))
            .and_then(|state| {
                state.update_user(recipient_id, |recipient| recipient.add_license(license))
            })
    }

    pub fn apply(&self, transaction: &Transaction) -> Option<State> {
        match transaction {
            CurrencyTransfer(transfer) => self.transfer_currency(
                transfer.signee.hash(),
                transfer.content.recipient,
                transfer.content.amount,
            ),
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
