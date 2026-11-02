fn main() {
    // Ensure frontendDist directory exists so tauri::generate_context!() does not panic
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let dist_dir = std::path::Path::new(&manifest_dir).join("../dist");
    if !dist_dir.exists() {
        let _ = std::fs::create_dir_all(&dist_dir);
        let index_html = dist_dir.join("index.html");
        if !index_html.exists() {
            let _ = std::fs::write(
                index_html,
                "<!DOCTYPE html><html><head><title>Lumaq</title></head><body></body></html>",
            );
        }
    }

    tauri_build::build();

    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("windows-gnu") {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let rc_path = format!("{}/resource.rc", out_dir);
        let obj_path = format!("{}/resource.o", out_dir);

        if std::path::Path::new(&rc_path).exists() {
            let status = std::process::Command::new("windres")
                .args(["-i", &rc_path, "-o", &obj_path])
                .status();

            if let Ok(s) = status {
                if s.success() {
                    println!("cargo:rustc-link-arg={}", obj_path);
                }
            }
        }
    }
}
