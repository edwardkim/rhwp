//! Issue #1785: 셀 안 여백 선택 규칙은 레이아웃과 높이 측정이 단일 출처를 공유해야 한다.
//!
//! height_measurer 가 단순 `aim ? cell : table` 규칙을 쓰는 동안 레이아웃
//! (resolve_cell_padding)은 레거시 보존값 규칙(aim=false 여도 cell > table 이면 셀 값)을
//! 써서, aim=false + table.padding=0 + cell.padding=140 인 micro-grid 결재란 표에서
//! 예약 높이(측정)와 실제 렌더가 어긋났다. HWPX→HWP 어댑터의 width_ref bit0 계약이
//! aim 을 true 로 물질화하면 재파스본만 측정이 달라져 라운드트립 렌더가 9.3px 틀어진다
//! (36381023 발신명의 표: HWPX 244.9px vs 재파스 254.2px).
//!
//! 수정: 규칙 본체를 `Cell::use_cell_padding_axis`/`effective_padding` 으로 추출하고
//! 두 소비자가 공유. 이후 원본/라운드트립 렌더가 일치한다 (render-diff PASS).

use std::fs;
use std::path::Path;

use rhwp::model::table::Cell;
use rhwp::model::Padding;

const SAMPLE: &str = "samples/task1772/table_outer_margin_common_sync.hwpx";

fn pad(v: i16) -> Padding {
    Padding {
        left: v,
        right: v,
        top: v,
        bottom: v,
    }
}

#[test]
fn issue_1785_effective_padding_rule() {
    // aim=false + 레거시 보존값(cell > table, < 2500) → 셀 값 사용
    let mut cell = Cell {
        padding: pad(140),
        ..Default::default()
    };
    cell.apply_inner_margin = false;
    assert_eq!(cell.effective_padding(&pad(0)).top, 140);

    // aim=false + cell <= table → 표 기본값
    assert_eq!(cell.effective_padding(&pad(140)).top, 140);
    assert_eq!(cell.effective_padding(&pad(200)).top, 200);

    // aim=false + 10mm급(>=2500) 보존값 → 한컴은 렌더에 쓰지 않음 → 표 기본값
    cell.padding = pad(2834);
    assert_eq!(cell.effective_padding(&pad(0)).top, 0);

    // aim=true → 셀 값. [#2070] 0 도 사용자가 지정한 셀 고유 안 여백으로 존중
    // (한글 PDF 실측: 시장구조조사 pad=0 셀의 코드 폭 37.0px > 표 폴백 inner
    // 26.5px — 폴백이면 물리적으로 1줄 배치 불가). 음수(결측 센티널)만 표 폴백.
    cell.apply_inner_margin = true;
    cell.padding = pad(141);
    assert_eq!(cell.effective_padding(&pad(0)).top, 141);
    cell.padding = pad(0);
    assert_eq!(cell.effective_padding(&pad(140)).top, 0);
    cell.padding = pad(-1);
    assert_eq!(cell.effective_padding(&pad(140)).top, 140);
}

fn table_bboxes(doc: &rhwp::wasm_api::HwpDocument) -> Vec<(f64, f64)> {
    let tree = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree: {:?}", e));
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("parse tree json");
    fn collect(v: &serde_json::Value, out: &mut Vec<(f64, f64)>) {
        if let Some(o) = v.as_object() {
            if o.get("type").and_then(|t| t.as_str()) == Some("Table") {
                if let Some(b) = o.get("bbox") {
                    out.push((
                        b.get("y").and_then(|y| y.as_f64()).unwrap_or(-1.0),
                        b.get("height").and_then(|h| h.as_f64()).unwrap_or(-1.0),
                    ));
                }
            }
            for c in o.values() {
                collect(c, out);
            }
        } else if let Some(a) = v.as_array() {
            for c in a {
                collect(c, out);
            }
        }
    }
    let mut out = Vec::new();
    collect(&json, &mut out);
    out
}

#[test]
fn issue_1785_roundtrip_preserves_microgrid_table_geometry() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    let original = table_bboxes(&doc);

    let hwp_bytes = doc
        .export_hwp_with_adapter()
        .unwrap_or_else(|e| panic!("adapter export: {:?}", e));
    let rt_doc = rhwp::wasm_api::HwpDocument::from_bytes(&hwp_bytes)
        .unwrap_or_else(|e| panic!("reparse hwp: {:?}", e));
    let roundtrip = table_bboxes(&rt_doc);

    assert_eq!(original.len(), roundtrip.len(), "표 개수 보존");
    for (i, (a, b)) in original.iter().zip(roundtrip.iter()).enumerate() {
        assert!(
            (a.0 - b.0).abs() < 0.5 && (a.1 - b.1).abs() < 0.5,
            "표[{}] 라운드트립 기하 불일치: 원본 y={:.2} h={:.2} vs 재파스 y={:.2} h={:.2} \
             (수정 전 결함: 발신명의 micro-grid 표 h 244.9 → 254.2)",
            i,
            a.0,
            a.1,
            b.0,
            b.1
        );
    }
}
