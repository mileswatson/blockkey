use crate::crypto::hashing::Hashable;
use libp2p::identity;

// Definition of PublicKey
pub struct PublicKey {
    key: identity::PublicKey,
}

impl PublicKey {
    fn verify_bytes(&self, msg: &[u8], sig: &[u8]) -> bool {
        self.key.verify(msg, sig)
    }
}
// End of PublicKey

// Definition of Signature
pub struct Signature {
    signee: PublicKey,
    signature: Vec<u8>,
}

impl Signature {
    fn verify_bytes(&self, msg: &[u8]) -> bool {
        self.signee.verify_bytes(msg, &self.signature)
    }
}
// End of Signature

// Definition of PrivateKey
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

    fn sign_bytes(&self, msg: &[u8]) -> Signature {
        let signature = self.keypair.sign(msg).expect("Failed to sign bytes");

        Signature {
            signee: self.get_public(),
            signature,
        }
    }
}
// End of PrivateKey

// Definition of Contract
pub struct Contract<T: Hashable> {
    signature: Signature,
    contract: T,
}

impl<T: Hashable> Contract<T> {
    pub fn sign(contract: T, private_key: PrivateKey) -> Self {
        let hash = contract.hash();
        Contract {
            signature: private_key.sign_bytes(&hash.0),
            contract
        }
    }

    pub fn verify(&self) -> bool {
        let hash = self.contract.hash();
        self.signature.verify_bytes(&hash.0)
    }
}
// End of Contract

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_signature() {
        let private = PrivateKey::generate();
        let message = b"Hello world.";

        let signature = private.sign_bytes(message);
        assert!(signature.verify_bytes(message));
    }

    #[test]
    fn test_incorrect_signature() {
        let signer = PrivateKey::generate();
        let false_public = PrivateKey::generate().get_public();
        let message = b"Hello world.";

        let mut signature = signer.sign_bytes(message);
        signature.signee = false_public;
        assert!(!signature.verify_bytes(message));
    }
}

