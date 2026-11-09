//! 관계형: 시스템 스타일 목록의 대체 계열이 요청한 실제 글꼴 프로그램을 대신하지 않는다.
//! 주석 갱신: 2026-11-08T21:00:56+09:00; 직전 변경: 2026-11-08T17:16:11+09:00, 57827f09aebeb97e9e347e3940b5dd568177a4c6.
//! 합성 이름 선택 계약으로서 조판 좌표·쪽수·golden을 고정하지 않는다. raw·DPI·독립 PDF는 비해당이다.
//! Linux 시스템 선택은 별도 Fontconfig로, 공통 선택은 Linux·Mac·Windows의 Native backend에
//! GUST/OFL 확인 글꼴 두 개를 명시 공급해 검사한다. 호스트의 한컴·개인 글꼴 설치에 의존하지 않는다.
//! 수정 전 공개 Native 관측은 요청 CFF 대신 다른 family의 TTF를 골라 실패했다.
//! 근거: mydocs/pr/assets/planet6897_20261106/native-weight-r46g-analysis.md.
#![cfg(feature = "native-skia")]

use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::DocumentCore;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

const REQUESTED: &str = "Latin Modern Math";
const OTHER: &str = "Rhwp Family Contract";
#[cfg(target_os = "linux")]
const CHILD: &str = "RHWP_FONTCONFIG_STYLE_CONTRACT_CHILD";

#[cfg(target_os = "linux")]
struct Directory(PathBuf);

#[cfg(target_os = "linux")]
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("이 검사만 만든 글꼴·cache 디렉터리 정리");
    }
}

#[cfg(target_os = "linux")]
fn escaped(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn observe(renderer: &SkiaLayerRenderer, bold: bool, requested: &str) -> serde_json::Value {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("공개 원본의 정상 구역·스타일 기본값");
    let mut document = core.document().clone();
    document.sections.truncate(1);
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = "A".into();
    paragraph.char_count = 2;
    paragraph.char_offsets = vec![0];
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    document.sections[0].paragraphs = vec![paragraph];
    for font in document.doc_info.font_faces.iter_mut().flatten() {
        font.name = requested.into();
        font.is_embedded = false;
        font.alt_name = None;
        font.default_name = None;
        font.subst_font = None;
    }
    document.doc_info.char_shapes[0].bold = bold;
    core.set_document(document);
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":64}"#)
            .expect("현재 renderer의 실제 선택"),
    )
    .expect("공개 추적 JSON");
    let records: Vec<_> = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| record["source"]["character"] == "A")
        .collect();
    assert_eq!(records.len(), 1, "한 문자 원본의 실제 선택 한 건");
    records[0]["paint"]["native"].clone()
}

#[test]
#[cfg(target_os = "linux")]
fn system_style_list_keeps_requested_family_before_other_truetype() {
    if std::env::var_os(CHILD).is_some() {
        // 각 자식은 새 FontMgr를 만들므로 다른 검사의 전역 글꼴 cache에 기대지 않는다.
        let families: Vec<_> = skia_safe::FontMgr::default().family_names().collect();
        assert!(families.iter().any(|family| family == REQUESTED));
        assert!(families.iter().any(|family| family == OTHER));
        assert!(
            families.iter().all(|family| matches!(
                family.as_str(),
                REQUESTED
                    | OTHER
                    | "RHWP LMM Variable Outline"
                    | "검사용 WWS 가족"
                    | "검사용 표기 가족"
                    | "검사용 가족 이름"
            )),
            "명시 공급한 두 프로그램만 노출: {families:?}"
        );
        for bold in [false, true] {
            for requested in [REQUESTED, OTHER] {
                let native = observe(&SkiaLayerRenderer::new(), bold, requested);
                assert_eq!(native["status"], "complete");
                assert_eq!(native["source"], "system");
                assert_eq!(
                    native["resolved"], requested,
                    "TTF 우선은 같은 계열 안에서만 적용하며 다른 계열의 요청도 보존한다"
                );
                assert_eq!(native["failures"], serde_json::json!([]));
            }
        }
        return;
    }

    let path = std::env::temp_dir().join(format!(
        "rhwp-system-family-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(path);
    let fonts = directory.0.join("fonts");
    let cache = directory.0.join("cache");
    std::fs::create_dir(&fonts).unwrap();
    std::fs::create_dir(&cache).unwrap();
    std::fs::write(
        fonts.join("requested.otf"),
        include_bytes!("../../assets/fonts/LatinModernMath-VariableOutline-subset.otf"),
    )
    .unwrap();
    std::fs::write(
        fonts.join("other.ttf"),
        include_bytes!("../fixtures/native_font_family_names/family-contract.ttf"),
    )
    .unwrap();
    let config = directory.0.join("fonts.conf");
    std::fs::write(&config, format!(r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig><dir>{}</dir><cachedir>{}</cachedir>
<match target="pattern"><test name="family" compare="eq"><string>{REQUESTED}</string></test>
<edit name="family" mode="prepend" binding="strong"><string>{OTHER}</string></edit></match>
<match target="pattern"><test name="family" compare="eq"><string>{REQUESTED}</string></test>
<edit name="family" mode="prepend_first" binding="strong"><string>{REQUESTED}</string></edit></match>
</fontconfig>"#, escaped(&fonts.to_string_lossy()), escaped(&cache.to_string_lossy()))).unwrap();
    // 실제 재현 환경의 두 pattern 규칙을 공개 프로그램 이름으로 바꾼 독립 대조군이다.
    // 전체 하네스를 재호출하지 않고 현재 검사 한 건만 별도 프로세스에서 실행한다.
    let name = std::thread::current().name().unwrap().to_owned();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &name, "--nocapture"])
        .env(CHILD, "1")
        .env("FONTCONFIG_FILE", &config)
        .env_remove("FONTCONFIG_PATH")
        .env_remove("RHWP_FONT_PATH")
        .output()
        .expect("격리된 시스템 글꼴 계약 실행");
    assert!(
        output.status.success(),
        "격리된 자식 검사 실패:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn supplied_style_list_keeps_requested_family_before_other_truetype_on_each_native_os() {
    // CoreText·DirectWrite에서도 실제 Native 요청과 resolved family를 관측한다.
    // Fontconfig의 pattern 규칙 자체를 타 OS에서 실행했다고 주장하지 않는다.
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[
            include_bytes!("../../assets/fonts/LatinModernMath-VariableOutline-subset.otf")
                .to_vec(),
            include_bytes!("../fixtures/native_font_family_names/family-contract.ttf").to_vec(),
        ])
        .unwrap();
    for bold in [false, true] {
        for requested in [REQUESTED, OTHER] {
            let native = observe(&renderer, bold, requested);
            assert_eq!(native["status"], "complete");
            assert_eq!(native["source"], "custom");
            assert_eq!(
                native["resolved"], requested,
                "TTF 형식 우선으로 다른 계열의 CFF 요청을 바꾸지 않는다"
            );
            assert_eq!(native["failures"], serde_json::json!([]));
        }
    }
}
