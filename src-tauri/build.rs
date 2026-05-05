fn main() {
    // Ensure Tauri externalBin triple-suffixed copies exist before bundling.
    // Tauri looks for `binaries/smainer-provider-<TARGET_TRIPLE>[.exe]`;
    // we keep the plain names as source-of-truth and auto-copy here.
    let target_triple = std::env::var("TARGET").unwrap_or_default();
    let bin_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");

    if target_triple.contains("windows") {
        let src = bin_dir.join("smainer-provider.exe");
        let dst = bin_dir.join(format!("smainer-provider-{}.exe", target_triple));
        if src.exists() && !dst.exists() {
            std::fs::copy(&src, &dst).expect("failed to copy smainer-provider.exe");
        }
    } else if !target_triple.is_empty() {
        let src = bin_dir.join("smainer-provider");
        let dst = bin_dir.join(format!("smainer-provider-{}", target_triple));
        if src.exists() && (!dst.exists() || std::fs::metadata(&dst).map(|m| m.len()).unwrap_or(1) == 0) {
            std::fs::copy(&src, &dst).expect("failed to copy smainer-provider");
        }
    }

    tauri_build::build()
}