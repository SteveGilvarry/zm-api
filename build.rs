fn main() {
    // Basic build configuration for the ZoneMinder API
    // MSE plugin now uses socket-based communication instead of FFI
    
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
    
    println!("cargo:warning=MSE plugin now uses socket-based communication. No FFI library linking required.");
}
