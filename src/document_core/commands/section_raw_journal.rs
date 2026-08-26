//! 구역 raw 저널 — #5769 Stage 4.
//!
//! set_section_def / set_page_def 는 적용 시 `section.raw_stream = None` 으로
//! passthrough 를 무효화한다(`apply_section_def_json`, `set_page_def_native`).
//! 무효화된 채 저장하면 직렬화기가 IR 재구성 경로로 바뀌어 원본 한컴 바이트와
//! 어긋난다 — **속성을 하나도 바꾸지 않고 같은 값을 한 번 적용만 해도** 저장
//! 바이트가 달라진다(실측: 표본 hongbo @1144,
//! `tests/cases/issue_5769_stage4_setter_convergence.rs`).
//!
//! 이 저널은 속성 변경 **직전** 의 구역 raw 스트림+출처 봉인을 보관했다가, old
//! 속성 재적용(raw 재무효화) 뒤 되돌려 "속성쌍 역연산 + passthrough 복원"으로
//! 스냅샷 없이 저장 바이트를 수렴시킨다.
//!
//! # page_def 는 대상이 아니다
//!
//! page_def(기하 키)는 wrap 폭 변화 시 본문 전체를 재래핑해 저장 line_segs 를
//! 통째로 재작성한다([#4956] `reflow_body_paragraphs_in_section`). 원본 파일의
//! 한컴 줄 나눔은 rhwp 조판값과 다르므로 raw 복원만으로 수렴하지 않는다(같은
//! 실측에서 len 562176→561664 잔류). page_setup·page_margin 은 스냅샷 잔류다
//! (그림 회전 선례 준용).
//!
//! # 계약
//!
//! - 캡처는 반드시 setter 호출 **전**, 복원은 old 재적용 **후** 에 한다. 캡처와
//!   복원 사이에 그 구역의 다른 편집이 끼면 안 된다(TS 히스토리의 선형
//!   execute/undo 가 보장하는 엄격 짝).
//! - 복원 전제 검증: 현재 raw 가 무효화(None) 상태여야 한다 — Some 인 채의 복원
//!   요청은 이중 undo 같은 배선 버그므로 거부해 무음 중복 복원을 막는다.
//! - 코어는 자동 축출하지 않는다 — TS 히스토리가 스택에서 내보낼 때
//!   `discardSectionRaw` 로 해제하는 것이 계약이다(delete fragment 와 동일).

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::raw_provenance::SectionSeal;

/// 구역 passthrough 1회분 캡처.
#[derive(Debug, Clone)]
pub struct SectionRawCapture {
    /// 대상 구역
    pub section_idx: usize,
    /// 변경 직전 구역 raw 스트림(None 이면 원래도 None)
    pub raw_stream: Option<Vec<u8>>,
    /// raw_stream 출처 봉인 — raw 와 짝으로 복원한다(#4493 계약)
    pub raw_provenance: Option<SectionSeal>,
}

impl DocumentCore {
    /// 속성 변경 직전 구역 raw 를 보관하고 ID 를 돌려준다.
    ///
    /// 반드시 `set_section_def_native` / `set_page_def_native` 호출 **전** 에 불린다.
    pub fn capture_section_raw_native(&mut self, section_idx: usize) -> Result<u32, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        let capture = SectionRawCapture {
            section_idx,
            raw_stream: section.raw_stream.clone(),
            raw_provenance: section.raw_provenance.clone(),
        };
        let id = self.next_section_raw_id;
        self.next_section_raw_id += 1;
        self.section_raw_store.push((id, capture));
        Ok(id)
    }

    /// 캡처한 구역 raw 를 되돌린다 — old 속성 재적용(raw 재무효화) **뒤** 에 불린다.
    ///
    /// 저널 항목은 소비된다(제거). redo 가 다시 execute 하면 그때 다시 캡처한다.
    pub fn restore_section_raw_native(&mut self, capture_id: u32) -> Result<String, HwpError> {
        let pos = self
            .section_raw_store
            .iter()
            .position(|(id, _)| *id == capture_id);
        let capture = match pos {
            Some(pos) => self.section_raw_store[pos].1.clone(),
            None => {
                return Err(HwpError::RenderError(format!(
                    "구역 raw 캡처 {} 없음",
                    capture_id
                )))
            }
        };

        let section = self
            .document
            .sections
            .get_mut(capture.section_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "캡처 {} 의 구역 {} 이 현재 문서에 없습니다",
                    capture_id, capture.section_idx
                ))
            })?;
        // 전제 검증 — setter 가 무효화한 직후(None)여야 복원이 성립한다.
        if section.raw_stream.is_some() {
            return Err(HwpError::RenderError(format!(
                "구역 raw 캡처 {} 의 전제가 어긋났습니다 — passthrough 가 이미 살아있다",
                capture_id
            )));
        }
        section.raw_stream = capture.raw_stream;
        section.raw_provenance = capture.raw_provenance;
        // raw 복원이 실제로 끝난 경우에만 소비한다. 전제 오류는 호출자가 같은
        // undo를 재시도하거나 discard 하도록 캡처를 남긴다.
        self.section_raw_store
            .remove(pos.expect("존재를 확인한 캡처 위치"));
        Ok("{\"ok\":true}".to_string())
    }

    /// 저널 항목을 제거하여 메모리를 해제한다.
    ///
    /// TS 히스토리가 엔트리를 축출·클리어할 때 `discardDeleteFragment` 와 짝으로
    /// 호출하는 것이 계약이다.
    pub fn discard_section_raw_native(&mut self, capture_id: u32) {
        self.section_raw_store.retain(|(id, _)| *id != capture_id);
    }
}
