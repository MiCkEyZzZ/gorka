#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilliHz(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millimeter(pub i64);

impl Millimeter {
    pub fn new(v: i64) -> Self {
        Self(v)
    }
}

impl MilliHz {
    pub fn new(v: i32) -> Self {
        Self(v)
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}
