use crate::crypto::hashing::{Hash, Hashable};
use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    key: ed25519_dalek::PublicKey,
}

pub type UserId = Hash<PublicKey>;

impl PublicKey {
    fn verify_bytes(&self, msg: &[u8], sig: &Signature) -> bool {
        self.key.verify(msg, sig).is_ok()
    }
}

impl Hashable for PublicKey {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.key.to_bytes()).cast()
    }
}

impl Hashable for Signature {
    fn hash(&self) -> Hash<Signature> {
        self.to_bytes()[..].hash().cast()
    }
}

pub struct PrivateKey {
    keypair: ed25519_dalek::Keypair,
}

impl PrivateKey {
    pub fn generate() -> Self {
        let mut csprng = rand::rngs::OsRng;
        PrivateKey {
            keypair: ed25519_dalek::Keypair::generate(&mut csprng),
        }
    }

    pub fn get_public(&self) -> PublicKey {
        PublicKey {
            key: self.keypair.public,
        }
    }

    pub fn sign<T: Hashable>(&self, content: T) -> Contract<T> {
        let timestamp = SystemTime::now().elapsed().unwrap().as_millis();
        let mut bytes_to_sign = content.hash().get_bytes().to_vec();
        bytes_to_sign.extend(timestamp.to_be_bytes().iter());

        Contract {
            signee: self.get_public(),
            signature: self.sign_bytes(&bytes_to_sign),
            timestamp,
            content,
        }
    }

    fn sign_bytes(&self, msg: &[u8]) -> Signature {
        self.keypair.sign(msg)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract<T: Hashable> {
    pub signee: PublicKey,
    signature: Signature,
    pub timestamp: u128,
    pub content: T,
}

impl<T: Hashable> Contract<T> {
    pub fn verify(&self) -> bool {
        let mut bytes_to_sign = self.content.hash().get_bytes().to_vec();
        bytes_to_sign.extend(self.timestamp.to_be_bytes().iter());

        self.signee.verify_bytes(&bytes_to_sign, &self.signature)
    }
}

impl<T: Hashable> Hashable for Contract<T> {
    fn hash(&self) -> Hash<Contract<T>> {
        hash![self.signee, self.signature, self.timestamp, self.content]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_contract() {
        let private = PrivateKey::generate();
        let message = 123;

        let contract = private.sign(message);

        assert!(contract.verify());
    }

    #[test]
    fn test_tampered_content() {
        let private = PrivateKey::generate();
        let message = 123;
        let mut contract = private.sign(message);

        contract.content = 321;

        assert!(!contract.verify());
    }

    #[test]
    fn test_tampered_signee() {
        let private = PrivateKey::generate();
        let message = 123;
        let mut contract = private.sign(message);

        contract.signee = PrivateKey::generate().get_public();

        assert!(!contract.verify());
    }

    #[test]
    fn test_serde_public_key() {
        let original = PrivateKey::generate().get_public();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: PublicKey = serde_json::from_str(&serialized).unwrap();

        assert!(original == deserialized);
    }

    #[test]
    fn test_serde_contract_i32() {
        let private = PrivateKey::generate();
        let message: i32 = 123;
        let original = private.sign(message);

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Contract<i32> = serde_json::from_str(&serialized).unwrap();

        assert!(original == deserialized);
    }
}
