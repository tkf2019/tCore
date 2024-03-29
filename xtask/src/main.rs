mod libc;
mod pack;

use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Args, Parser, Subcommand};
use libc::LibcArgs;
use once_cell::sync::Lazy;
use pack::PackArgs;

const DEFAULT_KERNEL: &str = "tcore-kernel";
const DEFAULT_ARCH: &str = "riscv64";
const DEFAULT_TARGET: &str = "riscv64gc-unknown-none-elf";

static PROJECT: Lazy<&'static Path> =
    Lazy::new(|| Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap());
static TARGET: Lazy<PathBuf> = Lazy::new(|| PROJECT.join("target").join(DEFAULT_TARGET));
static TEST: Lazy<PathBuf> = Lazy::new(|| PROJECT.join("test"));

const LOCAL_TESTCASES: &'static [&'static str] = &["hello_world"];
const LIBC_TESTCASES: &'static [&'static str] = &["hello"];

#[derive(Parser)]
#[clap(name = "tCore")]
#[clap(author, version, about, long_about = None)]
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
    /// Kernel package name.
    #[clap(long, default_value = DEFAULT_KERNEL)]
    kernel: Option<String>,

    /// Build architecture.
    #[clap(long, default_value = DEFAULT_ARCH)]
    arch: Option<String>,

    /// Build target.
    #[clap(long, default_value = DEFAULT_TARGET)]
    target: Option<String>,

    /// Run platform.
    #[clap(long, default_value = "qemu")]
    plat: Option<String>,

    /// Choose log level.
    #[clap(long, default_value = "debug")]
    log: Option<String>,

    /// Test mode.
    #[clap(long)]
    test: bool,

    /// Choose optimizing level.
    #[clap(long)]
    debug: bool,

    /// Dump binary file to ASM.
    #[clap(long)]
    dump: bool,

    /// If set, run build tests.
    #[clap(long)]
    build_test: bool,

    /// Use libc or local user tests.
    #[clap(long)]
    libc: bool,

    /// Testcases pre-built in test/oscomp.
    ///
    /// If set, the flag `test/libc` will be ignored.
    /// If not set, the `pack_args` will be ignored.
    #[clap(long)]
    oscomp: bool,

    /// Build libc tests.
    #[clap(flatten)]
    libc_args: LibcArgs,

    /// Pack arguments.
    #[clap(flatten)]
    pack_args: PackArgs,

    /// Build API docs.
    #[clap(long)]
    doc: bool,

    /// Other features
    #[clap(long)]
    other_features: Option<String>,
}

/// Prepare cargo utils.
fn check() {
    Command::new("cargo")
        .args(&["install", "cargo-binutils"])
        .status()
        .expect("Failed to install cargo-binutils");
    Command::new("rustup")
        .args(&["component", "add", "rust-src", "llvm-tools-preview"])
        .status()
        .expect("Failed to add components");
}

impl BuildArgs {
    /// Build local user tests.
    fn build_local_test(&self) {
        let target = self.target.as_ref().unwrap();
        let target = target.as_str();
        let user = TEST.join("user");
        let user_root = user.to_str().unwrap();
        let user_src = format!("{}/src", &user_root);
        let user_target = format!("{}/target", &user_root);
        println!("Building local tests {}", &user_root);
        // Build all user testcases
        Command::new("cargo")
            .arg("build")
            .arg("--quiet")
            .args(&["--package", "user_lib"])
            .args(&["--target", target])
            .arg("--release")
            .env("CARGO_TARGET_DIR", &user_target)
            .env(
                "RUSTFLAGS",
                format!("-Clink-arg=-T{}", format!("{}/linker.ld", user_src)),
            )
            .status()
            .expect("Failed to run cargo");

        // Build easy_fs image form local testcase list
        let mut cases: Vec<&str> = Vec::new();
        cases.extend(LOCAL_TESTCASES.into_iter());
        self.pack_args
            .pack_easy_fs(
                &cases,
                format!(
                    "{}/{}/{}",
                    &user_target,
                    target,
                    if self.debug { "debug" } else { "release" }
                ),
            )
            .expect("Faild to pack user tests");
    }

    /// Build Libc user tests.
    fn build_libc_test(&self) {
        let libc = TEST.join("libc");
        let libc_root = libc.to_str().unwrap();
        let libc_build = format!("{}/build", &libc_root);
        // Make libc static tests
        self.libc_args.build(&libc_root);
        // Build easy_fs image form libc testcase list
        let mut cases: Vec<&str> = Vec::new();
        cases.extend(LIBC_TESTCASES.into_iter());
        self.pack_args
            .pack_easy_fs(&cases, libc_build)
            .expect("Faild to pack libc tests");
    }

