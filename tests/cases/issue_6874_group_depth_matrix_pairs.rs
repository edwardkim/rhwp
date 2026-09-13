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
//! 최상위 도형은 ctrl_id가 두 번 있으므로 4바이트 오프셋을 적용한다. 깊이 0을
//! 제외하지 않고, 같은 입력의 독립 한컴 HWP5 저장본과 모든 깊이별 개수까지 대조한다.
//! 스트림/레코드 해석 실패와 표본 누락은 실패다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;
use std::path::Path;

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("fixture {}: {e}", path.display()))
}

fn convert_hwp3(name: &str) -> Vec<u8> {
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&fixture(name)).expect("HWP3 parse");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        rhwp::parser::FileFormat::Hwp3,
    );
    rhwp::serializer::cfb_writer::serialize_hwp(&doc).expect("HWP5 serialize")
}

type Census = std::collections::BTreeMap<(u16, u16), usize>;

fn depth_and_pairs(bytes: &[u8]) -> Census {
    let mut cfb = CfbReader::open(bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header[36] & 0x01 != 0;
    let mut out = Census::new();
    assert!(cfb.section_count() > 0, "본문 section 없음");
    for index in 0..cfb.section_count() {
        let section = cfb
            .read_body_text_section(index, compressed, false)
            .expect("section 읽기");
        let records = Record::read_all(&section).expect("section records 읽기");
        for r in records {
            if r.tag_id != tags::HWPTAG_SHAPE_COMPONENT {
                continue;
            }
            assert!(r.data.len() >= 48, "짧은 SHAPE_COMPONENT");
            let offset = if r.data[0..4] == r.data[4..8] { 4 } else { 0 };
            assert!(r.data.len() >= 48 + offset);
            let depth = u16::from_le_bytes([r.data[12 + offset], r.data[13 + offset]]);
            let pairs = u16::from_le_bytes([r.data[46 + offset], r.data[47 + offset]]);
            // translation 48 bytes 뒤 각 scale/rotation 쌍은 96 bytes다.
            assert!(
                r.data.len() >= 48 + offset + 48 + usize::from(pairs) * 96,
                "선언한 행렬 쌍보다 짧은 레코드: depth={depth}, pairs={pairs}"
            );
            *out.entry((depth, pairs)).or_default() += 1;
        }
    }
    out
}

#[test]
fn hwp3_group_children_match_independent_hancom_census() {
    for (source, oracle, total, max_depth) in [
        (
            "samples/hwp3-sample11.hwp",
            "samples/hwp3-sample11-hwp5.hwp",
            1392,
            9,
        ),
        (
            "tests/fixtures/issue_4680/german-legislative-system.hwp",
            "tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp",
            358,
            3,
        ),
    ] {
        let expected = depth_and_pairs(&fixture(oracle));
        assert_eq!(expected.values().sum::<usize>(), total, "oracle {oracle}");
        assert_eq!(expected.keys().map(|(d, _)| *d).max(), Some(max_depth));
        assert!(expected.keys().all(|(depth, pairs)| *pairs == depth + 1));
        assert_eq!(
            depth_and_pairs(&convert_hwp3(source)),
            expected,
            "source {source}"
        );
    }
}

#[test]
fn top_level_shapes_keep_a_single_matrix_pair() {
    let rows = depth_and_pairs(&convert_hwp3("samples/hwp3-sample11.hwp"));
    // 최상위 25개도 실제로 검사한다. 빈 목록의 all()로 성공할 수 없다.
    assert_eq!(rows.get(&(0, 1)), Some(&25));
    assert!(!rows.keys().any(|(depth, pairs)| *depth == 0 && *pairs != 1));
}

#[test]
fn hwp5_oracle_roundtrip_preserves_all_group_matrix_counts() {
    for source in [
        "samples/hwp3-sample11-hwp5.hwp",
        "tests/fixtures/issue_4680/german-legislative-system-hancom-2020.hwp",
    ] {
        let bytes = fixture(source);
        let doc = rhwp::parser::parse_document(&bytes).expect("oracle parse");
        let saved = rhwp::serializer::cfb_writer::serialize_hwp(&doc).expect("oracle serialize");
        assert_eq!(depth_and_pairs(&saved), depth_and_pairs(&bytes), "{source}");
    }
}
