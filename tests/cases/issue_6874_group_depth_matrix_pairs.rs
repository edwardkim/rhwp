//! [#6874] 묶음 자식의 **그룹 깊이**가 IR 에 실리고, `SHAPE_COMPONENT` 의 rendering
//! 행렬 쌍 개수가 그 깊이를 따른다.
//!
//! ## 규칙의 출처
//!
//! 한/글이 같은 HWP3 원본을 저장한 HWP5 변환본에서 **행렬 쌍 개수 = 그룹 깊이 + 1**
//! 이다. 두 문서에서 깊이별 개수까지 전부 일치한다(이 값은 구현이 아니라 변환본에서 왔다):
//!
//! ```text
//!   samples/hwp3-sample11.hwp   (1,2):76 (2,3):195 (3,4):99 (4,5):145 (5,6):112
//!                               (6,7):140 (7,8):184 (8,9):224 (9,10):192
//!   코퍼스 1170000-200500003    (1,2):186 (2,3):142 (3,4):20
//! ```
//!
//! 우리 자신의 HWP5 재저장(o2h)도 같은 census 를 낸다 — HWP5 를 읽어 다시 쓸 때는
//! 원본 rendering 바이트를 그대로 보존하기 때문이다. 어긋나던 것은 **HWP3 읽기 경로**
//! 하나였다.
//!
//! ## 근인 둘
//!
//! 1. `parser/hwp3/drawing.rs::parse_drawing_object_tree` 가 `shape_attr.group_level`
//!    을 매기지 않아 모든 자식이 0 이었다.
//! 2. `serializer/control.rs` 의 세 rendering 경로가 깊이를 안 봤다 — 폴백은
//!    `group_level > 0` 이면 무조건 2쌍, 명시 변환 경로는 깊이와 무관하게 1쌍.
//!
//! 그래서 HWP3 변환본의 묶음 자식 레코드가 한/글 저장본보다 깊이당 96바이트씩 짧았다.
//!
//! ## 이 테스트가 잠그는 것
//!
//! 저장 바이트에서 `(그룹 깊이, 행렬 쌍 개수)` 를 직접 읽어 `쌍 = 깊이 + 1` 을 단언한다.
//! 최상위 도형(`SHAPE_COMPONENT` 가 ctrl_id 를 두 번 쓰는 것)은 오프셋이 4 밀리므로
//! 제외한다 — 그걸 놓치면 엉뚱한 바이트를 보고 수정 전에도 통과한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;
use std::path::Path;

fn sample(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name)
}

/// HWP3 를 CLI `convert` 와 같은 경로로 HWP5 바이트로 만든다.
fn convert_hwp3(name: &str) -> Option<Vec<u8>> {
    let raw = std::fs::read(sample(name)).ok()?;
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).ok()?;
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        rhwp::parser::FileFormat::Hwp3,
    );
    rhwp::serializer::cfb_writer::serialize_hwp(&doc).ok()
}

/// 저장 바이트의 묶음 자식 `SHAPE_COMPONENT` 에서 (그룹 깊이, 행렬 쌍 개수)를 모은다.
fn depth_and_pairs(bytes: &[u8]) -> Vec<(u16, u16)> {
    let mut cfb = CfbReader::open(bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let mut out = Vec::new();
    for index in 0..64u32 {
        let Ok(section) = cfb.read_body_text_section(index, compressed, false) else {
            break;
        };
        let Ok(records) = Record::read_all(&section) else {
            break;
        };
        for r in records {
            if r.tag_id != tags::HWPTAG_SHAPE_COMPONENT || r.data.len() < 48 {
                continue;
            }
            // 최상위 도형은 ctrl_id 를 두 번 쓴다 — 그때는 오프셋이 4 밀린다.
            if r.data[0..4] == r.data[4..8] {
                continue;
            }
            let depth = u16::from_le_bytes([r.data[12], r.data[13]]);
            let pairs = u16::from_le_bytes([r.data[46], r.data[47]]);
            out.push((depth, pairs));
        }
    }
    out
}

/// 깊이 1~9 가 모두 나오는 실문서 — 한/글 변환본과 깊이별 개수까지 맞는다.
#[test]
fn hwp3_group_children_get_depth_matched_matrix_pairs() {
    let Some(bytes) = convert_hwp3("hwp3-sample11.hwp") else {
        panic!("samples/hwp3-sample11.hwp 변환 실패 — 표본 배치를 확인하라");
    };
    let rows = depth_and_pairs(&bytes);
    assert!(
        rows.len() > 100,
        "묶음 자식 SHAPE_COMPONENT 가 너무 적다 ({}) — 전제가 깨졌다",
        rows.len()
    );
    let bad: Vec<_> = rows
        .iter()
        .filter(|(depth, pairs)| *pairs != depth.saturating_add(1))
        .take(8)
        .collect();
    assert!(
        bad.is_empty(),
        "행렬 쌍 개수가 그룹 깊이 + 1 이 아니다 (한/글 변환본 규칙) — {:?}",
        bad
    );
    let max_depth = rows.iter().map(|(d, _)| *d).max().unwrap_or(0);
    assert!(
        max_depth >= 3,
        "중첩 깊이가 얕아 규칙을 시험하지 못한다: 최대 {max_depth}"
    );
}

/// 최상위 도형(깊이 0)은 종전대로 1쌍이다 — 반례.
#[test]
fn top_level_shapes_keep_a_single_matrix_pair() {
    let Some(bytes) = convert_hwp3("hwp3-sample11.hwp") else {
        panic!("samples/hwp3-sample11.hwp 변환 실패");
    };
    let rows = depth_and_pairs(&bytes);
    let zero_depth: Vec<_> = rows.iter().filter(|(d, _)| *d == 0).collect();
    assert!(
        zero_depth.iter().all(|(_, pairs)| *pairs == 1),
        "깊이 0 인데 쌍이 1 이 아니다 — {:?}",
        zero_depth.iter().take(8).collect::<Vec<_>>()
    );
}
