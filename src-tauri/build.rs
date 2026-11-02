fn main() {
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
