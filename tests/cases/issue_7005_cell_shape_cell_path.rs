#![cfg(not(target_arch = "wasm32"))]

//! [#7005] 표 칸 안 도형도 레이아웃에 `cellPath` 를 실어야 한다.
//!
//! studio 는 `cellPath` 유무로 by-path API 와 본문 API 를 가른다. 칸 안 도형에 이 값이 없으면
//! `getObjectProperties` 가 본문 `getShapeProperties(sec, ppi, ci)` 로 떨어져 "지정된 컨트롤이
//! Shape이 아닙니다" 로 던지고(그 자리가 마우스 리사이즈 진입부라 조작이 조용히 막힌다),
//! 선택 토글도 `cellPath` 로 개체를 구분하므로 같은 `(sec, ppi, ci)` 를 쓰는 칸 안 도형 둘을
//! 같은 개체로 본다.
//!
//! 표본 `21_언어_기출_편집가능본.hwp` 1쪽에는 칸 안 도형 둘(「제 1 교시」·「홀수형」)이 있고
//! 둘 다 `(sec, ppi, ci) = (0, 0, 0)` 이며 칸만 다르다 — `cellPath` 없이는 구분되지 않는다.

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/21_언어_기출_편집가능본.hwp";

fn page0_controls() -> Vec<serde_json::Value> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(p).expect("표본 로드")).expect("파싱");
    let json = core
        .get_page_control_layout_native(0)
        .expect("0쪽 레이아웃");
    let v: serde_json::Value = serde_json::from_str(&json).expect("레이아웃 JSON");
    v["controls"].as_array().expect("controls 배열").clone()
}

/// 칸 안(= `cellIdx` 를 가진) 컨트롤만.
fn in_cell(controls: &[serde_json::Value], ty: &str) -> Vec<serde_json::Value> {
    controls
        .iter()
        .filter(|c| c["type"] == ty && !c["cellIdx"].is_null())
        .cloned()
        .collect()
}

#[test]
fn cell_shapes_carry_cell_path() {
    let controls = page0_controls();
    let shapes = in_cell(&controls, "shape");
    assert!(
        shapes.len() >= 2,
        "표본 전제: 1쪽에 칸 안 도형이 둘 이상 (실측 {})",
        shapes.len()
    );

    for s in &shapes {
        let path = s["cellPath"].as_array().unwrap_or_else(|| {
            panic!("칸 안 도형에 cellPath 가 없다 — studio 가 본문 API 로 떨어진다: {s}")
        });
        assert_eq!(path.len(), 1, "평평한 3필드는 1단계 경로다: {s}");
        // 경로 항목이 같은 컨트롤의 스칼라와 일치해야 by-path 조회가 성립한다.
        assert_eq!(path[0]["controlIndex"], s["outerTableControlIdx"], "{s}");
        assert_eq!(path[0]["cellIndex"], s["cellIdx"], "{s}");
        assert_eq!(path[0]["cellParaIndex"], s["cellParaIdx"], "{s}");
    }
}

#[test]
fn cell_shapes_are_distinguishable_by_cell_path() {
    // studio `togglePictureObjectSelection` 은 (sec, ppi, ci, cellPath) 로 동일성을 본다.
    // 이 표본의 두 도형은 앞 셋이 같으므로 cellPath 가 갈라 주어야 한다.
    let controls = page0_controls();
    let shapes = in_cell(&controls, "shape");
    let ident: Vec<String> = shapes
        .iter()
        .map(|s| {
            format!(
                "{}/{}/{}/{}",
                s["secIdx"], s["paraIdx"], s["controlIdx"], s["cellPath"]
            )
        })
        .collect();
    let mut uniq = ident.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(
        uniq.len(),
        ident.len(),
        "칸 안 도형들이 선택 동일성 판정에서 구분되지 않는다: {ident:?}"
    );

    let bare: Vec<String> = shapes
        .iter()
        .map(|s| format!("{}/{}/{}", s["secIdx"], s["paraIdx"], s["controlIdx"]))
        .collect();
    let mut bare_uniq = bare.clone();
    bare_uniq.sort();
    bare_uniq.dedup();
    assert!(
        bare_uniq.len() < bare.len(),
        "표본 전제: cellPath 없이는 두 도형이 같은 좌표로 겹쳐야 판정이 의미를 갖는다: {bare:?}"
    );
}

#[test]
fn body_shapes_keep_no_cell_fields() {
    // 대조 — 본문 직속 도형은 종전대로 칸 필드가 없어야 한다(빈 문자열 방출).
    let controls = page0_controls();
    let body: Vec<_> = controls
        .iter()
        .filter(|c| (c["type"] == "shape" || c["type"] == "line") && c["cellIdx"].is_null())
        .collect();
    assert!(
        !body.is_empty(),
        "표본 전제: 1쪽에 본문 직속 도형/직선이 있어야 대조가 선다"
    );
    for c in body {
        assert!(
            c["cellPath"].is_null(),
            "본문 도형에 cellPath 가 붙었다: {c}"
        );
        assert!(c["outerTableControlIdx"].is_null(), "{c}");
    }
}
