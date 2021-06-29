use crate::crypto::hashing::{Hash, Hashable};
use libp2p::identity;
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(PartialEq, Eq)]
pub struct PublicKey {
    key: identity::PublicKey,
}

pub type UserId = Hash<PublicKey>;

impl PublicKey {
    fn verify_bytes(&self, msg: &[u8], sig: &[u8]) -> bool {
        self.key.verify(msg, sig)
    }
}

impl Hashable for PublicKey {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.key.clone().into_protobuf_encoding()).cast()
    }
}

pub struct PrivateKey {
    keypair: identity::Keypair,
}

impl PrivateKey {
    pub fn generate() -> Self {
        PrivateKey {
            keypair: identity::Keypair::generate_ed25519(),
        }
    }

    pub fn get_public(&self) -> PublicKey {
        PublicKey {
            key: self.keypair.public(),
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

    fn sign_bytes(&self, msg: &[u8]) -> Vec<u8> {
        self.keypair.sign(msg).expect("Failed to sign bytes")
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract<T: Hashable> {
    pub signee: PublicKey,
    signature: Vec<u8>,
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

#[derive(Serialize, Deserialize)]
struct EncodedPublicKey {
    data: Vec<u8>,
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded_data = self.key.clone().into_protobuf_encoding();
        let encoded_public_key = EncodedPublicKey { data: encoded_data };
        encoded_public_key.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_public_key = EncodedPublicKey::deserialize(deserializer)?;
        let encoded_data = encoded_public_key.data;
        let key = identity::PublicKey::from_protobuf_encoding(&encoded_data)
            .map_err(de::Error::custom)?;
        Ok(PublicKey { key })
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
