#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct PlayerId(pub u32);

impl PlayerId {
    pub fn to_u32(self) -> u32 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EntityId(pub u32);

impl EntityId {
    pub fn to_u32(self) -> u32 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}
