//! 그림 앵커·TAC 줄·뒤 본문의 소속을 독립 한컴 Print와 대조한다.
//! 같은 입력의 전쪽 Native224/fresh063 90% 이상과 직접 판독을 근거로 추가한다.
//! ±2 CSSpx는 두 경로 98% 이상인 해당 페이지의 독립 PDF 좌표에만 적용한다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command, time::SystemTime};

const ASSETS: &str = "mydocs/pr/assets/issue_6133_mixed_host_line_20261025";

fn scalar(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().unwrap()
}
fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && *c > '\u{001f}')
        .collect()
}
fn text(node: &Value) -> String {
    let mut result = if node["type"] == "TextRun" {
        node["text"].as_str().unwrap_or_default().to_owned()
    } else {
        String::new()
    };
    for child in node["children"].as_array().into_iter().flatten() {
        result.push_str(&text(child));
    }
    result
}
fn collect<'a>(node: &'a Value, page: usize, out: &mut Vec<(usize, &'a Value)>) {
    out.push((page, node));
    for child in node["children"].as_array().into_iter().flatten() {
        collect(child, page, out);
    }
}
fn render(input: &str, key: &str) -> Vec<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent = root.join("output/test-runs");
    fs::create_dir_all(&parent).unwrap();
    let output = parent.join(format!(
        "mixed-host-{key}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&output).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-render-tree")
        .arg(root.join(input))
        .args(["--compat", "2022", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{key}: CLI 실패, 증적 {}: {}",
        output.display(),
        String::from_utf8_lossy(&result.stderr)
    );
    let mut paths: Vec<_> = fs::read_dir(&output)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("render_tree_")
                && p.extension().is_some_and(|ext| ext == "json")
        })
        .collect();
    paths.sort();
    let pages = paths
        .iter()
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
        .collect();
    fs::remove_dir_all(&output).unwrap();
    pages
}
fn close(actual: f64, expected: f64, key: &str, item: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{key}/{item}: 독립 Print {expected}, 실제 {actual}, 허용 ±2 CSSpx"
    );
}
fn verify(key: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data: Value = serde_json::from_slice(
        &fs::read(root.join(ASSETS).join("independent_coordinates.json")).unwrap(),
    )
    .unwrap();
    let case = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["key"] == key)
        .unwrap();
    for kind in ["input", "pdf"] {
        let bytes = fs::read(root.join(case[kind].as_str().unwrap())).unwrap();
        let digest = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            digest,
            case[format!("{kind}_sha256")],
            "{key}: 동일 원본/Print 바이트"
        );
    }
    let pages = render(case["input"].as_str().unwrap(), key);
    assert_eq!(
        pages.len(),
        case["page_count"].as_u64().unwrap() as usize,
        "{key}: 독립 Print의 전쪽 수"
    );
    let mut nodes = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        collect(page, index + 1, &mut nodes);
    }
    let primary = case["para_index"].as_u64().unwrap();
    let object = |kind: &str, control: &str| {
        let matches: Vec<_> = nodes
            .iter()
            .filter(|(_, n)| {
                n["type"] == kind
                    && n["pi"].as_u64() == Some(primary)
                    && n["ci"].as_u64() == case[control].as_u64()
            })
            .copied()
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "{key}/{kind}: 원문 개체의 누락·중복 없이 한 번만 소유"
        );
        matches[0]
    };
    let (table_page, table) = object("Table", "table_control");
    let (picture_page, picture) = object("Image", "picture_control");
    assert_eq!(
        table_page,
        case["table_page"].as_u64().unwrap() as usize,
        "{key}: TAC 소유 쪽"
    );
    assert_eq!(
        picture_page,
        case["picture_page"].as_u64().unwrap() as usize,
        "{key}: 그림 소유 쪽"
    );
    close(
        scalar(table, "x"),
        case["table_x"].as_f64().unwrap(),
        key,
        "TAC x",
    );
    close(
        scalar(table, "y"),
        case["table_y"].as_f64().unwrap(),
        key,
        "TAC y",
    );
    assert_eq!(
        normalized(&text(table)),
        "붙임2행사포스터",
        "{key}: 셀의 내용·순서 보존"
    );
    if let Some(expected) = case["picture_css_bbox"].as_array() {
        // 독립 PDF의 두 이미지 띠도 실제 clip을 적용한 뒤 합집합으로 읽었다.
        close(
            scalar(picture, "x"),
            expected[0].as_f64().unwrap(),
            key,
            "그림 왼쪽",
        );
        close(
            scalar(picture, "y"),
            expected[1].as_f64().unwrap(),
            key,
            "그림 위쪽",
        );
        close(
            scalar(picture, "x") + scalar(picture, "w"),
            expected[2].as_f64().unwrap(),
            key,
            "그림 오른쪽",
        );
        close(
            scalar(picture, "y") + scalar(picture, "h"),
            expected[3].as_f64().unwrap(),
            key,
            "그림 아래쪽",
        );
    }
    let table_bottom = scalar(table, "y") + scalar(table, "h");
    let picture_bottom = scalar(picture, "y") + scalar(picture, "h");
    if table_page == picture_page {
        assert!(
            table_bottom <= scalar(picture, "y") + 2.0
                || picture_bottom <= scalar(table, "y") + 2.0,
            "{key}: 같은 쪽의 제목과 그림은 겹치지 않는다"
        );
    }
    let paper = &pages[picture_page - 1];
    assert!(
        scalar(picture, "y") >= -2.0 && picture_bottom <= scalar(paper, "h") + 2.0,
        "{key}: 이월 그림은 본문 하단으로 끌어올리지 않고 실제 용지 안에 보존한다"
    );
    if let Some(after_page) = case["after_page"].as_u64() {
        let after: Vec<_> = nodes
            .iter()
            .filter(|(_, n)| n["type"] == "TextRun" && n["pi"].as_u64() == Some(primary + 1))
            .collect();
        let actual_text: String = after
            .iter()
            .map(|(_, n)| n["text"].as_str().unwrap_or_default())
            .collect();
        assert_eq!(
            normalized(&actual_text),
            "AFTER-MIXED-HOST",
            "{key}: 뒤 문단의 소유·내용 순서·중복 확인"
        );
        assert!(
            after.iter().all(|(page, _)| *page == after_page as usize),
            "{key}: 뒤 문단의 독립 Print 쪽 소속"
        );
        let after_top = after
            .iter()
            .map(|(_, n)| scalar(n, "y"))
            .reduce(f64::min)
            .unwrap();
        if after_page as usize == table_page {
            // 저장 원본의 줄간격은 TAC 뒤에서 소비한다. 그림 끝으로 문단을
            // 다시 닫아 이 간격을 없애면 Native214의 98.60% 반례를 검출한다.
            let gap = case["source_table_gap_hu"].as_f64().unwrap() * 96.0 / 7200.0;
            assert!(
                after_top >= table_bottom + gap - 2.0,
                "{key}: 표 뒤 원본 줄간격 보존: {after_top} < {}",
                table_bottom + gap
            );
        }
        if after_page as usize == picture_page {
            assert!(
                after_top >= picture_bottom - 2.0,
                "{key}: 뒤 본문이 그림의 물리 띠를 다시 침범하지 않는다"
            );
        }
    }
}

