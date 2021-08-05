use data_encoding::HEXUPPER;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug)]
pub struct Hash<T: ?Sized = ()>([u8; 32], PhantomData<T>);

impl<T: ?Sized> Clone for Hash<T> {
    fn clone(&self) -> Self {
        Hash(self.0, PhantomData)
    }
}

impl<T: ?Sized> PartialEq for Hash<T> {
    fn eq(&self, h: &Self) -> bool {
        self.0 == h.0
    }
}

impl<T: ?Sized> Eq for Hash<T> {}

impl<T: ?Sized> Copy for Hash<T> {}

impl<T: ?Sized> std::hash::Hash for Hash<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Hash {
    pub fn from_bytes(bytes: &[u8]) -> Hash<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        Hash(result.as_slice().try_into().unwrap(), PhantomData)
    }
}

impl<T: ?Sized> Hash<T> {
    pub fn empty() -> Hash<T> {
        Vec::<u8>::new().hash().cast()
    }

    pub fn get_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn cast<H: ?Sized>(&self) -> Hash<H> {
        Hash(self.0, PhantomData)
    }
}

impl<T: ?Sized> fmt::Display for Hash<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HEXUPPER.encode(&self.0))
    }
}

pub trait Hashable<Input: ?Sized = Self> {
    fn hash(&self) -> Hash<Input>;
}

impl<T: ?Sized> Hashable<T> for Hash<T> {
    fn hash(&self) -> Hash<T> {
        *self
    }
}

impl Hashable for Vec<u8> {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(self)
    }
}

impl Hashable for [u8] {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(self).cast()
    }
}

impl Hashable for usize {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.to_be_bytes()).cast()
    }
}

impl Hashable for i32 {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.to_be_bytes()).cast()
    }
}

impl Hashable for u8 {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.to_be_bytes()).cast()
    }
}

impl Hashable for u64 {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.to_be_bytes()).cast()
    }
}

impl Hashable for u128 {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self.to_be_bytes()).cast()
    }
}

impl Hashable for bool {
    fn hash(&self) -> Hash<Self> {
        match self {
            true => 1u8,
            false => 0u8,
        }
        .hash()
        .cast()
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
    [$x:expr] => ( $x.hash().cast() );
    [$($y:expr),+] => (
        {
            let mut v = vec![];
            hash!(impl &mut v, $($y),*);
            Hash::from_bytes(v.as_slice()).cast()
        }
    );
}

impl<T: Hashable> Hashable for Option<T> {
    fn hash(&self) -> Hash<Self> {
        match self {
            Some(t) => hash![true, t],
            None => hash![false, Hash::<T>::empty()],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::crypto::hashing::*;

    #[test]
    fn equality() {
        let x: Hash = hash![1, 2, 3];
        let y: Hash = hash![1, 2, 3];
        assert_eq!(x, y);
    }

    #[test]
    fn hash_transparency() {
        let x: Hash = hash![1, 2, 3];
        assert_eq!(x, hash![1, 2.hash(), 3]);
    }

    #[test]
    fn nested_hashing() {
        let x: Hash = hash![1, 2, 3];
        let y: Hash = hash![2, 3];
        assert_ne!(x, hash![1, y]);
    }
}
