//! #8199 글자 없는 문단(조절 문자만 있는 문단)에 개체를 넣어도 저장 줄 정보의
//! 줄 시작(`PARA_LINE_SEG.text_start`)이 넣은 길이만큼 밀려야 한다.
//!
//! 입력: 국가법령정보센터 공개 빈 서식 「여권발급신청서」(여권법 시행규칙 별지 제1호서식).
//! 제보 링크(flSeq=164628771)의 파일은 기존 `samples/issue2020/passport_application_lawgo.hwp` 와
//! 바이트가 같아 그 재현물을 그대로 쓴다.
//! 구역 0 문단 0 은 글자 없이 `secd·cold·gso(사각형)·tbl·tbl` 다섯 조절 문자만 담는다.
//! 한/글이 저장한 원본의 둘째 줄은 **둘째 표의 스트림 위치(32)** 에서 시작한다.
//!
//! 독립 기대값: 줄 시작은 그 줄 첫 글자의 스트림 위치다(HWP5 LINE_SEG `text_start`).
//! 개체를 넣어도 기존 개체의 줄 소속은 그대로여야 하므로, 각 줄은 편집 전과 **같은 조절
//! 문자**에서 시작해야 한다. 조절 문자는 (종류, 같은 종류 안 순번)으로 가리킨다 — 넣은
//! 개체와 종류가 다르면 이 이름은 편집 전후에 같은 개체를 뜻한다. 원본 파일 자신이 이
//! 관계를 만족한다(첫 테스트).
//!
//! 결함: 글자가 없는 문단은 `char_offsets` 가 비어 있어 공통 시프트가 조기 반환했다.
//! 조절 문자는 8칸씩 밀리는데 둘째 줄은 32 에 남아, 한/글이 첫 표를 둘째 줄로 읽고
//! 쪽을 하나 더 만들었다(제보 한컴 2022 PDF 3쪽, 원본 2쪽).
//!
//! 공통 시프트(`shift_for_inline_control_insert`)를 쓰는 삽입 경로를 각각 실행한다.
//!
//! - 시프트 결과가 그대로 저장되는 경로(결함 검출): 그림(본문)·떠 있는 도형.
//! - 시프트 결과가 편집 직후 메모리에 남고 저장기는 줄 정보를 싣지 않는 경로(결함 검출):
//!   책갈피·구역 나누기.
//! - 넣은 뒤 문단을 다시 조판해(`reflow_paragraph`) 줄 정보를 새로 만드는 경로(대조군):
//!   각주·미주·새 번호·덧말·글자 겹침. 수정 전에도 통과한다 — 공통 시프트를 바꿔도 이
//!   경로의 줄 소속이 깨지지 않는지 지킨다.
//! - 수식도 다시 조판하지만 글자처럼 취급 수식이 표와 한 줄에 들지 못해 줄 구성 자체가
//!   바뀐다. 줄 소속 보존을 기대할 수 없으므로 여기서 검사하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::mem::Discriminant;
use std::path::PathBuf;

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;

/// 확장 조절 문자 하나가 문단 스트림에서 차지하는 UTF-16 code unit 수.
const EXTENDED_CONTROL_UNITS: u32 = 8;

fn sample_bytes(rel: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("입력을 읽지 못했다 {}: {e}", path.display()))
}

fn passport_form() -> HwpDocument {
    HwpDocument::from_bytes(&sample_bytes(
        "samples/issue2020/passport_application_lawgo.hwp",
    ))
    .expect("여권발급신청서 열기")
}

fn tiny_png() -> Vec<u8> {
    sample_bytes("assets/logo/logo-16.png")
}

fn para(doc: &HwpDocument, sec: usize, idx: usize) -> &Paragraph {
    &doc.document().sections[sec].paragraphs[idx]
}

fn kind(c: &Control) -> Discriminant<Control> {
    std::mem::discriminant(c)
}

/// 줄을 여는 조절 문자: (종류, 같은 종류 안 순번). `None` 은 문단 끝 자리.
type Opener = Option<(Discriminant<Control>, usize)>;

