#[macro_use]
extern crate clap;

use clap::Parser;
use once_cell::sync::Lazy;
use os_xtask_utils::{BinUtil, Cargo, CommandExt, Qemu};
use std::path::{Path, PathBuf};

static PROJECT: Lazy<&'static Path> =
    Lazy::new(|| Path::new(std::env!("CARGO_MANIFEST_DIR")).parent().unwrap());

#[derive(Parser)]
#[clap(name = "try-rtos")]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Make(BuildArgs),
    Asm,
    Qemu(QemuArgs),
}

fn main() {
    use Commands::*;
    match Cli::parse().command {
        Make(args) => {
            args.make();
        }
        Asm => todo!(),
        Qemu(args) => args.run(),
    }
}

#[derive(Args, Default)]
struct BuildArgs {
    /// log level
    #[clap(long)]
    log: Option<String>,
    /// build in debug mode
    #[clap(long)]
    debug: bool,
}

impl BuildArgs {
    fn make(&self) -> PathBuf {
        let package = "rtos";
        let target = "riscv32imac-unknown-none-elf";
        Cargo::build()
            .package(package)
            .optional(&self.log, |cargo, log| {
                cargo.env("LOG", log);
            })
            .conditional(!self.debug, |cargo| {
                cargo.release();
            })
            .target(target)
            .invoke();
        PROJECT
            .join("target")
            .join(target)
            .join(if self.debug { "debug" } else { "release" })
            .join(package)
    }
}

#[derive(Args)]
struct QemuArgs {
    #[clap(flatten)]
    build: BuildArgs,
    /// Port for gdb to connect. If set, qemu will block and wait gdb to connect.
    #[clap(long)]
    gdb: Option<u16>,
}

impl QemuArgs {
    fn run(self) {
        let elf = self.build.make();
        Qemu::system("riscv32")
            .args(&["-machine", "virt"])
            .arg("-nographic")
            .arg("-bios")
            .arg(objcopy(elf, true))
            .args(&["-serial", "mon:stdio"])
            .optional(&self.gdb, |qemu, gdb| {
                qemu.args(&["-S", "-gdb", &format!("tcp::{gdb}")]);
            })
            .invoke();
    }
}

fn objcopy(elf: impl AsRef<Path>, binary: bool) -> PathBuf {
    let elf = elf.as_ref();
    let bin = elf.with_extension("bin");
    BinUtil::objcopy()
        .arg(elf)
        .arg("--strip-all")
        .conditional(binary, |binutil| {
            binutil.args(["-O", "binary"]);
        })
        .arg(&bin)
        .invoke();
    bin
}
