use crate::crypto::contracts::UserId;
use crate::crypto::hashing::Hashable;
use crate::transactions::license::{LicenseCreation, LicenseId, LicenseTransfer};
use crate::transactions::Transaction::{self, *};
use im_rc::HashMap;
use im_rc::HashSet;

#[derive(Clone, Default)]
pub struct UserState {
    pub created: HashSet<LicenseId>,
    pub licenses: HashSet<LicenseId>,
}

impl UserState {
    fn create_license(&self, license: LicenseId) -> Option<UserState> {
        if self.created.contains(&license) {
            None
        } else {
            Some(UserState {
                created: self.created.update(license),
                licenses: self.licenses.update(license),
            })
        }
    }

    fn remove_license(&self, license: LicenseId) -> Option<UserState> {
        if !self.created.contains(&license) {
            None
        } else {
            Some(UserState {
                created: self.created.clone(),
                licenses: self.licenses.without(&license),
            })
        }
    }

    fn add_license(&self, license: LicenseId) -> Option<UserState> {
        Some(UserState {
            created: self.created.clone(),
            licenses: self.licenses.update(license),
        })
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

    pub fn create_license(&self, creation: &LicenseCreation) -> Option<State> {
        if !creation.verify() {
            None
        } else {
            self.update_user(creation.signee.hash(), |user| {
                user.create_license(creation.hash())
            })
        }
    }

    pub fn transfer_license(&self, transfer: &LicenseTransfer) -> Option<State> {
        if !transfer.verify() {
            None
        } else {
            let license = transfer.content.license;
            let sender_id = transfer.signee.hash();
            let recipient_id = transfer.content.recipient;
            self.update_user(sender_id, |sender| sender.remove_license(license))
                .and_then(|state| {
                    state.update_user(recipient_id, |recipient| recipient.add_license(license))
                })
        }
    }
}

pub fn apply(state: &State, transaction: &Transaction) -> Option<State> {
    match transaction {
        LicenseCreation(creation) => state.create_license(creation),
        LicenseTransfer(transfer) => state.transfer_license(transfer),
    }
}

#[cfg(test)]
mod test {
    use super::State;
    use crate::crypto::contracts::PrivateKey;
    use crate::crypto::hashing::Hashable;
    use crate::transactions::license::{UnsignedLicenseCreation, UnsignedLicenseTransfer};

    #[test]
    pub fn test_create_transfer() {
        let user1 = PrivateKey::generate();
        let user2 = PrivateKey::generate();

        let creation = user1.sign(UnsignedLicenseCreation { seed: 5 });
        let license = creation.hash();

        let transfer = user1.sign(UnsignedLicenseTransfer {
            license,
            recipient: user2.get_public().hash(),
        });

        let state = State::default()
            .create_license(&creation)
            .and_then(|s| s.transfer_license(&transfer))
            .unwrap();

        let user1_state = state.users.get(&user1.get_public().hash()).unwrap();
        let user2_state = state.users.get(&user2.get_public().hash()).unwrap();

        assert!(user1_state.created.contains(&license));

        assert!(!user1_state.licenses.contains(&license));

        assert!(user2_state.licenses.contains(&license));
    }
}
