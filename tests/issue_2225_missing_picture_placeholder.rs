//! Issue #2225: 그림 미지정 placeholder 컨텍스트 분기.
//!
//! 한컴 동작(작업지시자 확인): 편집기 = 점선 테두리 + 그림-없음 아이콘으로
//! 정보 제공 / 인쇄·인쇄 등가 출력 = 미출력.
//!
//! 재현: 36389312 결재문서 "심볼" 필드(그림 bin_id=0 미지정, 외부 경로 없음).
//! 정정: layout 이 미지정 그림을 Placeholder(MissingPicture) 노드로 방출 —
//! web_canvas(편집 뷰) = 편집기식 표시 / svg·skia(export) = 무방출.

use rhwp::renderer::render_tree::{PlaceholderKind, RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;
#[cfg(feature = "native-skia")]
use std::process::Command;

const SAMPLE: &str =
    "samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx";

fn count_kinds(n: &RenderNode, missing: &mut usize, images: &mut usize) {
    match &n.node_type {
        RenderNodeType::Placeholder(ph) if ph.kind == PlaceholderKind::MissingPicture => {
            *missing += 1;
        }
        RenderNodeType::Image(_) => *images += 1,
        _ => {}
    }
    for c in &n.children {
        count_kinds(c, missing, images);
    }
}

#[test]
fn issue_2225_missing_picture_placeholder_split() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let p = Path::new(repo_root).join(SAMPLE);
    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&fs::read(&p).unwrap()).expect("parse 36389312");
    let tree = doc.build_page_render_tree(0).expect("render p1");

    // 1) 렌더 트리: 미지정 그림 → MissingPicture placeholder (수정 전: 회색 Image).
    let (mut missing, mut images) = (0usize, 0usize);
    count_kinds(&tree.root, &mut missing, &mut images);
    assert!(
        missing >= 1,
        "MissingPicture placeholder 부재 (미지정 '심볼' 필드) — #2225 회귀"
    );
    // 2) 정상 그림(로고 bin_id=1 등)은 Image 노드 유지.
    assert!(images >= 1, "정상 그림 Image 노드가 사라짐 — #2225 과억제");

    // 3) export SVG(인쇄 등가): placeholder 회색 박스 무방출.
    let svg = doc.render_page_svg_native(0).expect("svg p1");
    assert!(
        !svg.contains("#f0f0f0"),
        "export SVG 에 미지정 placeholder 회색 박스가 방출됨 — #2225 회귀"
    );
    // 정상 그림 이미지는 방출 유지.
    assert!(svg.contains("<image"), "export SVG 에서 정상 그림이 사라짐");
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_2225_export_png_defaults_to_print_equivalent_skia_profile() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sample_path = repo_root.join(SAMPLE);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let output_root =
        std::env::temp_dir().join(format!("rhwp-issue-2225-{}-{nonce}", std::process::id()));
    let default_dir = output_root.join("default");
    let screen_dir = output_root.join("screen");

    let default_output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-png")
        .arg(&sample_path)
        .args(["--page", "0", "--output"])
        .arg(&default_dir)
        .output()
        .expect("run default export-png");
    assert!(
        default_output.status.success(),
        "default export-png failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&default_output.stdout),
        String::from_utf8_lossy(&default_output.stderr)
    );

    let screen_output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-png")
        .arg(&sample_path)
        .args(["--page", "0", "--output"])
        .arg(&screen_dir)
        .args(["--profile", "screen"])
        .output()
        .expect("run screen export-png");
    assert!(
        screen_output.status.success(),
        "screen export-png failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&screen_output.stdout),
        String::from_utf8_lossy(&screen_output.stderr)
    );

    let file_name = format!(
        "{}.png",
        sample_path
            .file_stem()
            .and_then(|value| value.to_str())
            .expect("UTF-8 sample stem")
    );
    let default_image = image::open(default_dir.join(&file_name))
        .expect("decode default PNG")
        .to_rgba8();
    let screen_image = image::open(screen_dir.join(&file_name))
        .expect("decode screen PNG")
        .to_rgba8();
    assert!(default_image.width() >= 722 && default_image.height() >= 131);
    assert_eq!(default_image.dimensions(), screen_image.dimensions());

    let count_placeholder_ink = |image: &image::RgbaImage| {
        (55..131)
            .flat_map(|y| (646..722).map(move |x| image.get_pixel(x, y)))
            .filter(|pixel| {
                pixel.0[3] > 0 && (pixel.0[0] < 250 || pixel.0[1] < 250 || pixel.0[2] < 250)
            })
            .count()
    };
    let default_ink = count_placeholder_ink(&default_image);
    let screen_ink = count_placeholder_ink(&screen_image);
    let _ = fs::remove_dir_all(&output_root);

    assert_eq!(
        default_ink, 0,
        "default export-png emitted print-equivalent missingPicture ink"
    );
    assert!(
        screen_ink > 1_000,
        "explicit screen profile did not emit the missingPicture editor visual: {screen_ink}"
    );
}
