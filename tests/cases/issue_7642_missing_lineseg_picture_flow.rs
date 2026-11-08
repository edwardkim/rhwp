//! 저장 줄이 없는 그림의 공통 물리 줄·후속 문단 계약.
//! 독립 Print의 Native/fresh WASM 전쪽 90%와 확대 판독 뒤 추가한다.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn binary() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into())
}
fn capture(name: &str) -> Vec<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root
        .join("tests/fixtures/issue_7642/no-ls")
        .join(format!("{name}.hwp"));
    let out: PathBuf = std::env::temp_dir().join(format!(
        "rhwp-7642-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&out).unwrap();
    let p = Command::new(binary())
        .args([
            "export-render-tree",
            src.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        p.status.success(),
        "CLI 실패: {}",
        String::from_utf8_lossy(&p.stderr)
    );
    let mut files = std::fs::read_dir(&out)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect::<Vec<_>>();
    files.sort();
    let trees = files
        .into_iter()
        .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
        .collect();
    std::fs::remove_dir_all(out).unwrap();
    trees
}
fn rect(node: &Value) -> [f64; 4] {
    let b = &node["bbox"];
    [
        b["x"].as_f64().unwrap(),
        b["y"].as_f64().unwrap(),
        b["w"].as_f64().unwrap(),
        b["h"].as_f64().unwrap(),
    ]
}
fn walk<'a>(
    node: &'a Value,
    owner: Option<&'a Value>,
    out: &mut Vec<(&'a Value, Option<&'a Value>)>,
) {
    let owner = if node["type"] == "TextLine" {
        Some(node)
    } else {
        owner
    };
    out.push((node, owner));
    for c in node["children"].as_array().into_iter().flatten() {
        walk(c, owner, out)
    }
}
fn nodes(trees: &[Value]) -> Vec<(usize, &Value, Option<&Value>)> {
    let mut all = Vec::new();
    for (page, tree) in trees.iter().enumerate() {
        let mut ns = Vec::new();
        walk(tree, None, &mut ns);
        all.extend(ns.into_iter().map(|(n, o)| (page, n, o)));
    }
    all
}
fn images(trees: &[Value]) -> Vec<(usize, &Value, &Value)> {
    let mut ns = nodes(trees)
        .into_iter()
        .filter(|(_, n, _)| n["type"] == "Image" && n["pi"] == 1)
        .map(|(p, n, o)| (p, n, o.expect("그림의 실제 소유 글줄")))
        .collect::<Vec<_>>();
    ns.sort_by_key(|(_, n, _)| n["ci"].as_u64().unwrap());
    ns
}
fn following(trees: &[Value]) -> (usize, &Value) {
    let ns = nodes(trees)
        .into_iter()
        .filter(|(_, n, _)| n["type"] == "TextRun" && n["pi"] == 2 && n["text"] == "다음문단")
        .collect::<Vec<_>>();
    assert_eq!(ns.len(), 1, "후속 본문의 누락·중복");
    (ns[0].0, ns[0].1)
}

// 독립 Print 전체 2쪽과 직접 판독에서 확정한 줄 소속·순서·내용 보존.
#[test]
fn no_lineseg_picture_rows_preserve_ownership_and_content_order() {
    for (key, count, same_row) in [
        ("same-row", 2, true),
        ("wrapped-row", 2, false),
        ("center-row", 2, true),
        ("authored-break", 2, false),
        ("column-transition", 2, true),
        ("mixed-end-picture", 2, true),
        ("outer-margin-row", 2, true),
        ("tail-gap-boundary", 1, true),
        ("tail-gap-only-fit", 1, true),
    ] {
        let t = capture(&format!("picture-flow-{key}-no-ls"));
        assert_eq!(t.len(), 2, "{key}: 독립 Print 쪽수");
        let imgs = images(&t);
        assert_eq!(imgs.len(), count, "{key}: 그림 누락·중복");
        for (ci, (page, img, row)) in imgs.iter().enumerate() {
            assert_eq!(*page, 1, "{key}: 그림 쪽 소속");
            assert_eq!(img["ci"], ci);
            assert_eq!(row["pi"], 1, "{key}: 개체의 문단 소유");
            let [x, y, w, h] = rect(img);
            let [rx, ry, rw, rh] = rect(row);
            assert!(
                x >= rx && y >= ry && x + w <= rx + rw && y + h <= ry + rh,
                "{key}: 자기 줄 상자 안에 있어야 함"
            );
        }
        if count == 2 {
            let (_, a, ar) = imgs[0];
            let (_, b, br) = imgs[1];
            assert_eq!(std::ptr::eq(ar, br), same_row, "{key}: 실제 줄 소속");
            let [ax, ay, aw, ah] = rect(a);
            let [bx, by, _, _] = rect(b);
            if same_row {
                assert!(ax + aw <= bx, "{key}: 앞뒤 그림 겹침")
            } else {
                assert!(ay + ah <= by, "{key}: 줄 순서·겹침")
            }
        }
        let (page, next) = following(&t);
        let (_, _, last) = imgs.last().unwrap();
        assert_eq!(page, 1);
        let [_, ny, _, _] = rect(next);
        let [_, y, _, h] = rect(last);
        assert!(ny >= y + h, "{key}: 다음 문단이 개체 줄을 침범함");
    }
}

#[test]
fn center_row_uses_one_alignment_point_for_text_and_pictures() {
    let t = capture("picture-flow-mixed-center-row-no-ls");
    let imgs = images(&t);
    assert_eq!(imgs.len(), 2);
    let matches = nodes(&t)
        .into_iter()
        .filter(|(_, n, _)| n["type"] == "TextRun" && n["pi"] == 1 && n["text"] == "그림사이")
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "사이 본문 보존");
    let (_, text, owner) = matches[0];
    let owner = owner.unwrap();
    for (_, img, row) in &imgs {
        assert!(
            std::ptr::eq(*row, owner),
            "그림과 사이 글자는 같은 실제 줄이어야 함"
        );
        for item in [*img, text] {
            let [_, y, _, h] = rect(item);
            let [_, ry, _, rh] = rect(owner);
            // JSON bbox의 소수 자리 반올림을 줄 높이에 대한 무차원 비율로만 허용한다.
            assert!(
                ((y + h / 2.0 - ry) / rh - 0.5).abs() < 0.001,
                "CENTER의 정렬점이 서로 다름"
            );
        }
    }
    let [ax, _, aw, _] = rect(imgs[0].1);
    let [tx, _, tw, _] = rect(text);
    let [bx, _, _, _] = rect(imgs[1].1);
    assert!(ax + aw <= tx && tx + tw <= bx, "본문과 그림의 순서·겹침");
    following(&t);
}

