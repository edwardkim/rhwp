//! 선택 어댑터 파일 존재 여부를 cfg 로 올린다.
//!
//! `PngBackend` / `SkiaBackend` 는 devel 에는 없고 M06-1/M06-2 가 파일을 추가하면
//! 나타난다. 계약 시험은 그 타입이 있을 때만 컴파일되어야 한다.

fn main() {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let backend = root.join("src").join("render_backend");

    println!("cargo:rerun-if-changed=src/render_backend/png_adapter.rs");
    println!("cargo:rerun-if-changed=src/render_backend/skia_adapter.rs");
    println!("cargo:rustc-check-cfg=cfg(rhwp_has_png_backend)");
    println!("cargo:rustc-check-cfg=cfg(rhwp_has_skia_backend)");

    if backend.join("png_adapter.rs").is_file() {
        println!("cargo:rustc-cfg=rhwp_has_png_backend");
    }
    if backend.join("skia_adapter.rs").is_file() {
        println!("cargo:rustc-cfg=rhwp_has_skia_backend");
    }
}
