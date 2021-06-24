use crate::crypto::contracts::{PublicKey, UserId};
use crate::crypto::hashing::{Hash, Hashable};
use crate::transactions::license::{LicenseCreation, LicenseId};
use crate::transactions::Transaction::{self, *};
use im_rc::HashMap;
use im_rc::HashSet;

#[derive(Clone)]
struct UserState {
    created: HashSet<LicenseId>,
    licenses: HashSet<LicenseId>,
}

impl UserState {
    fn new() -> UserState {
        UserState {
            created: HashSet::<LicenseId>::new(),
            licenses: HashSet::<LicenseId>::new(),
        }
    }

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
}

#[derive(Clone)]
pub struct State {
    users: HashMap<Hash<PublicKey>, UserState>,
}

impl State {
    fn get_user(&self, user_id: UserId) -> UserState {
        self.users
            .get(&user_id)
            .cloned()
            .unwrap_or_else(UserState::new)
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

    pub fn create_license(&self, creation: LicenseCreation) -> Option<State> {
        if !creation.verify() {
            None
        } else {
            self.update_user(creation.signee.hash(), |user| {
                user.create_license(creation.hash())
            })
        }
    }
}

pub fn apply(state: &State, transaction: Transaction) -> Option<State> {
    match transaction {
        LicenseCreation(creation) => state.create_license(creation),
        _ => None,
    }
}
