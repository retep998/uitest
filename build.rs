use std::process::Command;
use std::env::var_os;
use std::path::PathBuf;
fn main() {
    let pmanifest: PathBuf = var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let pout: PathBuf = var_os("OUT_DIR").unwrap().into();
    let prc = pmanifest.join("resource.rc");
    let pres = pout.join("resource.lib");
    let mut cmd = Command::new("rc.exe");
    cmd.arg("/fo").arg(&pres).arg(&prc);
    let status = cmd.status().unwrap();
    assert!(status.success(), "rc failed: 0x{:X}", status.code().unwrap());
    println!("cargo:rustc-link-lib=dylib=resource");
    println!("cargo:rustc-link-search={}", pout.display());
    println!("cargo:rerun-if-changed={}", prc.display());
    println!("cargo:rerun-if-changed={}", pmanifest.join("Bunny.ico").display());
    println!("cargo:rerun-if-changed={}", pmanifest.join("app.manifest").display());
}