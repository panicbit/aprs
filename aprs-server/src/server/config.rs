use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub struct Config {
    state_path: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_state_path(mut self, state_path: PathBuf) -> Self {
        self.state_path = Some(state_path);
        self
    }

    pub fn with_state_path_opt(mut self, state_path: Option<PathBuf>) -> Self {
        self.state_path = state_path;
        self
    }

    pub fn state_path(&self) -> Option<&Path> {
        self.state_path.as_deref()
    }
}
