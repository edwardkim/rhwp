//! [#4132] native-skia `export-png` CLI 종료 코드 계약.
//!
//! 파일 전체를 native-skia로 게이트해 Native Skia job·classifier의 파일 게이트
//! 규약이 이 target의 배선을 자동으로 강제한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

#[path = "support/cli_exit_code_support.rs"]
mod cli_exit_code_support;

use cli_exit_code_support::{assert_code, unique_temp_path};

#[test]
fn export_png_follows_the_same_contract() {
    let missing = unique_temp_path("missing-png.hwp");
    let missing = missing.to_str().expect("utf-8 경로").to_string();
    let out_dir = unique_temp_path("png-out");
    let out_dir = out_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-png"], 2);
    assert_code(&["export-png", &missing, "-o", &out_dir], 1);
}