/// 글자 없는 문단의 둘째 줄부터 각 줄을 여는 조절 문자.
fn line_openers(para: &Paragraph, label: &str) -> Vec<Opener> {
    assert!(
        para.text.is_empty(),
        "{label}: 글자 없는 문단이어야 한다 (text={:?})",
        para.text
    );
    para.line_segs
        .iter()
        .skip(1)
        .map(|seg| {
            assert_eq!(
                seg.text_start % EXTENDED_CONTROL_UNITS,
                0,
                "{label}: 글자 없는 문단의 줄 시작은 조절 문자 경계여야 한다 ({})",
                seg.text_start
            );
            let idx = (seg.text_start / EXTENDED_CONTROL_UNITS) as usize;
            if idx == para.controls.len() {
                return None;
            }
            let c = para.controls.get(idx).unwrap_or_else(|| {
                panic!(
                    "{label}: 줄 시작 {} 이 조절 문자 {}개 밖이다",
                    seg.text_start,
                    para.controls.len()
                )
            });
            let ordinal = para.controls[..idx]
                .iter()
                .filter(|o| kind(o) == kind(c))
                .count();
            Some((kind(c), ordinal))
        })
        .collect()
}

fn kind_count(para: &Paragraph, k: Discriminant<Control>) -> usize {
    para.controls.iter().filter(|c| kind(c) == k).count()
}

/// 편집 전 문단의 줄 소속 기록.
struct Before {
    openers: Vec<Opener>,
    counts: Vec<usize>,
    controls: usize,
}

fn record(para: &Paragraph) -> Before {
    let openers = line_openers(para, "편집 전");
    assert!(
        !openers.is_empty(),
        "편집 전: 둘째 줄이 있어야 줄 시작 이동을 관측할 수 있다"
    );
    let counts = openers
        .iter()
        .map(|o| o.map_or(0, |(k, _)| kind_count(para, k)))
        .collect();
    Before {
        openers,
        counts,
        controls: para.controls.len(),
    }
}

/// 넣은 개체 수만큼 조절 문자가 늘고, 각 줄은 편집 전과 같은 조절 문자에서 시작한다.
fn assert_line_membership_kept(before: &Before, after: &Paragraph, inserted: usize, label: &str) {
    assert_eq!(
        after.controls.len(),
        before.controls + inserted,
        "{label}: 조절 문자가 {inserted}개 늘어야 한다"
    );
    for (opener, count) in before.openers.iter().zip(&before.counts) {
        if let Some((k, _)) = opener {
            assert_eq!(
                kind_count(after, *k),
                *count,
                "{label}: 넣은 개체가 줄을 여는 개체와 같은 종류라 순번으로 가리킬 수 없다"
            );
        }
    }
    assert_eq!(
        after.line_segs[0].text_start, 0,
        "{label}: 첫 줄은 문단 시작"
    );
    let starts: Vec<u32> = after.line_segs.iter().map(|s| s.text_start).collect();
    assert_eq!(
        line_openers(after, label),
        before.openers,
        "{label}: 각 줄은 편집 전과 같은 조절 문자에서 시작해야 한다 (줄 시작 {starts:?})"
    );
    assert_eq!(
        after.char_count,
        after.controls.len() as u32 * EXTENDED_CONTROL_UNITS + 1,
        "{label}: char_count 는 조절 문자 × 8 + 문단 끝"
    );
}

fn reparse(doc: &HwpDocument) -> HwpDocument {
    let bytes = doc.export_hwp_with_adapter_snapshot().expect("HWP 저장");
    HwpDocument::from_bytes(&bytes).expect("저장본 다시 열기")
}

