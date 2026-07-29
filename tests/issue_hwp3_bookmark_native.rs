//! HWP3 책갈피가 저장하면 통째로 사라진다 (#3505 스윕에서 발견).
//!
//! `convert --verify` 가 `hwp3-sample16.hwp` 에서 exit 3 을 냈다.
//!
//! ```text
//! paragraph[70] controls: expected=[field,newNum,pageNumPos] actual=[newNum,pageNumPos]
//! paragraph[73] controls: expected=[field] actual=[]
//! ```
//!
//! ## 근인
//!
//! HWP3 파서가 책갈피를 **HWP3 전용 합성 표현**으로 만들었다.
//!
//! ```text
//! Control::Field { field_type: Unknown, command: "Bookmark:<이름>:type=0" }
//! ```
//!
//! 이 `command` 문자열을 읽는 곳은 저장소 어디에도 없다 — 순수 잔재다. 그런데 HWP5
//! 저장기에는 `FieldType::Unknown` 필드를 쓸 방법이 없어 저장하면 컨트롤이 통째로 사라졌다.
//!
//! 공통 IR 에는 이미 `Control::Bookmark` 가 있고 저장기도 `CTRL_BOOKMARK` 로 완비돼 있다.
//! HWP3 전용 해석을 `src/parser/hwp3/` 안에서 끝내고 공통 표현을 내보내면 그대로 왕복한다.
//! **#3492(개요번호 마커 IR 유출)와 같은 계열**이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;

/// 책갈피 10개가 든 HWP 3.0 문서.
const SAMPLE: &str = "samples/hwp3-sample16.hwp";
const BOOKMARK_COUNT: usize = 10;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// nextest archive 는 런타임에 `CARGO_BIN_EXE_rhwp`를 주입한다(#3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn load(path: &std::path::Path) -> rhwp::model::document::Document {
    let data = std::fs::read(path).expect("파일 읽기");
    rhwp::parser::parse_document(&data).expect("파싱")
}

fn bookmark_names(doc: &rhwp::model::document::Document) -> Vec<String> {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Bookmark(b) => Some(b.name.clone()),
            _ => None,
        })
        .collect()
}

fn convert_roundtrip() -> std::path::PathBuf {
    let out = std::env::temp_dir().join(format!(
        "rhwp-hwp3bm-{}-{}.hwp",
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
    assert_eq!(
        res.status.code(),
        Some(0),
        "convert --verify 가 IR 손실을 보고했다:\n{}",
        String::from_utf8_lossy(&res.stdout)
    );
    out
}

/// 책갈피는 공통 표현(`Control::Bookmark`)으로 나온다 — HWP3 전용 합성 필드가 아니다.
#[test]
fn hwp3_bookmarks_use_the_common_control() {
    let doc = load(&sample_path());
    assert_eq!(
        bookmark_names(&doc).len(),
        BOOKMARK_COUNT,
        "책갈피가 공통 표현으로 나오지 않았다"
    );

    // 합성 표현이 남아 있으면 안 된다.
    let leaked: Vec<String> = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Field(f) if f.command.starts_with("Bookmark:") => Some(f.command.clone()),
            _ => None,
        })
        .collect();
    assert!(
        leaked.is_empty(),
        "HWP3 전용 책갈피 합성 필드가 IR 에 남았다: {leaked:?}"
    );
}

/// 저장해도 책갈피가 이름까지 그대로 살아남는다.
#[test]
fn bookmarks_survive_saving_to_hwp5() {
    let before = bookmark_names(&load(&sample_path()));
    assert_eq!(before.len(), BOOKMARK_COUNT);

    let out = convert_roundtrip();
    let after = bookmark_names(&load(&out));
    let _ = std::fs::remove_file(&out);

    assert_eq!(
        after, before,
        "저장 후 책갈피가 달라졌다 (수정 전에는 통째로 사라졌다)"
    );
}
