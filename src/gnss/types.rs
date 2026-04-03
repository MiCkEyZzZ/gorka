#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilliHz(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millimeter(pub i64);

// Временно добавлены вспомогательные методы для newtype.
// TODO: заменить на более идиоматичный API (конверсии или trait-методы)
impl Millimeter {
    pub fn new(v: i64) -> Self {
        Self(v)
    }

    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl MilliHz {
    pub fn new(v: i32) -> Self {
        Self(v)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}
