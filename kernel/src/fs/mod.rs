use spin::{Lazy, Mutex};

mod fd;
mod stdio;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "oscomp"))] {
        mod efs;
        pub use efs::FileSystem;
    } else {
        mod fat;
        pub use fat::FileSystem;
    }
}
pub use fd::*;
pub use stdio::*;

pub static DISK_FS: Lazy<Mutex<FileSystem>> = Lazy::new(|| Mutex::new(FileSystem));
