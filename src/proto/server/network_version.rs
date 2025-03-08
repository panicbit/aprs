use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct NetworkVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl Serialize for NetworkVersion {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // The python client requires an additional `class = "Version"` field,
        // however, this should not be required for deserialization.
        #[derive(Serialize)]
        #[serde(tag = "class", rename = "Version")]
        struct PythonNetworkVersion {
            major: u32,
            minor: u32,
            build: u32,
        }

        let NetworkVersion {
            major,
            minor,
            build,
        } = *self;

        let version = PythonNetworkVersion {
            major,
            minor,
            build,
        };

        version.serialize(ser)
    }
}

impl NetworkVersion {
    pub fn new(major: u32, minor: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            build,
        }
    }
}

impl From<(u32, u32, u32)> for NetworkVersion {
    fn from((major, minor, build): (u32, u32, u32)) -> Self {
        Self {
            major,
            minor,
            build,
        }
    }
}

impl From<archipelago_core::game::PickledVersion> for NetworkVersion {
    fn from(value: archipelago_core::game::PickledVersion) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            build: value.patch,
        }
    }
}
