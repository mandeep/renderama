fn main() {
    // Old versions of the `getrandom` crate (pulled in transitively via `rand 0.7`)
    // call `SystemFunction036` on Windows but don't declare the link dependency
    // on advapi32. Modern MSVC linkers don't auto-resolve this, so we add it here.
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=advapi32");
}