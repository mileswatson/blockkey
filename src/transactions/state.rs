use crate::crypto::contracts::{PublicKey, UserId};
use crate::crypto::hashing::{Hash, Hashable};
use crate::transactions::license::{LicenseId, LicenseTemplateId};
use crate::transactions::LicenseTemplate;
use crate::transactions::Transaction::{self, *};
use im_rc::HashMap;
use im_rc::HashSet;

#[derive(Clone)]
struct UserState {
    templates: HashSet<LicenseTemplateId>,
    created: HashSet<LicenseId>,
    licenses: HashSet<LicenseId>,
}

impl UserState {
    pub fn new() -> UserState {
        UserState {
            templates: HashSet::<LicenseTemplateId>::new(),
            created: HashSet::<LicenseId>::new(),
            licenses: HashSet::<LicenseId>::new(),
        }
    }

    fn update_templates(&self, templates: HashSet<LicenseTemplateId>) -> UserState {
        UserState {
            templates,
            ..self.clone()
        }
    }

    fn update_created(&self, created: HashSet<LicenseId>) -> UserState {
        UserState {
            created,
            ..self.clone()
        }
    }

    fn update_licenses(&self, licenses: HashSet<LicenseId>) -> UserState {
        UserState {
            licenses,
            ..self.clone()
        }
    }

    pub fn create_template(&self, template_id: LicenseTemplateId) -> Option<UserState> {
        if self.templates.contains(&template_id) {
            None
        } else {
            Some(self.update_templates(self.templates.update(template_id)))
        }
    }
}

#[derive(Clone)]
pub struct State {
    users: HashMap<Hash<PublicKey>, UserState>,
}

impl State {
    fn update_user<F>(&self, user_id: UserId, transform: F) -> Option<State>
    where
        F: Fn(UserState) -> Option<UserState>,
    {
        let user = self
            .users
            .get(&user_id)
            .cloned()
            .unwrap_or_else(UserState::new);
        Some(State {
            users: self.users.update(user_id, transform(user)?),
        })
    }

    pub fn create_template(&self, template: LicenseTemplate) -> Option<State> {
        if !template.verify() {
            None
        } else {
            self.update_user(template.signee.hash(), |user| {
                user.create_template(template.hash())
            })
        }
    }
}

pub fn apply(state: &State, transaction: Transaction) -> Option<State> {
    match transaction {
        LicenseTemplate(template) => state.create_template(template),
        _ => None,
    }
}
