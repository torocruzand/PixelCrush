use std::env;

fn main() {
    slint_build::compile("ui/app-window.slint").unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    // Windows: Embed the icon in the executable itself
    if target_os == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
    
    // Note: For macOS and Linux, icons are not embedded in the binary.
    // They are handled during the packaging step (e.g., App Bundle for Mac, .desktop for Linux).
}
