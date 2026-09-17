use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo::rustc-link-search={manifest_dir}");
    println!("cargo::rustc-link-arg-bins=-Tlink.x");
    // Without --nmagic the linker pads sections to page boundaries, which both
    // wastes flash and pushes the vector table off address 0.
    println!("cargo::rustc-link-arg-bins=--nmagic");
    println!("cargo::rerun-if-changed=link.x");
}
