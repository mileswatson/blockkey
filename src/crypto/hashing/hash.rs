use data_encoding::HEXUPPER;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq)]
pub struct Hash<T: ?Sized = ()>([u8; 32], PhantomData<T>);

impl<T: ?Sized> Clone for Hash<T> {
    fn clone(&self) -> Self {
        Hash(self.0, PhantomData)
    }
}

impl<T: ?Sized> Copy for Hash<T> {}

impl Hash {
    pub fn from_bytes(bytes: &[u8]) -> Hash<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        Hash(result.as_slice().try_into().unwrap(), PhantomData)
    }
}

impl<T> Hash<T> {
    pub fn empty() -> Hash<T> {
        Vec::<u8>::new().hash().cast()
    }

    pub fn get_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn cast<H>(&self) -> Hash<H> {
        Hash(self.0, PhantomData)
    }
}

impl<T> fmt::Display for Hash<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", HEXUPPER.encode(&self.0))
    }
}

pub trait Hashable<Input = Self> {
    fn hash(&self) -> Hash<Input>;
}

impl<T> Hashable<T> for Hash<T> {
    fn hash(&self) -> Hash<T> {
        *self
    }
}

impl Hashable for Vec<u8> {
    fn hash(&self) -> Hash<Self> {
        Hash::from_bytes(&self)
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

#[cfg(test)]
mod test {
    use crate::crypto::hashing::*;
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
