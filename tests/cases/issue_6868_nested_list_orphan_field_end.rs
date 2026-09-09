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
use rhwp::parser::hwpx::section::parse_hwpx_section;

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
