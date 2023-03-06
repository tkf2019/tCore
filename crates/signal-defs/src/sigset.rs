///
#[derive(Debug, Default, Clone, Copy)]
pub struct SigSet(u64);

impl SigSet {
    /// Creates a new `SigSet`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all bits set.
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// Returns true if no bit is set.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if the bit set.
    pub fn get(&self, kth: usize) -> bool {
        ((self.0 >> kth) & 1) == 1
    }

    /// Sets the bit.
    pub fn set(&mut self, kth: usize) {
        self.0 |= 1 << kth;
    }

    /// Unsets the bit.
    pub fn unset(&mut self, kth: usize) {
        self.0 &= !(1 << kth);
    }

    /// Gets union.
    pub fn union(&mut self, other: &SigSet) {
        self.0 |= other.0;
    }

    /// Gets intersection.
    pub fn intersection(&mut self, other: &SigSet) {
        self.0 &= other.0;
    }

    /// Gets difference.
    pub fn difference(&mut self, other: &SigSet) {
        self.0 &= !other.0;
    }

    
}

impl From<u64> for SigSet {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
