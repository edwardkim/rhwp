//! [Issue #6868] 중첩 문단 목록(각주 subList·표 칸·글상자) 안에서 열리고 닫히는
//! 다단락 누름틀의 종료 마커가 짝 `fieldBegin` 의 control id 를 잇는지 고정한다.
//!
//! `link_orphan_field_ends` 는 본디 구역 최상위 `section.paragraphs` 에만 걸렸다. 중첩
//! 목록의 종료 마커는 `begin_ctrl_id` 가 0 으로 남고, HWP5 저장기의 두 방출 지점이 모두
//! `begin_ctrl_id != 0` 을 요구하므로 **끝 표시가 통째로 사라졌다** — 끝이 없는 누름틀은
//! 문단 나머지를 필드 안으로 삼킨다.
//!
//! 필드는 컨테이너 경계를 넘지 못하므로 목록마다 **독립적으로** 이어야 한다. 둘째 시험이
//! 그 경계를 지킨다 — 바깥 목록의 열린 필드를 중첩 목록이 짝으로 훔치지 않는다.
//!
//! `src/` 안 `#[cfg(test)]` 총량은 래칫으로 묶여 있어(`rust-unit-test-tiers`) 이 시험은
//! 통합 시험으로 둔다. `parse_hwpx_section` 은 이미 공개 경로라 그대로 부를 수 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::parser::hwpx::section::{link_orphan_field_ends_across_sections, parse_hwpx_section};

#[test]
fn issue6868_orphan_field_end_links_inside_nested_paragraph_lists() {
    // 다단락 누름틀이 **중첩 문단 목록 안에서** 열리고 닫힌다 (각주 subList / 표 칸).
    // 종전에는 `link_orphan_field_ends` 가 구역 최상위 문단에만 걸려 이 종료 마커의
    // `begin_ctrl_id` 가 0 으로 남았고, HWP5 저장기의 두 방출 지점이 모두
    // `begin_ctrl_id != 0` 을 요구해 **끝 표시가 통째로 사라졌다**(#6868).
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
    xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0">
  <hp:ctrl>
    <hp:footNote number="1" instId="200">
      <hp:subList>
        <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="555000111" type="CLICK_HERE" name="각주필드" fieldid="627272811"/></hp:ctrl><hp:t>앞</hp:t></hp:run></hp:p>
        <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>뒤</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="555000111" fieldid="627272811"/></hp:ctrl></hp:run></hp:p>
      </hp:subList>
    </hp:footNote>
  </hp:ctrl>
</hp:run>
  </hp:p>
</hs:sec>"##;
    let section = parse_hwpx_section(xml).unwrap();
    let footnote = section.paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Footnote(f) => Some(f),
            _ => None,
        })
        .expect("각주 컨트롤");
    let begin_ctrl_id = match footnote.paragraphs[0].controls.first() {
        Some(Control::Field(field)) => field.ctrl_id,
        other => panic!("각주 첫 문단이 fieldBegin 을 갖지 않는다: {other:?}"),
    };
    let ofe = footnote.paragraphs[1]
        .orphan_field_ends
        .first()
        .expect("각주 둘째 문단에 고아 fieldEnd 기록");
    assert_eq!(ofe.begin_id_ref, 555_000_111);
    assert_eq!(
        ofe.begin_ctrl_id, begin_ctrl_id,
        "중첩 목록 안에서도 짝 fieldBegin 의 control id 를 잇는다 (0 이면 HWP5 저장에서 끝 표시가 사라진다)"
    );
}

#[test]
fn issue6868_nested_list_does_not_borrow_outer_open_field() {
    // 바깥 문단이 **열어 둔** 필드를 중첩 목록의 종료 마커가 닫는 짝으로 훔치면 안 된다.
    // 목록마다 독립적으로 이어야 한다 — 필드는 컨테이너 경계를 넘지 못한다.
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
    xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
<hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="900000001" type="CLICK_HERE" name="바깥" fieldid="1"/></hp:ctrl><hp:t>바깥열림</hp:t></hp:run>
<hp:run charPrIDRef="0">
  <hp:ctrl>
    <hp:footNote number="1" instId="200">
      <hp:subList>
        <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>안</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="900000001" fieldid="1"/></hp:ctrl></hp:run></hp:p>
      </hp:subList>
    </hp:footNote>
  </hp:ctrl>
</hp:run>
  </hp:p>
</hs:sec>"##;
    let section = parse_hwpx_section(xml).unwrap();
    let footnote = section.paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Footnote(f) => Some(f),
            _ => None,
        })
        .expect("각주 컨트롤");
    let ofe = footnote.paragraphs[0]
        .orphan_field_ends
        .first()
        .expect("각주 문단의 고아 fieldEnd");
    assert_eq!(
        ofe.begin_ctrl_id, 0,
        "바깥 목록의 열린 필드를 중첩 목록이 짝으로 가져오지 않는다"
    );
}

