use crate::crypto::hashing::Hashable;
use libp2p::identity;

pub struct PublicKey {
    key: identity::PublicKey,
}

impl PublicKey {
    fn verify_bytes(&self, msg: &[u8], sig: &[u8]) -> bool {
        self.key.verify(msg, sig)
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
        Contract {
            signee: self.get_public(),
            signature: self.sign_bytes(&content.hash().0),
            content,
        }
    }

    fn sign_bytes(&self, msg: &[u8]) -> Vec<u8> {
        self.keypair.sign(msg).expect("Failed to sign bytes")
    }
}

pub struct Contract<T: Hashable> {
    signee: PublicKey,
    signature: Vec<u8>,
    content: T,
}

impl<T: Hashable> Contract<T> {
    pub fn verify(&self) -> bool {
        let hash = self.content.hash();
        self.signee.verify_bytes(&hash.0, &self.signature)
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
}
