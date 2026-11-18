use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::{ops::Deref, sync::Arc};

pub struct SharedStr<Brand>(Arc<str>, PhantomData<Brand>);

impl<Brand> SharedStr<Brand> {
    fn new(player_name: Arc<str>) -> Self {
        Self(player_name, PhantomData)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn rebrand<NewBrand>(&self) -> SharedStr<NewBrand> {
        SharedStr(self.0.clone(), PhantomData)
    }
}

impl<Brand> Deref for SharedStr<Brand> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<Brand> From<&str> for SharedStr<Brand> {
    fn from(value: &str) -> Self {
        Self::new(Arc::from(value))
    }
}

impl<Brand> From<String> for SharedStr<Brand> {
    fn from(value: String) -> Self {
        Self::new(Arc::from(value))
    }
}

impl<Brand> Clone for SharedStr<Brand> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

impl<Brand> PartialEq for SharedStr<Brand> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<Brand> Eq for SharedStr<Brand> {}

impl<Brand> PartialOrd for SharedStr<Brand> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Brand> Ord for SharedStr<Brand> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).then_with(|| self.1.cmp(&other.1))
    }
}

impl<Brand> Hash for SharedStr<Brand> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<Brand> fmt::Display for SharedStr<Brand> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
