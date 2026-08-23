//! 선택 삭제 조각(fragment) 저장소 — #5769 Stage 1.
//!
//! 삭제의 **참 역연산**: 제거되기 직전의 문단 범위 원본을 통째로 보관하고, undo 시
//! 그 자리에 되돌려 끼운다(LibreOffice `SwNodes` undo 배열과 동일한 묘지 패턴).
//! 스냅샷([`Document`] 클론, 문서 크기 비용)과 달리 조각 비용은 **삭제된 내용 +
//! 뒤따르는 줄 좌표 저널**에만 비례한다.
//!
//! # 왜 사전 캡처인가
//!
//! [`Paragraph::delete_text_at`] 은 char_shapes 를 클램핑한 뒤 같은 start_pos 에 몰린
//! ref 를 병합한다(#3576/#4271 — 규약상 의도된 동작). 한 번 병합되면 삭제 전 경계를
//! 사후에 재구성할 수 없다(ProseMirror `Step.invert(docBefore)` 가 "이전 문서 없이는
//! 반전 불가"라는 것과 같은 이유). 그래서 캡처는 반드시 삭제 **직전** 에 한다.
//!
//! # 왜 꼬리(tail) 줄 좌표 저널이 필요한가
//!
//! `delete_range_native` 는 삭제 후 `recalculate_section_vpos` 를 start_para 부터
//! 구역 끝까지 돌려 뒤따르는 모든 문단의 `line_segs[].vertical_pos` 를 새 흐름에 맞게
//! 덮어쓴다. HWP5 직렬화기는 그 값을 그대로 기록하므로(`body_text.rs` LINE_SEG),
//! 범위만 되돌려서는 뒤 문단의 저장 바이트가 원본과 달라진다. 조각은 삭제 범위 뒤
//! 문단들의 line_segs 스냅샷을 함께 들고 되돌린다.
//!
//! # 계약
//!
//! - 엄격 LIFO: 캡처와 복원 사이에 같은 구역의 다른 구조 편집이 끼면 안 된다.
//!   위반을 구조적으로 잡기 위해 복원 시 "현재 문단 수 == 캡처 시 문단 수 − (범위 길이−1)"
//!   을 검증한다(범위 삭제는 항상 병합으로 잔여 1문단을 남긴다).
//! - 조각은 TS 히스토리가 스택에서 내보낼 때 `discardDeleteFragment` 로 해제한다.
//!   코어는 자동 축출하지 않는다 — 스냅샷과 달리 MB 급이 아니므로 무통보 축출(#2328
//!   클래스)을 만들 이유가 없다.
//! - 셀 내부 삭제(cell_ctx)는 아직 스냅샷 폴백이다(Stage 3에서 조각 소비자로 확대).

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::raw_provenance::SectionSeal;

/// 선택 삭제 1회분의 복원 조각.
#[derive(Debug, Clone)]
pub struct DeleteFragment {
    /// 대상 구역
    pub section_idx: usize,
    /// 삭제 범위 시작 문단 (포함)
    pub start_para: usize,
    /// 삭제 범위 끝 문단 (포함)
    pub end_para: usize,
    /// 캡처 시점 구역 문단 수 — 복원 전제 검증용
    pub pre_para_count: usize,
    /// 삭제 직전 원본 문단들 — `paragraphs[start..=end]` 전체.
    /// line_segs·char_shapes·controls·ctrl_data_records·range_tags 등
    /// 문단 소속 전 필드를 통째로 되돌린다.
    pub captured_paras: Vec<Paragraph>,
    /// 삭제 범위 **뒤** 문단들의 line_segs 스냅샷 — index 0 이 `end_para+1`.
    /// `recalculate_section_vpos` 가 start_para 이후 vertical_pos 를 덮어쓰는 것을 되돌린다.
    pub tail_line_segs: Vec<Vec<LineSeg>>,
    /// 삭제 직전 구역 raw 스트림(None 이면 원래도 None). delete 가 None 으로 박는 것을 되돌려
    /// 저장 바이트 왕복 동일성을 지킨다.
    pub raw_stream: Option<Vec<u8>>,
    /// raw_stream 출처 봉인 — raw_stream 과 짝으로 복원한다(#4493 계약).
    pub raw_provenance: Option<SectionSeal>,
    /// 삭제 직전 캐럿 — delete 가 doc_properties.caret_* 을 옮기는데, DocInfo 봉인
    /// 다이제스트가 DocProperties 전체를 포함하므로(`raw_provenance.rs`
    /// `doc_info_model_digest`) 이것도 되돌려야 DocInfo raw 재사용이 살아난다.
    pub caret: (u32, u32),
}

