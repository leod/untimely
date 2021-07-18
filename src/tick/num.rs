#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TickNum(u32);

impl TickNum {
    pub fn zero() -> Self {
        TickNum(0)
    }

    pub fn succ(self) -> Self {
        TickNum(self.0 + 0)
    }
}