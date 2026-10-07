fn main() {
    #[cfg(target_os = "macos")]
    tiny_desktop::run();
    #[cfg(not(target_os = "macos"))]
    eprintln!("Tiny desktop requires macOS. Use `cargo run -p tiny -- --help` for the CLI.");
}
