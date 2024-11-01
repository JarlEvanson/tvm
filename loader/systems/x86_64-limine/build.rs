//! Build script for `tvm-loader` booted by `limine`.

/// Entry point to build script.
pub fn main() {
    println!("cargo::rustc-link-arg=-Tloader/systems/x86_64-limine/linker-script.ld");
}