#[test]
fn verified_offset_0_picture_first_coherent() {
    verify("offset-0-picture-first-coherent");
}

#[test]
fn verified_offset_0_picture_first_coherent_successor() {
    verify("offset-0-picture-first-coherent-successor");
}

#[test]
fn verified_offset_1200_picture_first_coherent() {
    verify("offset-1200-picture-first-coherent");
}

#[test]
fn verified_offset_1200_table_first_coherent() {
    verify("offset-1200-table-first-coherent");
}

#[test]
fn verified_offset_2831_picture_first_coherent() {
    verify("offset-2831-picture-first-coherent");
}

#[test]
fn verified_offset_3518_picture_first_coherent() {
    verify("offset-3518-picture-first-coherent");
}

#[test]
fn verified_offset_3518_table_first_coherent() {
    verify("offset-3518-table-first-coherent");
}

#[test]
fn verified_offset_3518_table_first_coherent_successor() {
    verify("offset-3518-table-first-coherent-successor");
}

#[test]
fn verified_offset_negative_1200_picture_first_coherent() {
    verify("offset-negative-1200-picture-first-coherent");
}

#[test]
fn verified_original_6133_full() {
    verify("original-6133-full");
}

#[test]
fn verified_original_6133_slice() {
    verify("original-6133-slice");
}

#[test]
fn verified_picture_height_60000_offset_3518_coherent() {
    verify("picture-height-60000-offset-3518-coherent");
}

#[test]
fn verified_picture_height_60000_offset_3518_coherent_successor() {
    verify("picture-height-60000-offset-3518-coherent-successor");
}

#[test]
fn verified_picture_height_62000_offset_3518_coherent() {
    verify("picture-height-62000-offset-3518-coherent");
}

#[test]
fn verified_picture_height_63000_offset_3518_coherent() {
    verify("picture-height-63000-offset-3518-coherent");
}

#[test]
fn verified_picture_height_65000_offset_3518_coherent() {
    verify("picture-height-65000-offset-3518-coherent");
}

#[test]
fn verified_picture_height_68200_offset_3518_coherent() {
    verify("picture-height-68200-offset-3518-coherent");
}

#[test]
fn verified_picture_height_68200_offset_3518_coherent_successor() {
    verify("picture-height-68200-offset-3518-coherent-successor");
}
