fn main() {
    tauri_build::build();

    // tauri_build emits cargo:rustc-link-arg-BINS which embeds the Windows
    // resource (manifest + icon) only in [[bin]] targets.  Without the manifest
    // the OS loads comctl32.dll v5.82 instead of v6, and TaskDialogIndirect
    // (v6-only) is absent → STATUS_ENTRYPOINT_NOT_FOUND when running tests.
    // Re-emit as cargo:rustc-link-arg so the test binary gets the manifest too.
    #[cfg(windows)]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let resource = std::path::Path::new(&out_dir).join("resource.lib");
        if resource.exists() {
            println!("cargo:rustc-link-arg={}", resource.display());
        }
    }
}