    /// Build oscomp user tests.
    fn build_oscomp_test(&self) {
        self.pack_args.pack_fat32();
    }

    /// Build testcases.
    ///
    /// Returns a feature string parsed from command line arguments.
    fn build_test(&self) {
        if self.oscomp {
            self.build_oscomp_test();
        } else if self.libc {
            self.build_libc_test();
        } else {
            self.build_local_test();
        }
    }

    /// Dump kernel ELF to an assembly file.
    fn dump(&self) {
        let kernel = TARGET
            .join(if self.debug { "debug" } else { "release" })
            .join(self.kernel.as_ref().unwrap());
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

    fn make(&self) -> PathBuf {
        // Debug mode, Release mode and Test mode
        let target = self.target.as_ref().unwrap().as_str();
        let opt_level = if self.debug {
            "--profile=dev"
        } else {
            "--release"
        };
        let (subcmd, options) = if self.doc {
            ("doc", opt_level)
        } else {
            ("build", opt_level)
        };
        let test = if self.test { "test" } else { "" };
        let oscomp = if self.oscomp { "oscomp" } else { "" };

        if self.build_test {
            self.build_test();
        }

        // Linker file for target platform to configure kernel layout
        let linker = PROJECT
            .join("plat")
            .join(self.plat.as_ref().unwrap())
            .join("linker.ld");

        let mut features = format!("{} {} ", test, oscomp);
        if self.other_features.is_some() {
            self.other_features
                .as_ref()
                .unwrap()
                .split(',')
                .for_each(|f| {
                    features.push_str(f);
                    features.push(' ');
                });
        }
        println!("Features {}", features);

        // Run cargo build command
        Command::new("cargo")
            .arg(subcmd)
            .args(&["--package", self.kernel.as_ref().unwrap().as_str()])
            .args(&["--target", target])
            .args(&["--features", features.as_str()])
            .arg(options)
            .env("LOG", self.log.as_ref().unwrap().as_str())
            .env(
                "RUSTFLAGS",
                format!("-Clink-arg=-T{}", linker.as_os_str().to_str().unwrap()),
            )
            .status()
            .expect("Failed to run cargo");

        // Dump ASM file
        if self.dump {
            self.dump();
        }

        TARGET
            .join(if self.debug { "debug" } else { "release" })
            .join(self.kernel.as_ref().unwrap())
    }
}

/// Run on Qemu
#[derive(Args)]
struct QemuArgs {
    /// Use Build Arguments
    #[clap(flatten)]
    build: BuildArgs,

    /// QEMU path
    #[clap(long, default_value = "")]
    qemu: Option<String>,

    /// Multiprocessing
    #[clap(long, default_value = "1")]
    smp: Option<String>,
}

impl QemuArgs {
    fn run(&self) {
        // Build the kernel ELF
        assert!(self.build.plat.as_ref().unwrap().eq("qemu"));
        let kernel = self.build.make();

        // Kernel binary for qemu
        // let kernel_bin = kernel.with_extension("bin");
        // Command::new("rust-objcopy")
        //     .arg(format!(
        //         "--binary-architecture={}",
        //         self.build.arch.as_ref().unwrap().as_str()
        //     ))
        //     .arg(&kernel)
        //     .args(&["--strip-all", "-O", "binary"])
        //     .arg(&kernel_bin)
        //     .status()
        //     .expect("Failed to generate kernel binary file");
        // Run Qemu
        let mut cmd = Command::new(format!(
            "{}qemu-system-{}",
            self.qemu.as_ref().unwrap().as_str(),
            self.build.arch.as_ref().unwrap().as_str()
        ));
        cmd.args(&["-machine", "virt"])
            .args(&["-m", "2G"])
            .args(&["-smp", self.smp.as_ref().unwrap().as_str()])
            .arg("-nographic")
            .args(&["-bios", "default"])
            .arg("-kernel")
            .arg(&kernel)
            .args(&["-serial", "mon:stdio"])
            .args(&[
                "-drive",
                format!(
                    "file={},if=none,format=raw,id=x0",
                    self.build.pack_args.pack_image.as_ref().unwrap().clone()
                )
                .as_str(),
                "-device",
                "virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0",
            ])
            .status()
            .expect("Failed to run qemu");
    }
}

fn main() {
    match Commands::parse().inner {
        Subcommands::Make(args) => {
            check();
            args.make();
        }
        Subcommands::Qemu(args) => {
            check();
            args.run()
        }
    }
}
