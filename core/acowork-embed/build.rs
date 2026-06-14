//! Build script for acowork-embed.
//!
//! ORT linking is configured via the `ORT_LIB_LOCATION` environment variable.
//! This variable is auto-detected at build time by:
//!   - `dev/build_core.sh`      (Linux / macOS — probes .ort/ directory)
//!   - `dev/build_core.ps1`     (Windows — probes .ort/ directory)
//!   - `dev/ort_env.js`         (Tauri dev mode — probes .ort/ directory)
//!
//! ORT is installed to .ort/ by:
//!   - `dev/setup_ort.sh`       (Linux / macOS)
//!   - `dev/setup_ort.ps1`      (Windows)
//!
//! If `ORT_LIB_LOCATION` is unset and `download-ort` is not active, this
//! script emits a warning so developers know to install ORT first.

fn main() {
    println!("cargo:rerun-if-env-changed=ORT_LIB_LOCATION");
    println!("cargo:rerun-if-env-changed=ORT_DYLIB_PATH");
    println!("cargo:rerun-if-env-changed=ORT_PREFER_DYNAMIC_LINK");

    if std::env::var("ORT_LIB_LOCATION").is_ok() {
        return;
    }

    if cfg!(feature = "download-ort") {
        return;
    }

    // ORT not configured — ort-sys will print its own detailed error
    // during the link step. Just emit a heads-up.
    println!(
        "cargo:warning=ORT_LIB_LOCATION not set and download-ort not enabled. \
         Run dev/setup_ort.ps1 (Windows) or dev/setup_ort.sh (Linux/macOS) to install ONNX Runtime."
    );
}
