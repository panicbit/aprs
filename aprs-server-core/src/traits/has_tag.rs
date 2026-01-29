use std::ops::Deref;

pub trait HasTag {
    fn has_tag(&self, tag: &str) -> bool;
}

impl<T> HasTag for T
where
    T: Deref,
    T::Target: HasTag,
{
    fn has_tag(&self, tag: &str) -> bool {
        self.deref().has_tag(tag)
    }
}
