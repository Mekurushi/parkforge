use std::fmt;

use disc_riider::structs::WiiPartType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionKind {
    Data,
    Channel,
    Update,
}

impl PartitionKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Data => "DATA",
            Self::Channel => "CHANNEL",
            Self::Update => "UPDATE",
        }
    }

    pub(crate) fn as_disc_riider_type(self) -> WiiPartType {
        match self {
            Self::Data => WiiPartType::Data,
            Self::Channel => WiiPartType::Channel,
            Self::Update => WiiPartType::Update,
        }
    }
}

impl fmt::Display for PartitionKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
