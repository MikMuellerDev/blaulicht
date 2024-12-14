fn main() {
    // pkg_config::probe_library("libudev").unwrap();

    tauri_build::build();

    println!("cargo:rustc-link-lib=udev");
}
