use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Permissions {
    /// permission for the `release` command
    pub release: CommandPermission,
    /// permission for the `collect` command
    pub collect: CommandPermission,
    /// permission for the `remaining` command
    pub remaining: RemainingCommandPermission,
}

#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone)]
pub enum CommandPermission {
    Disabled = 0b000,    // 0, completely disables access
    Enabled = 0b001,     // 1, allows manual use
    Goal = 0b010,        // 2, allows manual use after goal completion
    Auto = 0b110,        // 6, forces use after goal completion, only works for release and collect
    AutoEnabled = 0b111, // 7, forces use after goal completion, allows manual use any time
}

#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone)]
pub enum RemainingCommandPermission {
    Disabled = 0b000, // 0, completely disables access
    Enabled = 0b001,  // 1, allows manual use
    Goal = 0b010,     // 2, allows manual use after goal completion
}
