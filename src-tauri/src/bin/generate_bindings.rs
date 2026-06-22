/// Standalone binary that regenerates `src/lib/bridge/bindings.ts`.
///
/// Run from `src-tauri/`:
///   cargo run --bin generate-bindings
///
/// The CI staleness check runs this, then `git diff --exit-code`.
fn main() {
    mantle_lib::export_bindings();
    println!("bindings written to ../src/lib/bridge/bindings.ts");
}
