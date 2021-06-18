use data_encoding::HEXUPPER;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn empty() -> Hash {
        Hash::from_bytes(&[])
    }

    // Method that takes a byte array and return hash
    pub fn from_bytes(msg: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(msg);
        let result = hasher.finalize();
        Hash(result.as_slice().try_into().unwrap())
    }

    pub fn get_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HEXUPPER.encode(&self.0))
    }
}

pub trait Hashable {
    fn hash(&self) -> Hash;
}

impl Hashable for Hash {
    fn hash(&self) -> Hash {
        *self
    }
}

impl Hashable for i8 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i16 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i32 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i64 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for i128 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for isize {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u8 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u16 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u32 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u64 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for u128 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for usize {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for f32 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for f64 {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self.to_be_bytes())
    }
}

impl Hashable for char {
    fn hash(&self) -> Hash {
        let mut bytes = [0; 2];
        self.encode_utf8(&mut bytes);
        Hash::from_bytes(&bytes)
    }
}

// There is no to_be_bytes for bool, so it's kinda hacky
impl Hashable for bool {
    fn hash(&self) -> Hash {
        (*self as i32).hash()
    }
}

impl Hashable for String {
    fn hash(&self) -> Hash {
        Hash::from_bytes(self.as_bytes())
    }
}

impl Hashable for Vec<u8> {
    fn hash(&self) -> Hash {
        Hash::from_bytes(&self)
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! hash {
    (impl $x:expr, $y:expr) => {
        $x.extend_from_slice($y.hash().get_bytes());
    };

    (impl $x:expr, $y:expr, $($z:expr),+) => {
        $x.extend_from_slice($y.hash().get_bytes());
        hash!(impl $x, $($z),+);
    };
    [$x:expr] => ( $x.hash() );
    [$($y:expr),+] => (
        {
            let mut v = vec![];
            hash!(impl &mut v, $($y),*);
            Hash::from_bytes(v.as_slice())
        }
    );
}

#[cfg(test)]
mod test {
    use crate::crypto::hashing::*;
    #[test]
    fn hash_transparency() {
        assert_eq!(hash![1, 2, 3], hash![1, 2.hash(), 3]);
    }

    #[test]
    fn nested_hashing() {
        assert_ne!(hash![1, 2, 3], hash![1, hash![2, 3]]);
    }
}
