//! [Issue #8305] HWPX 저장이 제목 차례 표시(titleMark)를 자기 글자 모양 run 에 둔다.
//!
//! 표시는 UTF-16 축에서 8유닛을 차지한다. 한컴 9.x HWPX 는 표시를 글자 모양 경계에 붙여
//! 쓴다 — 표시만 담은 run(`글 run | 표시 run | 다음 run`), 문단 끝 표시 뒤 다른 모양의
//! `newNum` run 등. rhwp 는 표시를 앞 run 에 싣거나 표시 뒤 경계를 표시 앞에서 끊어,
//! 재저장한 HWPX 의 글자 모양 경계가 표시 폭(8)만큼 밀렸다(`export-hwpx --verify` exit 3).
//! 경계와 같은 위치의 콘텐츠는 새 run 소속이라는 run 분할 규칙을 표시에도 적용한다.
//! 실물 기준: `pdf/issue4599/1790387_policy_progress-hwpx-2020.pdf`(64쪽).

use std::io::Read;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{AutoNumberType, Control, NewNumber};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, Paragraph, TitleMark};
use rhwp::serializer::hwpx::serialize_hwpx;

fn section0_xml(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX zip 열기");
    let mut entry = zip
        .by_name("Contents/section0.xml")
        .expect("Contents/section0.xml");
    let mut xml = String::new();
    entry.read_to_string(&mut xml).expect("section0.xml 읽기");
    xml
}

fn shapes(list: &[(u32, u32)]) -> Vec<CharShapeRef> {
    list.iter()
        .map(|&(start_pos, char_shape_id)| CharShapeRef {
            start_pos,
            char_shape_id,
        })
        .collect()
}

fn mark(char_idx: usize) -> Vec<TitleMark> {
    vec![TitleMark {
        char_idx,
        ignore: true,
    }]
}

/// `para` 를 HWPX 로 직렬화한 run 열. 첫 문단 run 에는 구역 정의(`secPr`)가 붙으므로
/// 빈 문단을 앞에 두고 두 번째 문단만 본다.
fn runs_of(para: Paragraph) -> String {
    let mut doc = Document::default();
    doc.doc_info.char_shapes = vec![Default::default(), Default::default()];
    doc.sections.push(Section {
        paragraphs: vec![Paragraph::default(), para],
        ..Default::default()
    });
    let xml = section0_xml(&serialize_hwpx(&doc).expect("HWPX 직렬화"));
    let first = xml.find("<hp:p ").expect("첫 문단");
    let second = first + 1 + xml[first + 1..].find("<hp:p ").expect("두 번째 문단");
    let para_xml = &xml[second..second + xml[second..].find("</hp:p>").expect("문단 끝")];
    let start = para_xml.find("<hp:run ").expect("run 이 있어야 한다");
    let end = para_xml.rfind("</hp:run>").expect("run 끝") + "</hp:run>".len();
    para_xml[start..end].to_string()
}

const MARK_T: &str = r#"<hp:t><hp:titleMark ignore="1"/></hp:t>"#;

/// 문단 끝 표시의 **시작** 유닛에 경계가 있으면 표시는 새 run 에 든다
/// (한컴: `<run 86>…결과</run><run 49><t><titleMark/></t></run>`).
#[test]
fn trailing_mark_after_boundary_opens_its_own_run() {
    let runs = runs_of(Paragraph {
        text: "가나".to_string(),
        char_offsets: vec![0, 1],
        // 글자 2 + 표시 8 + 문단 끝 1
        char_count: 11,
        char_shapes: shapes(&[(0, 0), (2, 1)]),
        title_marks: mark(2),
        ..Default::default()
    });
    assert_eq!(
        runs,
        format!(
            r#"<hp:run charPrIDRef="0"><hp:t>가나</hp:t></hp:run><hp:run charPrIDRef="1">{MARK_T}</hp:run>"#
        )
    );
}

/// 문장 중간 표시의 시작·끝 유닛에 모두 경계가 있으면 표시만 담은 run 이 된다
/// (한컴: `<run 41>…기술지원</run><run 49>표시</run><run 41> </run>`).
#[test]
fn mid_text_mark_between_two_boundaries_keeps_its_own_run() {
    let runs = runs_of(Paragraph {
        text: "가나다".to_string(),
        char_offsets: vec![0, 1, 10],
        char_count: 12,
        char_shapes: shapes(&[(0, 0), (2, 1), (10, 0)]),
        title_marks: mark(2),
        ..Default::default()
    });
    assert_eq!(
        runs,
        format!(
            r#"<hp:run charPrIDRef="0"><hp:t>가나</hp:t></hp:run><hp:run charPrIDRef="1">{MARK_T}</hp:run><hp:run charPrIDRef="0"><hp:t>다</hp:t></hp:run>"#
        )
    );
}

