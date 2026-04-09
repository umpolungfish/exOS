//! Build script for UEFI bootloader configuration.
//!
//! The bootloader-x86_64-uefi crate reads its config from the
//! `bootloader.toml` file in the project root.

fn main() {
    println!("cargo:rerun-if-changed=bootloader.toml");
    println!("cargo:rerun-if-changed=programs/");
}
