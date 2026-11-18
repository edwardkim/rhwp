//! 관계형: Native 경로 글꼴 공급이 앞선 큰 글꼴 파일 때문에 뒤 경로의 face 를 잃지 않는다.
//! 주석 작성: 2026-11-05 06:40 KST (새 파일). 측정: 2026-11-05 KST, ai101.
//!
//! # 측정 근거
//!
//! - 이 파일은 글꼴 공급 계약(합성 글꼴 디렉터리)만 검사한다. 한/글 배치나 픽셀의 기준이 아니다.
//! - 시각 영향 측정: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 판정값
//!   `silhouette_raw_match_percent`. 글꼴은 `--font-path`·`RHWP_FONT_PATH` 로 검증 글꼴 폴더
//!   `ttfs/hwp`·`ttfs/windows` 만 그대로 넘겼다(우선 글꼴 폴더 없음). 수정 전 = devel e96b500e3e,
//!   수정 후 = 3d194ae876(이 파일의 clippy 보정 전 code head, 렌더링 코드 동일).
//! - `samples/basic/shortcut.hwp` 전체 7쪽(PDF `pdf/basic/shortcut-hwp-2020.pdf` 7쪽):
//!   수정 후 7쪽 Native 99.99% · fresh WASM 99.97%, 전쪽 최저 Native 97.19%(3쪽) ·
//!   fresh WASM 97.26%(3쪽). 수정 전 Native 7쪽 82.08%, 전쪽 최저 82.08%(90% 미만 3개 쪽).
//! - `samples/basic/sungeo.hwp` 전체 94쪽(PDF 94쪽): 수정 후 1쪽 Native 100.00% · fresh WASM 100.00%,
//!   전쪽 최저 Native 99.90% · fresh WASM 99.96%. 수정 전 Native 1쪽 98.61%(전쪽 최저).
//! - 편람 `--handbook-gate` 383쪽 Native: 수정 전 `re_review_required`(6쪽 99.97~99.99),
//!   수정 후 `passed`.
//! - 증거: `mydocs/pr/assets/issue_8597/{native,wasm}/*_silhouette.tsv`,
//!   `mydocs/pr/assets/issue_8597/png/`.
//!
//! # 무엇이 깨져 있었나
//!
//! `TypefaceCatalog::load` 는 `--font-path`·`RHWP_FONT_PATH` 의 파일을 힙에 복사하면서
//! 전체 64MiB 를 넘는 파일을 경고 없이 건너뛰었다. 검증용 글꼴 폴더를 넘기면 앞쪽
//! 파일만 실리고 뒤쪽의 함초롬·맑은 고딕·윤고딕 등이 빠졌다(#8597).
//! 수정 후 경로 파일은 파일 매핑을 face 끼리 공유하므로 64MiB 힙 복사 상한에 들지
//! 않는다. 파일 하나의 크기 상한(32MiB)과 파일 수 상한(256개)은 그대로다.
//!
//! # 기대값의 출처
//!
//! 사용자가 지정한 경로의 글꼴은 경로 순서와 무관하게 상한 안의 파일이면 모두 공급된다는
//! 기존 공급 계약(`font_paths::custom_font_dirs`, `native_supplied_font_family_names`)에서
//! 정한다. 반례로 파일 하나 상한을 넘는 파일은 여전히 공급되지 않아야 한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::DocumentCore;

const PRIMARY: &str = "Rhwp Family Contract";
const MIB: usize = 1024 * 1024;
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        loop {
            let serial = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "rhwp-issue8597-fonts-{}-{serial}",
                std::process::id()
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create isolated font directory: {error}"),
            }
        }
    }

    /// 유효한 글꼴 뒤에 0 을 덧붙여 정확히 `size` 바이트인 파일을 쓴다.
    fn padded(&self, name: &str, program: &[u8], size: usize) -> PathBuf {
        assert!(program.len() <= size);
        let mut bytes = program.to_vec();
        bytes.resize(size, 0);
        let path = self.0.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn filler_program() -> Vec<u8> {
    std::fs::read(repo("ttfs/opensource/RHWPPublicBatang-Regular.ttf"))
        .expect("저장소 OFL 글꼴(ttfs/opensource)은 모든 체크아웃에 있다")
}

fn target_program() -> Vec<u8> {
    std::fs::read(repo(
        "tests/fixtures/native_font_family_names/family-contract.ttf",
    ))
    .expect("family contract fixture")
}

/// 문서 글꼴을 `requested` 로 바꾼 한 글자 「A」의 Native paint 결정.
fn native_decision(font_paths: &[PathBuf]) -> serde_json::Value {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public document supplies valid section and style defaults");
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = "A".into();
    paragraph.char_count = 2;
    paragraph.char_offsets = vec![0];
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    paragraph.line_segs.clear();
    doc.sections[0].paragraphs = vec![paragraph];
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.name = PRIMARY.into();
        font.is_embedded = false;
        font.alt_name = None;
        font.default_name = None;
        font.subst_font = None;
    }
    core.set_document(doc);
    let renderer = SkiaLayerRenderer::new().with_font_paths(font_paths);
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":64}"#)
            .expect("renderer-bound font observation"),
    )
    .expect("trace JSON");
    let records: Vec<_> = trace["records"]
        .as_array()
        .expect("font records")
        .iter()
        .filter(|record| record["source"]["character"] == "A")
        .collect();
    assert_eq!(records.len(), 1, "one synthetic source character");
    records[0]["paint"]["native"].clone()
}

#[test]
fn later_path_face_survives_64mib_of_earlier_font_files() {
    // 앞 경로: 파일 하나 상한(32MiB) 안의 유효한 글꼴 두 개 = 정확히 64MiB.
    // 예전 적재는 여기서 총량 64MiB 를 다 써서 뒤 경로의 작은 글꼴을 버렸다.
    let filler = Directory::new();
    let program = filler_program();
    filler.padded("a-filler.ttf", &program, 32 * MIB);
    filler.padded("b-filler.ttf", &program, 32 * MIB);
    let target = repo("tests/fixtures/native_font_family_names");

    let native = native_decision(&[filler.0.clone(), target]);
    assert_eq!(native["status"], "complete");
    assert_eq!(
        native["resolved"], PRIMARY,
        "뒤 경로가 공급한 face 가 실린다: {native}"
    );
    assert_eq!(native["source"], "custom");
}

#[test]
fn file_over_the_per_file_bound_is_still_not_supplied() {
    // 반례: 파일 하나의 크기 상한은 유지한다. 같은 프로그램을 상한 + 1 바이트로 만든다.
    let oversized = Directory::new();
    oversized.padded("family-contract.ttf", &target_program(), 32 * MIB + 1);

    let native = native_decision(std::slice::from_ref(&oversized.0));
    assert_ne!(
        native["source"], "custom",
        "32MiB 를 넘는 파일은 공급하지 않는다: {native}"
    );
    assert_ne!(native["resolved"], PRIMARY);
}

#[test]
fn same_program_within_the_bound_is_supplied() {
    // 대조군: 같은 프로그램이 상한과 같은 크기(32MiB)면 공급된다.
    let bounded = Directory::new();
    bounded.padded("family-contract.ttf", &target_program(), 32 * MIB);

    let native = native_decision(std::slice::from_ref(&bounded.0));
    assert_eq!(native["resolved"], PRIMARY, "{native}");
    assert_eq!(native["source"], "custom");
}
