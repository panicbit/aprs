use std::ops::Deref;

pub trait GetGame {
    fn get_game(&self) -> &str;
}

impl<T> GetGame for T
where
    T: Deref,
    T::Target: GetGame,
{
    fn get_game(&self) -> &str {
        self.deref().get_game()
    }
}
