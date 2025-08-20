fn main() {
    // Set macOS deployment target to 10.15 for whisper.cpp compatibility
    #[cfg(target_os = "macos")]
    {
        std::env::set_var("MACOSX_DEPLOYMENT_TARGET", "10.15");
        // Set cmake-specific deployment target directly
        std::env::set_var("CMAKE_OSX_DEPLOYMENT_TARGET", "10.15");
    }
    
    tauri_build::build()
}