#[test]
fn picture_line_spacing_affects_following_gap_and_preserves_picture() {
    let a = capture("normalized-quarter-reflow-false");
    let b = capture("normalized-quarter-reflow-true");
    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 2);
    let ai = images(&a);
    let bi = images(&b);
    assert_eq!(ai.len(), 1);
    assert_eq!(bi.len(), 1);
    assert_eq!(
        rect(ai[0].1),
        rect(bi[0].1),
        "줄 간격 속성이 그림을 이동·확대하지 않음"
    );
    let (_, an) = following(&a);
    let (_, bn) = following(&b);
    let [_, ay, _, ah] = rect(ai[0].2);
    let [_, by, _, bh] = rect(bi[0].2);
    let ag = rect(an)[1] - ay - ah;
    let bg = rect(bn)[1] - by - bh;
    assert!(ag > 0.0 && bg > ag, "줄 간격 영향 설정의 독립 Print 관계");
    // 같은 그림의 한컴 정상 저장본이 기록한 줄 메트릭이 독립 기대값이다.
    // 수정 구현의 계산값이나 출력 좌표를 기대값으로 복사하지 않는다.
    let saved_row = |flag: &str| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/issue_7642/relative-quarter-affect-{flag}-hancom-2020.hwp"
        ));
        let doc = rhwp::parser::parse_document(&std::fs::read(path).unwrap()).unwrap();
        doc.sections[0].paragraphs[1].line_segs[0].clone()
    };
    let off = saved_row("false");
    let on = saved_row("true");
    assert_eq!(off.line_height, on.line_height);
    let normalized_extra = (bg - ag) / bh;
    let expected = f64::from(on.line_spacing - off.line_spacing) / f64::from(on.line_height);
    assert!(
        (normalized_extra - expected).abs() < 0.001,
        "개체 간격의 비율 계약"
    );
}

#[test]
fn relative_display_size_preserves_declared_line_box() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path =
        root.join("tests/fixtures/issue_7642/no-ls/picture-flow-relative-font-row-no-ls.hwp");
    let saved = rhwp::parser::parse_document(&std::fs::read(path).unwrap()).unwrap();
    let p = &saved.sections[0].paragraphs[1];
    let rhwp::model::control::Control::Picture(picture) = &p.controls[0] else {
        panic!("독립 저장본의 그림")
    };
    // 한컴 정상 저장/Print가 확인한 선언 크기 계약을 원본 속성으로 검사한다.
    let cs = &saved.doc_info.char_shapes[p.char_shapes[0].char_shape_id as usize];
    let expected = f64::from(cs.base_size) / f64::from(picture.common.width);
    let t = capture("picture-flow-relative-font-row-no-ls");
    let img = images(&t);
    assert_eq!(img.len(), 1);
    let [_, _, width, _] = rect(img[0].1);
    let [_, _, _, height] = rect(img[0].2);
    assert!(
        (height / width - expected).abs() < 0.001,
        "표시 크기로 줄 상자가 확대됨"
    );
    following(&t);
}
