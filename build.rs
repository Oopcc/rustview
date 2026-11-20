// Embeds assets/icon.ico into the Windows .exe (icon in Explorer / taskbar).
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "RustView");
        res.set("FileDescription", "RustView image viewer");
        if let Err(e) = res.compile() {
            eprintln!("warning: failed to embed Windows resources: {e}");
        }
    }
}
