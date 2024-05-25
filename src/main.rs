use std::process::Command;

use clap::{command, Parser, Subcommand};

/// Qemu Wrapper
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// use uefi to boot
    #[arg(short, long, default_value_t = false)]
    uefi: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run,
    Debug,
    Asm,
    Gdb,
}

fn main() {
    let args = Args::parse();

    let uefi = args.uefi;

    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    let mut qemu = std::process::Command::new("qemu-system-x86_64");
    qemu.args(vec!["-nographic", "-m", "128M"]);
    if uefi {
        qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        qemu.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        qemu.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }
    // let mut child = cmd.spawn().unwrap();
    // child.wait().unwrap();

    let mut cmd = match &args.command {
        Commands::Run => qemu,
        Commands::Debug => debug(qemu),
        Commands::Asm => todo!(),
        Commands::Gdb => gdb(),
    };
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}

fn debug(mut qemu: Command) -> Command {
    qemu.args(vec!["-S", "-s"]);
    qemu
}

fn gdb() -> Command {
    let kernel_elf = env!("KERNEL_ELF");
    let mut gdb = std::process::Command::new("gdb");
    gdb.arg(kernel_elf);
    gdb
}