/// 문단 끝 표시의 **끝** 유닛에 경계가 있으면 표시는 앞 run 에 남고, 뒤따르는 말미 슬롯
/// (`newNum`)이 경계 뒤 run 에 든다(한컴: `<run 49>…내용 표시</run><run 80>newNum</run>`).
#[test]
fn trailing_mark_before_boundary_leaves_trailing_slot_in_next_run() {
    let runs = runs_of(Paragraph {
        text: "가나".to_string(),
        char_offsets: vec![0, 1],
        // 글자 2 + 표시 8 + newNum 8 + 문단 끝 1
        char_count: 19,
        char_shapes: shapes(&[(0, 0), (10, 1)]),
        title_marks: mark(2),
        controls: vec![Control::NewNumber(NewNumber {
            number_type: AutoNumberType::Page,
            number: 1,
        })],
        ..Default::default()
    });
    let mark_at = runs.find("<hp:titleMark").expect("표시");
    let boundary = runs
        .find(r#"<hp:run charPrIDRef="1">"#)
        .expect("경계 뒤 run");
    let new_num = runs.find("<hp:newNum").expect("newNum");
    assert!(
        mark_at < boundary && boundary < new_num,
        "표시는 경계 앞 run, newNum 은 경계 뒤 run 이어야 한다: {runs}"
    );
}

/// 경계가 없으면 문단 끝 표시는 종전처럼 같은 run 에 남는다(반례).
#[test]
fn trailing_mark_without_boundary_stays_in_the_run() {
    let runs = runs_of(Paragraph {
        text: "가나".to_string(),
        char_offsets: vec![0, 1],
        char_count: 11,
        char_shapes: shapes(&[(0, 0)]),
        title_marks: mark(2),
        ..Default::default()
    });
    assert_eq!(
        runs,
        format!(r#"<hp:run charPrIDRef="0"><hp:t>가나</hp:t>{MARK_T}</hp:run>"#)
    );
}

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

/// 구역·문단마다 (글자 모양 경계, 표시 위치, 글자 위치).
#[allow(clippy::type_complexity)]
fn mark_layout(core: &DocumentCore) -> Vec<(usize, usize, Vec<(u32, u32)>, Vec<usize>, Vec<u32>)> {
    let mut out = Vec::new();
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            out.push((
                si,
                pi,
                para.char_shapes
                    .iter()
                    .map(|c| (c.start_pos, c.char_shape_id))
                    .collect(),
                para.title_marks.iter().map(|m| m.char_idx).collect(),
                para.char_offsets.clone(),
            ));
        }
    }
    out
}

fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| core.extract_page_text_native(page).expect("쪽 글"))
        .collect()
}

/// 한컴 HWPX 재저장: 세 모양이 모두 있는 문서에서 모든 문단의 글자 모양 경계·표시·글자
/// 위치와 쪽별 본문이 원본과 같다.
#[test]
fn hancom_hwpx_resave_keeps_title_mark_boundaries() {
    let original = load("samples/issue4599/1790387_policy_progress.hwpx");
    let before = mark_layout(&original);
    let find = |si: usize, pi: usize| {
        before
            .iter()
            .find(|row| row.0 == si && row.1 == pi)
            .map(|row| (row.2.clone(), row.3.len()))
            .expect("문단")
    };
    // 입력 확인 — 끝 표시(다음 run 소유), 끝 표시 + newNum(앞 run 소유), 중간 표시 run.
    assert_eq!(find(2, 0), (vec![(0, 86), (44, 49)], 1));
    assert_eq!(find(0, 83), (vec![(0, 49), (19, 80)], 1));
    assert_eq!(find(2, 245), (vec![(0, 41), (25, 49), (33, 41)], 1));

    let resaved = DocumentCore::from_bytes(&original.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기");
    let after = mark_layout(&resaved);
    assert_eq!(after.len(), before.len(), "문단 수");
    for (b, a) in before.iter().zip(&after) {
        assert_eq!(a, b, "구역 {} 문단 {}", b.0, b.1);
    }
    assert_eq!(page_texts(&resaved), page_texts(&original), "쪽별 본문");
}
