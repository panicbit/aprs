use aprs_proto::primitives::SlotId;

pub struct PlandoItemBlock {
    player: SlotId,
    from_pool: bool,
    force: Force,
}

pub enum Force {
    True,
    False,
    Silent,
}

impl Force {
    pub fn is_true(&self) -> bool {
        matches!(self, Self::True)
    }

    pub fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }

    pub fn is_silent(&self) -> bool {
        matches!(self, Self::Silent)
    }
}

impl From<bool> for Force {
    fn from(value: bool) -> Self {
        match value {
            true => Force::True,
            false => Force::False,
        }
    }
}
