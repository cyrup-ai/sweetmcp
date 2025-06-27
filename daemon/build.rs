fn main() {
    // The presence of libsystemd is enough; pkg‑config handles both Linux & vendored.
    if pkg_config::probe_library("libsystemd").is_ok() {
        println!("cargo:rustc-cfg=feature=\"systemd_available\"");
    }
}