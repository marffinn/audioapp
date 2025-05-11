fn main() {
    #[cfg(windows)]
    {
        // Tell Cargo to use the windows subsystem (no console window)
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
}
