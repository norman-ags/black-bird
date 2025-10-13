fn main() {
    // Enable system tray feature conditionally based on platform
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-cfg=feature=\"system-tray\"");
        println!("cargo:warning=Windows build: System tray enabled by default");
    }

    #[cfg(target_os = "linux")]
    {
        // Check if libappindicator is available, if not, disable system tray
        if std::process::Command::new("pkg-config")
            .args(&["--exists", "ayatana-appindicator3-0.1"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            println!("cargo:rustc-cfg=feature=\"system-tray\"");
            println!("cargo:warning=Linux build: System tray enabled (libayatana-appindicator found)");
        } else if std::process::Command::new("pkg-config")
            .args(&["--exists", "appindicator3-0.1"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            println!("cargo:rustc-cfg=feature=\"system-tray\"");
            println!("cargo:warning=Linux build: System tray enabled (libappindicator found)");
        } else {
            println!("cargo:warning=Linux build: System tray disabled (libappindicator not found)");
            println!("cargo:warning=Install libayatana-appindicator3-dev or libappindicator3-dev for tray support");
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-cfg=feature=\"system-tray\"");
        println!("cargo:warning=macOS build: System tray enabled by default");
    }

    tauri_build::build()
}