/// 저장본을 다시 열어 검사한다.
///
/// 편집으로 줄 구성이 무효가 된 문단(`stored_text_partition_dirty`)은 저장기가 줄 정보를
/// 싣지 않는다 — 한/글이 다시 조판한다. 그때는 낡은 줄 시작이 파일에 남지 않았는지 본다.
/// 그 밖에는 저장된 줄 정보가 편집 전 줄 소속을 지켜야 한다.
fn assert_saved_line_membership(
    before: &Before,
    edited: &Paragraph,
    saved: &Paragraph,
    inserted: usize,
    label: &str,
) {
    if edited.stored_text_partition_dirty {
        assert!(
            saved.line_segs.is_empty(),
            "{label}: 무효가 된 줄 구성은 저장하지 않아야 한다 (줄 시작 {:?})",
            saved
                .line_segs
                .iter()
                .map(|s| s.text_start)
                .collect::<Vec<_>>()
        );
        assert_eq!(saved.controls.len(), before.controls + inserted, "{label}");
        return;
    }
    assert_line_membership_kept(before, saved, inserted, label);
}

fn ok_json(result: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_str(result).expect("JSON");
    assert_eq!(json["ok"], true, "{result}");
    json
}

/// 여권발급신청서 문단 0 에 `edit` 를 적용하고 편집 직후·저장 후 다시 연 상태를 검사한다.
fn check_passport_edit(label: &str, edit: impl FnOnce(&mut HwpDocument)) -> HwpDocument {
    let mut doc = passport_form();
    let before = record(para(&doc, 0, 0));
    edit(&mut doc);
    assert_line_membership_kept(&before, para(&doc, 0, 0), 1, &format!("{label} 편집 직후"));
    let reopened = reparse(&doc);
    assert_saved_line_membership(
        &before,
        para(&doc, 0, 0),
        para(&reopened, 0, 0),
        1,
        &format!("{label} 저장 후 다시 열기"),
    );
    doc
}

fn table_indices(para: &Paragraph) -> Vec<usize> {
    para.controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| matches!(c, Control::Table(_)).then_some(i))
        .collect()
}

