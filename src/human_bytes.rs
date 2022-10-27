use crate::{Context, SizeOf};
use core::fmt::{self, Display};

/// Friendlier formatting for byte values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct HumanBytes {
    /// The inner bytes
    pub bytes: u64,
}

impl HumanBytes {
    /// Create a new `HumanBytes`
    #[inline]
    pub const fn new(bytes: u64) -> Self {
        Self { bytes }
    }

    /// Fetch the inner bytes of the `HumanBytes`
    #[inline]
    pub const fn into_inner(self) -> u64 {
        self.bytes
    }
}

impl From<u64> for HumanBytes {
    #[inline]
    fn from(bytes: u64) -> Self {
        Self { bytes }
    }
}

impl From<u32> for HumanBytes {
    #[inline]
    fn from(bytes: u32) -> Self {
        Self {
            bytes: bytes as u64,
        }
    }
}

impl From<usize> for HumanBytes {
    #[inline]
    fn from(bytes: usize) -> Self {
        Self {
            bytes: bytes as u64,
        }
    }
}

const KB: f64 = 1024.0;
const MB: f64 = KB * KB;
const GB: f64 = KB * KB * KB;
const TB: f64 = KB * KB * KB * KB;
const PB: f64 = KB * KB * KB * KB * KB;
const EB: f64 = KB * KB * KB * KB * KB * KB;
// const ZB: u64 = KB * KB * KB * KB * KB * KB * KB;
// const YB: u64 = KB * KB * KB * KB * KB * KB * KB * KB;

impl Display for HumanBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.bytes as f64;

        if bytes / EB > 1.0 {
            write!(f, "{:.02} EiB", bytes / EB)
        } else if bytes / PB > 1.0 {
            write!(f, "{:.02} PiB", bytes / PB)
        } else if bytes / TB > 1.0 {
            write!(f, "{:.02} TiB", bytes / TB)
        } else if bytes / GB > 1.0 {
            write!(f, "{:.02} GiB", bytes / GB)
        } else if bytes / MB > 1.0 {
            write!(f, "{:.02} MiB", bytes / MB)
        } else if bytes / KB > 1.0 {
            write!(f, "{:.02} KiB", bytes / KB)
        } else {
            write!(f, "{} B", self.bytes)
        }
    }
}

impl SizeOf for HumanBytes {
    fn size_of_children(&self, _context: &mut Context) {}
}
