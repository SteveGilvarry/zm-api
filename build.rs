fn main() {
    // Basic build configuration for the ZoneMinder API
    
    #[cfg(target_os = "macos")]
    {
        // Link essential system frameworks
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
    
    #[cfg(target_os = "linux")]
    {
        // Basic system libraries for Linux
        println!("cargo:rustc-link-lib=dylib=pthread");
    }
}