/// 그림을 `char_offset` 에 넣고 글 앞으로 둔다(제보 재현 순서와 같다).
fn insert_front_picture(doc: &mut HwpDocument, char_offset: usize) -> usize {
    let png = tiny_png();
    let result = doc
        .insert_picture_native(
            0,
            0,
            char_offset,
            &[],
            &png,
            2835,
            2835,
            16,
            16,
            "png",
            "issue8199",
            None,
            None,
        )
        .expect("그림 넣기");
    let json = ok_json(&result);
    assert_eq!(json["paraIdx"], 0, "{result}");
    let ctrl = json["controlIdx"].as_u64().expect("controlIdx") as usize;
    doc.set_picture_properties_native(0, 0, ctrl, r#"{"textWrap":"InFrontOfText"}"#)
        .expect("그림 속성");
    ctrl
}

#[test]
fn original_hancom_save_starts_second_line_at_second_table() {
    let doc = passport_form();
    let p = para(&doc, 0, 0);
    assert!(p.char_offsets.is_empty(), "결함 조건: 글자 없는 문단");
    assert_eq!(table_indices(p), vec![3, 4]);
    assert_eq!(p.line_segs.len(), 2);
    // 둘째 줄을 여는 개체는 둘째 표다.
    assert_eq!(
        line_openers(p, "원본"),
        vec![Some((kind(&p.controls[3]), 1))]
    );
}

#[test]
fn picture_before_first_table_keeps_line_membership() {
    let pages_before = passport_form().page_count();
    let doc = check_passport_edit("그림(첫 표 앞)", |doc| {
        let ctrl = insert_front_picture(doc, 0);
        let tables = table_indices(para(doc, 0, 0));
        assert!(
            ctrl < tables[0],
            "제보 경로: 첫 표 앞 (ctrl={ctrl}, 표={tables:?})"
        );
    });
    assert_eq!(
        reparse(&doc).page_count(),
        pages_before,
        "글 앞 그림은 쪽 수를 바꾸지 않는다"
    );
}

#[test]
fn picture_between_tables_keeps_line_membership() {
    check_passport_edit("그림(두 표 사이)", |doc| {
        let ctrl = insert_front_picture(doc, 1);
        let tables = table_indices(para(doc, 0, 0));
        assert!(
            tables[0] < ctrl && ctrl < tables[1],
            "두 표 사이 (ctrl={ctrl}, 표={tables:?})"
        );
    });
}

#[test]
fn picture_after_both_tables_leaves_line_starts_alone() {
    // 반례: 넣은 자리가 둘째 줄 시작보다 뒤면 줄 시작은 그대로다.
    let doc = check_passport_edit("그림(두 표 뒤)", |doc| {
        let ctrl = insert_front_picture(doc, 2);
        let tables = table_indices(para(doc, 0, 0));
        assert!(ctrl > tables[1], "두 표 뒤 (ctrl={ctrl}, 표={tables:?})");
    });
    assert_eq!(para(&doc, 0, 0).line_segs[1].text_start, 32);
}

#[test]
fn shape_keeps_line_membership() {
    check_passport_edit("도형", |doc| {
        ok_json(
            &doc.create_shape_control_native(
                0,
                0,
                0,
                2835,
                2835,
                0,
                0,
                false,
                "InFrontOfText",
                "ellipse",
                false,
                false,
                &[],
            )
            .expect("도형 넣기"),
        );
    });
}

#[test]
fn bookmark_keeps_line_membership() {
    check_passport_edit("책갈피", |doc| {
        ok_json(
            &doc.add_bookmark_native(0, 0, 0, "issue8199")
                .expect("책갈피 넣기"),
        );
    });
}

#[test]
fn section_split_keeps_line_membership_of_moved_paragraph() {
    // 구역 나누기는 새 구역 첫 문단 맨 앞에 단 정의·구역 정의 두 조절 문자를 넣는다.
    // 문단 0.16 은 글자 없이 `표·사각형·표·…` 를 담고 세 줄(줄 시작 0·8·24)이다.
    let mut doc = HwpDocument::from_bytes(&sample_bytes(
        "samples/issue8446/156667426-reset-line-logo.hwp",
    ))
    .expect("입력 열기");
    let before = record(para(&doc, 0, 16));
    assert_eq!(before.openers.len(), 2, "세 줄 문단이어야 한다");

    let json = ok_json(&doc.split_section_native(0, 16, 0).expect("구역 나누기"));
    assert_eq!(json["section"], 1, "{json}");
    assert_eq!(json["para"], 0, "{json}");
    assert_line_membership_kept(&before, para(&doc, 1, 0), 2, "구역 나누기 직후");
    let reopened = reparse(&doc);
    assert_saved_line_membership(
        &before,
        para(&doc, 1, 0),
        para(&reopened, 1, 0),
        2,
        "구역 나누기 저장 후",
    );
}

// ---- 대조군: 넣은 뒤 문단을 다시 조판하는 경로 (수정 전에도 통과) ----

#[test]
fn footnote_keeps_line_membership() {
    check_passport_edit("각주", |doc| {
        ok_json(&doc.insert_footnote_native(0, 0, 0).expect("각주 넣기"));
    });
}

#[test]
fn endnote_keeps_line_membership() {
    check_passport_edit("미주", |doc| {
        ok_json(&doc.insert_endnote_native(0, 0, 0).expect("미주 넣기"));
    });
}

#[test]
fn new_number_keeps_line_membership() {
    check_passport_edit("새 번호", |doc| {
        ok_json(
            &doc.insert_new_number_native(0, 0, 0, 1)
                .expect("새 번호 넣기"),
        );
    });
}

#[test]
fn ruby_keeps_line_membership() {
    check_passport_edit("덧말", |doc| {
        ok_json(
            &doc.insert_ruby_native(0, 0, 0, r#"{"mainText":"漢字","rubyText":"한자"}"#)
                .expect("덧말 넣기"),
        );
    });
}

#[test]
fn char_overlap_keeps_line_membership() {
    check_passport_edit("글자 겹침", |doc| {
        ok_json(
            &doc.insert_char_overlap_native(0, 0, 0, r#"{"text":"12","shape":5}"#)
                .expect("글자 겹침 넣기"),
        );
    });
}
