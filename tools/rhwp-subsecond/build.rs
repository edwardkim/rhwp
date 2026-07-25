#[cfg(unix)]
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }

    #[cfg(unix)]
    create_dioxus_rlib_alias();
}

#[cfg(unix)]
fn create_dioxus_rlib_alias() {
    use std::os::unix::fs::symlink;

    let Some(out_dir) = std::env::var_os("OUT_DIR").map(PathBuf::from) else {
        return;
    };
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        return;
    };
    let deps_dir = profile_dir.join("deps");
    let alias = deps_dir.join("librhwp-dioxus.rlib");

    if alias.exists() || std::fs::create_dir_all(&deps_dir).is_err() {
        return;
    }

    let _ = symlink("librhwp.rlib", alias);
}
