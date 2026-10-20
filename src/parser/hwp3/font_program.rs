//! 확인된 HWP3 원본 글꼴의 프로그램 종류를 공통 모델에 보존한다.

use crate::model::style::Font;

pub(super) fn preserve_source_kind(font: &mut Font, original: &str, normalized: &str) {
    // 명칭 정규화 대상은 기존에 확인한 원본 HFT다. 이름이 그대로인
    // HCI Poppy도 원본 HWP3의 한컴 Print에서 Type3 프로그램으로 확인했다.
    // 근거: hwp3-sample16-2020.pdf, T3 xref31, Widths xref1738.
    // 이름 변경 여부를 프로그램 종류로 대신하면 표시용 Palatino의 폭으로
    // 되돌아간다. 미확인 이름과 HWP5/HWPX의 TTF 선언에는 적용하지 않는다.
    if original != normalized || original.trim() == "HCI Poppy" {
        font.alt_type = 2;
    }
}
