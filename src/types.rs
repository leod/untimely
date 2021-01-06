#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub u32);

impl EntityId {
    pub const ZERO: EntityId = EntityId(0);

    pub fn to_next(self) -> Self {
        EntityId(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerId(pub u32);

impl PlayerId {
    pub fn to_next(self) -> Self {
        PlayerId(self.0 + 1)
    }
}
