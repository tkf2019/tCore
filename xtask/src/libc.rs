use clap::Args;
use std::process::Command;

/// Libc test build arguments
#[derive(Args, Default)]
pub struct LibcArgs {
    /// Static libc tests
    #[clap(long)]
    libc_static: bool,

    /// Dynamic libc tests
    #[clap(long)]
    libc_dynamic: bool,

    /// Clean before build
    #[clap(long)]
    libc_clean: bool,
}

impl LibcArgs {
    pub fn build(&self, libc_root: &str) {
        if self.libc_clean {
            Command::new("make")
                .current_dir(&libc_root)
                .arg("libc_clean")
                .status()
                .unwrap();
        }

        if self.libc_static {
            Command::new("make")
                .current_dir(&libc_root)
                .arg("build_static")
                .status()
                .unwrap();
        }

        if self.libc_dynamic {
            Command::new("make")
                .current_dir(&libc_root)
                .arg("build_libc_dynamic")
                .status()
                .unwrap();
        }
    }
}
