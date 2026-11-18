//! 관계형·고정형 회귀 — 유효한 저장 표 띠·후속 줄 관계와 독립 Print 쪽수를 보존한다.
//! 주석 갱신: 2026-11-06T23:58:40+09:00; 직전 파일 커밋: 2026-11-06T22:51:33+09:00,
//! `636ff407ac0070ff01388de8d260946bb63c834f`(Git committer 시각, 측정 시각과 별개).
//!
//! 입력: `samples/issue8679/stored_band_crossing_line.hwpx`(서울시 결재문서 36385506).
//! 최초 커밋·다운로드 원본·현재 fixture 모두 저장 줄 4723, 후속 문단 5203이다.
//! 이전 주석의 2200/4723은 이 입력의 저장값이 아니므로 교차 줄 정규화의 검출 근거로 쓰지 않는다.
//! 제목 표 띠 끝에 놓인 줄과 그 높이·줄 간격 뒤의 후속 문단을 다시 밀지 않는 대조 검사다.
//!
//! 2026-11-06 측정: 제품 `9470fb026a8cd3b3d9594db55298b6f6c347bfda`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96`, Print 프로필·compat 2022.
//! raw는 색상 경계 보정 전 2px 이웃 실루엣이다. Native Skia/fresh WASM Canvas
//! 전 2쪽의 1·2쪽 모두 각각 100.00%, 각 경로 전쪽 최저 100.00%, 미대응 내용 픽셀 0.
//! 독립 PDF `pdf/issue8679_stored_band_crossing_line-2020.pdf`는 실제 Hwp 2022 Print 2쪽이다.
//! 두 경로의 review/overlay에서 표 순서·간격과 2쪽 행·내용 보존을 직접 판독했다.
//! 2026-11-07 R11 공통 보정 후에도 양쪽 1·2쪽 PNG 바이트가 동일함을 확인했다.
//! 해당 제품 해시와 최소 글꼴 검사 결과는 아래 증거의 r11-provenance.json을 따른다.
//! 근거: `mydocs/pr/assets/planet6897_20261106/stored-band-contract-analysis.md`,
//! `stored-band-contract/{native,wasm}.tsv`와 같은 폴더의 PNG·provenance.
//! 검사는 원본 저장 줄·표 간격·순서의 관계형과 독립 PDF 전체 쪽수의 고정형을 함께 사용한다.
//! 절대 화면 px 좌표를 고정하지 않으며 시스템 글꼴 설치 여부에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8679/stored_band_crossing_line.hwpx";

fn px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.control_index == Some(control) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

fn first_table(core: &DocumentCore, para: usize) -> (usize, &Table) {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .enumerate()
        .find_map(|(ci, c)| match c {
            Control::Table(t) => Some((ci, t.as_ref())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {para} 에 표"))
}

#[test]
fn valid_stored_band_and_successor_spacing_are_preserved() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 2, "한컴 Print PDF 2쪽");

    // 원본의 유효한 관계: 줄 1은 제목 표 띠 끝, 후속 문단은 그 줄의 전진 뒤다.
    let host = &core.document().sections[0].paragraphs[1];
    let (title_ci, title) = first_table(&core, 1);
    assert!(
        !title.common.treat_as_char,
        "제목 표는 글자처럼 취급하지 않는다"
    );
    let [line0, line1] = host.line_segs.as_slice() else {
        panic!("문단 1 은 저장 줄 둘");
    };
    let band_top = i64::from(line0.vertical_pos) + i64::from(title.common.vertical_offset as i32);
    let band_bottom = band_top
        + i64::from(title.outer_margin_top)
        + i64::from(title.common.height)
        + i64::from(title.outer_margin_bottom);
    assert!(
        i64::from(line0.vertical_pos + line0.line_height) <= band_top,
        "줄 0 은 띠 위에서 끝난다"
    );
    assert_eq!(
        i64::from(line1.vertical_pos),
        band_bottom,
        "이미 유효한 저장 줄 1은 제목 표 띠 끝에 있다"
    );
    let next = &core.document().sections[0].paragraphs[2];
    assert_eq!(
        i64::from(next.line_segs[0].vertical_pos),
        i64::from(line1.vertical_pos)
            + i64::from(line1.line_height)
            + i64::from(line1.line_spacing),
        "원본의 다음 문단은 저장 줄 1의 높이·줄 간격 뒤에 있다"
    );

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("1쪽 렌더 트리: {e:?}"));
    let title_box = find_table(&tree.root, 1, title_ci).expect("제목 표를 그린다");
    let (next_ci, next_table) = first_table(&core, 2);
    let next_box = find_table(&tree.root, 2, next_ci).expect("다음 문단 표를 그린다");

    // 저장된 줄과 후속 문단의 전진 관계를 실제 두 표의 간격에서도 보존한다.
    let expected_gap = px(i64::from(title.outer_margin_bottom))
        + px(i64::from(line1.line_height) + i64::from(line1.line_spacing))
        + px(i64::from(next_table.outer_margin_top));
    let gap = next_box.y - (title_box.y + title_box.height);
    assert!(
        (gap - expected_gap).abs() <= 0.5,
        "제목 표 아래 끝→다음 표 위끝 {gap:.2} = 아래 바깥여백 + 저장 줄 높이·줄 간격 + 위 바깥여백 {expected_gap:.2}"
    );
    assert!(
        next_box.y > title_box.y + title_box.height,
        "다음 표는 제목 표 아래"
    );
}