/// 구역 경계를 넘는 누름틀도 짝을 잇는다 (#6868 잔여, 재난안전실 36455713).
///
/// `section*.xml` 은 **한 본문 흐름을 나눠 담은 것**이라 누름틀이 구역을 넘는다. 구역
/// 하나를 파싱하는 동안에는 앞 구역에서 열린 필드를 볼 수 없어 종료 마커의
/// `begin_ctrl_id` 가 0 으로 남았고, 그래서 HWP5 저장기가 끝 표시를 안 냈다
/// (한/글 집계 빈 `CtrlID` 3→2). 같은 구역 안에서 닫히는 필드는 영향이 없어야 한다.
#[test]
fn issue6868_orphan_field_end_links_across_section_boundary() {
    const SEC0: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
    xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="1799035886" type="CLICK_HERE" name="본문" fieldid="7"/></hp:ctrl><hp:t>앞구역</hp:t></hp:run></hp:p>
</hs:sec>"##;
    // 둘째 구역은 자기 안에서 열고 닫는 필드도 하나 갖는다 — 통제군이다.
    const SEC1: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
    xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="1693948357" type="FORMULA" fieldid="8"/></hp:ctrl><hp:t>수식</hp:t></hp:run></hp:p>
  <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>닫기</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="1693948357" fieldid="8"/></hp:ctrl></hp:run></hp:p>
  <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>뒷구역</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="1799035886" fieldid="7"/></hp:ctrl></hp:run></hp:p>
</hs:sec>"##;

    let mut sections = vec![
        parse_hwpx_section(SEC0).unwrap(),
        parse_hwpx_section(SEC1).unwrap(),
    ];

    let cross_begin_ctrl_id = match sections[0].paragraphs[0].controls.first() {
        Some(Control::Field(field)) => field.ctrl_id,
        other => panic!("앞 구역 첫 문단이 fieldBegin 을 갖지 않는다: {other:?}"),
    };
    assert_ne!(cross_begin_ctrl_id, 0, "fieldBegin 의 control id");

    link_orphan_field_ends_across_sections(&mut sections);

    // 같은 구역 안에서 닫히는 FORMULA — 구역 단위 짝잇기가 이미 이었고 그대로다.
    let same_section = sections[1].paragraphs[1]
        .orphan_field_ends
        .first()
        .expect("둘째 구역 안에서 닫히는 고아 fieldEnd");
    assert_eq!(same_section.begin_id_ref, 1_693_948_357);
    assert_ne!(
        same_section.begin_ctrl_id, 0,
        "구역 안에서 닫히는 필드의 짝은 그대로 남는다"
    );

    // 구역을 넘는 CLICK_HERE — 이 이슈가 고치는 자리.
    let across = sections[1].paragraphs[2]
        .orphan_field_ends
        .first()
        .expect("구역을 넘는 고아 fieldEnd");
    assert_eq!(across.begin_id_ref, 1_799_035_886);
    assert_eq!(
        across.begin_ctrl_id, cross_begin_ctrl_id,
        "앞 구역 fieldBegin 의 control id 를 이어야 HWP5 저장기가 끝 표시를 낸다"
    );
}

/// HWP5 축도 같다 — `BodyText/SectionN` 을 넘는 누름틀의 종료 마커를 잇는다.
///
/// HWP5 종료 마커에는 짝 id 가 없어 순서만이 짝이다. 구역마다 스택을 버리면 `Section1`
/// 의 마커가 `begin_id_ref = 0` 으로 남고, HWPX 직렬화기의 `emit_orphan_field_end` 가
/// `#5252` 가드로 그것을 버린다 — 그 가드의 전제("0 이면 문서 어디에도 짝이 없다")가
/// 구역을 넘는 필드에서는 거짓이다. 짝이 **정말로** 없는 마커는 그대로 0 이어야 한다.
#[test]
fn issue6868_hwp5_orphan_field_end_links_across_section_boundary() {
    use rhwp::model::control::Field;
    use rhwp::model::document::Section;
    use rhwp::model::paragraph::{OrphanFieldEnd, Paragraph};
    use rhwp::parser::body_text::link_orphan_field_ends_across_sections;

    /// 이 구역에서 열고 닫지 않는 누름틀 하나를 담은 문단.
    fn opening_para(field_id: u32, ctrl_id: u32) -> Paragraph {
        let mut para = Paragraph::default();
        para.controls.push(Control::Field(Field {
            field_id,
            ctrl_id,
            ..Default::default()
        }));
        para
    }

    /// 짝 없이 끝만 있는 문단.
    fn closing_para() -> Paragraph {
        let mut para = Paragraph::default();
        para.orphan_field_ends.push(OrphanFieldEnd {
            char_idx: 0,
            begin_id_ref: 0,
            field_id: 0,
            begin_ctrl_id: 0,
        });
        para
    }

    let mut sections = vec![Section::default(), Section::default()];
    sections[0]
        .paragraphs
        .push(opening_para(1_799_035_886, 627_272_811));
    sections[1].paragraphs.push(closing_para());
    // 짝이 정말 없는 둘째 마커 — `#5252` 의 본문 폐기 방지가 계속 걸려야 한다.
    sections[1].paragraphs.push(closing_para());

    link_orphan_field_ends_across_sections(&mut sections);

    let linked = &sections[1].paragraphs[0].orphan_field_ends[0];
    assert_eq!(linked.begin_id_ref, 1_799_035_886);
    assert_eq!(
        linked.begin_ctrl_id, 627_272_811,
        "앞 구역에서 열린 필드의 ctrl_id 를 이어야 HWPX 직렬화기가 마커를 낸다"
    );

    let unpaired = &sections[1].paragraphs[1].orphan_field_ends[0];
    assert_eq!(
        unpaired.begin_id_ref, 0,
        "짝이 없는 마커는 0 으로 남아 #5252 가드가 계속 버린다"
    );
}
