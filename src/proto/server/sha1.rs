use serde::Serialize;
use sha1::Digest;

#[derive(Copy, Clone)]
pub struct Sha1([u8; 20]);

impl Sha1 {
    pub fn digest(bytes: &[u8]) -> Self {
        let hash = sha1::Sha1::digest(bytes);

        Sha1(hash.into())
    }
}

impl Serialize for Sha1 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}
