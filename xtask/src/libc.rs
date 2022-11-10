use clap::Args;
use std::process::Command;

/// Libc test build arguments
#[derive(Args, Default)]
pub struct LibcArgs {
    /// Static libc tests
    #[clap(long)]
    static_: bool,

    /// Dynamic libc tests
    #[clap(long)]
    dynamic: bool,

    /// Clean before build
    #[clap(long)]
    clean: bool,
}

impl LibcArgs {
    pub fn build(&self, libc_root: &str) {
        if self.clean {
            Command::new("make")
                .current_dir(&libc_root)
                .arg("clean")
                .status()
                .unwrap();
        }

        if self.static_ {
            Command::new("make")
                .current_dir(&libc_root)
                .arg("build_static")
                .status()
                .unwrap();
        }

        if self.dynamic {
            Command::new("make")
                .current_dir(&libc_root)
                .arg("build_dynamic")
                .status()
                .unwrap();
        }
    }
}