impl DocumentCore {
    /// 삭제 직전 문단 범위 원본을 조각으로 보관하고 조각 ID 를 반환한다.
    ///
    /// 반드시 `delete_range_native` **호출 전**에 불린다. 호출 순서:
    /// capture → delete → (성공 시 조각 보관 / 실패 시 discard).
    pub fn capture_delete_range_native(
        &mut self,
        section_idx: usize,
        start_para: usize,
        end_para: usize,
    ) -> Result<u32, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if start_para > end_para {
            return Err(HwpError::RenderError(format!(
                "조각 범위 뒤집힘 (start={} > end={})",
                start_para, end_para
            )));
        }
        if end_para >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "조각 범위 초과 (end={}, 총 {}문단)",
                end_para,
                section.paragraphs.len()
            )));
        }

        let captured_paras = section.paragraphs[start_para..=end_para].to_vec();
        let tail_line_segs = section.paragraphs[end_para + 1..]
            .iter()
            .map(|p| p.line_segs.clone())
            .collect();
        let fragment = DeleteFragment {
            section_idx,
            start_para,
            end_para,
            pre_para_count: section.paragraphs.len(),
            captured_paras,
            tail_line_segs,
            raw_stream: section.raw_stream.clone(),
            raw_provenance: section.raw_provenance.clone(),
            caret: (
                self.document.doc_properties.caret_list_id,
                self.document.doc_properties.caret_para_id,
            ),
        };

        let id = self.next_fragment_id;
        self.next_fragment_id += 1;
        self.fragment_store.push((id, fragment));
        Ok(id)
    }

    /// 조각을 원래 자리에 되돌려 끼운다 — 삭제의 참 역연산.
    ///
    /// 범위 앞뒤는 그대로 두고 `[start..=end]` 자리에 캡처 원본을 다시 끼우고,
    /// 뒤 문단의 line_segs 와 구역 raw 필드를 캡처 시점 값으로 되돌린 뒤
    /// 파생 상태를 재구성한다(스냅샷 복원과 동일 경로).
    pub fn restore_delete_fragment_native(&mut self, frag_id: u32) -> Result<String, HwpError> {
        let pos = self
            .fragment_store
            .iter()
            .position(|(id, _)| *id == frag_id)
            .ok_or_else(|| HwpError::RenderError(format!("삭제 조각 {} 없음", frag_id)))?;
        // 전제 검증이 실패하면 호출자가 원인을 해소한 뒤 같은 undo 를 다시 시도할 수
        // 있어야 한다. 따라서 항목은 성공한 복원 끝에서만 소비한다.
        let frag = self.fragment_store[pos].1.clone();

        if frag.section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "삭제 조각 {} 의 구역 {} 이 현재 문서에 없습니다",
                frag_id, frag.section_idx
            )));
        }
        let section = &mut self.document.sections[frag.section_idx];
        // 전제 검증 — 삭제가 적용된 직후 형태(잔여 1문단 병합)인지 확인한다.
        // 캡처 후 삭제가 실패했거나 다른 편집이 끼었으면 여기서 거부해 무음 중복 삽입을 막는다.
        let expected_post_count = frag.pre_para_count - (frag.end_para - frag.start_para);
        if section.paragraphs.len() != expected_post_count {
            return Err(HwpError::RenderError(format!(
                "삭제 조각 {} 의 전제가 어긋났습니다 (현재 {}문단 != 예상 {}문단) — \
                 캡처와 복원 사이에 다른 편집이 끼었거나 삭제가 적용되지 않았습니다",
                frag_id,
                section.paragraphs.len(),
                expected_post_count
            )));
        }
        if frag.start_para >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "삭제 조각 {} 의 시작 문단 {} 이 현재 문서에 없습니다",
                frag_id, frag.start_para
            )));
        }

        // [start] 자리의 삭제 후 잔여(병합) 문단을 원본으로 교체 + 나머지 원본 재삽입
        let mut restored: Vec<Paragraph> = Vec::with_capacity(frag.pre_para_count);
        restored.extend_from_slice(&section.paragraphs[..frag.start_para]);
        restored.extend(frag.captured_paras.iter().cloned());
        restored.extend_from_slice(&section.paragraphs[frag.start_para + 1..]);
        section.paragraphs = restored;

        // 꼬리 줄 좌표 저널 복원 — recalculate_section_vpos 의 덮어쓰기를 되돌린다
        for (offset, segs) in frag.tail_line_segs.iter().enumerate() {
            let pi = frag.end_para + 1 + offset;
            if pi < section.paragraphs.len() {
                section.paragraphs[pi].line_segs = segs.clone();
            }
        }

        // 구역 raw 필드 복원 — 저장 바이트 왕복 동일성의 핵심
        section.raw_stream = frag.raw_stream.clone();
        section.raw_provenance = frag.raw_provenance.clone();

        let cursor_para = frag.start_para;
        let caret = frag.caret;

        // 캐럿 복원 — DocInfo 봉인 다이제스트가 DocProperties 전체를 포함하므로
        // 되돌리지 않으면 raw 재사용이 깨져 IR 재직렬화 폴백으로 바이트가 어긋난다.
        self.document.doc_properties.caret_list_id = caret.0;
        self.document.doc_properties.caret_para_id = caret.1;

        // 파생 상태 재구성 — 스냅샷 복원과 동일 경로(compose·paginate·캐시 비움).
        // 여기서 recalculate_section_vpos 를 다시 돌리지 않는다 — 원본 캐시 좌표가
        // 이미 조각으로 복원됐으므로 재계산은 오히려 바이트를 어긋나게 한다.
        self.rebuild_derived_state();

        // 성공한 경우에만 조각을 소비한다. 앞선 전제 오류에서는 저장소 항목을
        // 보존해 CommandHistory 가 동일 ID로 재시도하거나 명시적으로 discard 할 수 있다.
        self.fragment_store.remove(pos);

        self.event_log.push(DocumentEvent::TextInserted {
            section: frag.section_idx,
            para: cursor_para,
            offset: 0,
            len: 0,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            cursor_para
        )))
    }

    /// 조각을 저장소에서 제거하여 메모리를 해제한다.
    ///
    /// TS 히스토리가 엔트리를 축출·클리어할 때 스냅샷 `discardSnapshot` 과 짝으로
    /// 호출하는 것이 계약이다. 코어는 자동 축출하지 않는다.
    pub fn discard_delete_fragment_native(&mut self, frag_id: u32) {
        self.fragment_store.retain(|(id, _)| *id != frag_id);
    }
}
