// build.rs
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=advapi32");

    if env::var("CARGO_FEATURE_DENOISE").is_ok() {
        if let Ok(oidn_dir) = env::var("OIDN_DIR") {
            let bin = PathBuf::from(&oidn_dir).join("bin");
            
            // target/{debug|release}/
            let profile = env::var("PROFILE").unwrap();
            let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("target")
                .join(&profile);
            
            fs::create_dir_all(&out_dir).ok();
            
            for dll in &["OpenImageDenoise.dll", "OpenImageDenoise_core.dll",
                         "OpenImageDenoise_device_cpu.dll", "tbb12.dll"] {
                let src = bin.join(dll);
                let dst = out_dir.join(dll);
                if src.exists() && !dst.exists() {
                    fs::copy(&src, &dst).ok();
                }
            }
        }
    }
}