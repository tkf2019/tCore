mod pack;

use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Args, Parser, Subcommand};
use once_cell::sync::Lazy;

const DEFAULT_KERNEL: &str = "tcore-kernel";
const DEFAULT_ARCH: &str = "riscv64";
const DEFAULT_TARGET: &str = "riscv64gc-unknown-none-elf";

static PROJECT: Lazy<&'static Path> =
    Lazy::new(|| Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap());
static TARGET: Lazy<PathBuf> = Lazy::new(|| PROJECT.join("target").join(DEFAULT_TARGET));

#[derive(Parser)]
#[clap(name = "tCore")]
#[clap(version, about, long_about = None)]
struct Commands {
    #[clap(subcommand)]
    inner: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Make(BuildArgs),
    Qemu(QemuArgs),
}

/// Main build arguments for this project
#[derive(Args, Default)]
struct BuildArgs {
    /// Kernel package name
    #[clap(long, default_value = DEFAULT_KERNEL)]
    kernel: Option<String>,

    /// Build architecture
    #[clap(long, default_value = DEFAULT_ARCH)]
    arch: Option<String>,

    /// Build target
    #[clap(long, default_value = DEFAULT_TARGET)]
    target: Option<String>,

    /// Run platform
    #[clap(long, default_value = "qemu")]
    plat: Option<String>,

    /// Choose log level
    #[clap(long, default_value = "debug")]
    log: Option<String>,

    /// Start test instead of normal build
    #[clap(long)]
    test: bool,

    /// Choose optimizing level
    #[clap(long)]
    debug: bool,

    /// Dump binary file to ASM
    #[clap(long)]
    dump: bool,

    /// Use global or local user tests
    #[clap(long)]
    global: bool,
}

impl BuildArgs {
    fn make(&self) -> PathBuf {
        // Prepare cargo utils
        Command::new("cargo")
            .args(&["install", "cargo-binutils"])
            .status()
            .expect("Failed to install cargo-binutils");
        Command::new("rustup")
            .args(&["component", "add", "rust-src", "llvm-tools-preview"])
            .status()
            .expect("Failed to add components");

        // Debug mode, Release mode and Test mode
        let opt_level = if self.debug {
            "--profile=dev"
        } else {
            "--release"
        };
        let (subcmd, options) = if self.test {
            ("rustc", "-- --test")
        } else {
            ("build", opt_level)
        };
        let features = if self.global {
            "global_test"
        } else {
            "local_test"
        };

        // Linker file for target platform to configure kernel layout
        let linker = PROJECT
            .join("plat")
            .join(self.plat.as_ref().unwrap())
            .join("linker.ld");

        // Run cargo build command
        Command::new("cargo")
            .arg(subcmd)
            .args(&["--package", self.kernel.as_ref().unwrap().as_str()])
            .args(&["--target", self.target.as_ref().unwrap().as_str()])
            .args(&["--features", features])
            .arg(options)
            .env("LOG", self.log.as_ref().unwrap().as_str())
            .env(
                "RUSTFLAGS",
                format!("-Clink-arg=-T{}", linker.as_os_str().to_str().unwrap()),
            )
            .status()
            .expect("Failed to run cargo");
        let kernel = TARGET
            .join(if self.debug { "debug" } else { "release" })
            .join(self.kernel.as_ref().unwrap());

        // Dump ASM file
        if self.dump {
            let kernel_asm = kernel.with_extension("S");
            let asm = Command::new("rust-objdump")
                .arg(format!(
                    "--arch-name={}",
                    self.arch.as_ref().unwrap().as_str()
                ))
                .args(&["-d"])
                .arg(&kernel)
                .output()
                .expect("Failed to dump kernel ASM");
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(kernel_asm.as_os_str().to_str().unwrap())
                .expect("Failed to open or create target asm file");
            file.write(String::from_utf8_lossy(&asm.stdout).as_bytes())
                .expect("Failed to write to target asm file");
        }
        kernel
    }
}

/// Run on Qemu
#[derive(Args)]
struct QemuArgs {
    /// Use Build Arguments
    #[clap(flatten)]
    build: BuildArgs,
}

impl QemuArgs {
    fn run(&self) {
        // Build the kernel ELF
        assert!(self.build.plat.as_ref().unwrap().eq("qemu"));
        let kernel = self.build.make();

        // Kernel binary for qemu
        let kernel_bin = kernel.with_extension("bin");
        Command::new("rust-objcopy")
            .arg(format!(
                "--binary-architecture={}",
                self.build.arch.as_ref().unwrap().as_str()
            ))
            .arg(&kernel)
            .args(&["--strip-all", "-O", "binary"])
            .arg(&kernel_bin)
            .status()
            .expect("Failed to generate kernel binary file");

        // RustSBI bootloader binary for qemu
        let bootloader = PROJECT.join("plat/qemu/rustsbi.bin");

        // Run Qemu
        Command::new(format!(
            "qemu-system-{}",
            self.build.arch.as_ref().unwrap().as_str()
        ))
        .args(&["-machine", "virt"])
        .args(&["-m", "2G"])
        .arg("-nographic")
        .arg("-bios")
        .arg(&bootloader)
        .arg("-kernel")
        .arg(&kernel_bin)
        .args(&["-serial", "mon:stdio"])
        .status()
        .expect("Failed to run qemu");
    }
}

fn main() {
    match Commands::parse().inner {
        Subcommands::Make(args) => {
            args.make();
        }
        Subcommands::Qemu(args) => args.run(),
    }
}
