//! Issue #3528: 표 캡션 안에 표가 있으면 저장 시 캡션 문단이 잘린다.
//!
//! ## 근인
//!
//! HWP5 저장 순서는 `CTRL_TABLE → 캡션(LIST_HEADER + 문단) → HWPTAG_TABLE → 셀` 이다.
//! 파서는 그 순서를 이용해 **첫 `HWPTAG_TABLE` 앞까지**를 캡션으로 잘랐다.
//!
//! 그런데 캡션 문단 **안에 표가 들어 있으면** 그 표가 자기 `HWPTAG_TABLE` 을 더 앞에
//! 방출한다. 그러면 경계 판정이 바깥 표의 것이 아니라 **캡션 속 표의 것**을 집어, 캡션
//! 문단이 잘리고 그 안의 표도 얕게 읽힌다.
//!
//! 해법은 **직계 자식 레벨의 `HWPTAG_TABLE` 만** 경계로 삼는 것이다. 캡션 LIST_HEADER ·
//! HWPTAG_TABLE · 셀 LIST_HEADER 가 모두 같은 레벨이므로, 자식 레코드의 최소 레벨이 곧
//! 직계 레벨이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

/// 캡션 안에 표가 든 문서 (깊이 3 중첩).
const SAMPLE: &str = "samples/issue1891_external_bindata_link.hwpx";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// nextest archive 는 런타임에 `CARGO_BIN_EXE_rhwp`를 주입한다(#3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

/// 문서 전체에서 "캡션 문단 안에 표를 가진" 표를 찾아 (캡션 문단 수) 목록을 만든다.
fn captions_holding_tables(doc: &rhwp::model::document::Document) -> Vec<usize> {
    fn walk(paras: &[Paragraph], out: &mut Vec<usize>) {
        for p in paras {
            for c in &p.controls {
                let Control::Table(t) = c else { continue };
                if let Some(caption) = &t.caption {
                    let holds_table = caption
                        .paragraphs
                        .iter()
                        .any(|cp| cp.controls.iter().any(|cc| matches!(cc, Control::Table(_))));
                    if holds_table {
                        out.push(caption.paragraphs.len());
                    }
                    walk(&caption.paragraphs, out);
                }
                for cell in &t.cells {
                    walk(&cell.paragraphs, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    for s in &doc.sections {
        walk(&s.paragraphs, &mut out);
    }
    out
}

fn load(path: &std::path::Path) -> rhwp::model::document::Document {
    let data = std::fs::read(path).expect("파일 읽기");
    rhwp::parser::parse_document(&data).expect("파싱")
}

/// 표를 품은 캡션이 저장 후에도 문단 수를 유지한다.
#[test]
fn caption_holding_a_table_keeps_its_paragraphs() {
    let before = captions_holding_tables(&load(&sample_path()));
    assert!(
        !before.is_empty(),
        "캡션 안에 표를 가진 표를 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );

    let out = std::env::temp_dir().join(format!(
        "rhwp-issue3528-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let res = std::process::Command::new(rhwp_bin())
        .arg("convert")
        .arg(sample_path())
        .arg(&out)
        .arg("--verify")
        .output()
        .expect("rhwp 실행");
    let stdout = String::from_utf8_lossy(&res.stdout).to_string();
    let after = if out.exists() {
        captions_holding_tables(&load(&out))
    } else {
        Vec::new()
    };
    let _ = std::fs::remove_file(&out);

    assert_eq!(
        after, before,
        "저장 후 캡션 문단 수가 달라졌다 (수정 전 3→1 로 잘렸다)"
    );
    assert_eq!(
        res.status.code(),
        Some(0),
        "convert --verify 가 IR 손실을 보고했다:\n{stdout}"
    );
}
