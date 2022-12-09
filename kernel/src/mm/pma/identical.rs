use core::fmt;

use super::PMArea;

/// Represents an virtual memory area identically mapped, usually kernel
/// address space sections.
pub struct IdenticalPMA;

impl fmt::Debug for IdenticalPMA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Indentically mapped physical memory area")
    }
}

impl PMArea for IdenticalPMA {}
