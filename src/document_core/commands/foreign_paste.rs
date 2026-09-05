//! 외부 문서(다른 DocInfo 를 가진 Document)를 현재 문서의 커서 위치에 끼워 넣는다.
//!
//! 한글 클립보드 문서모델을 HWPX 로 옮겨 파싱하면 **서식표가 통째로 다른** Document 가 나온다.
//! 내부 클립보드 붙여넣기(`paste_internal_native`)는 같은 문서 안에서만 성립하므로(서식 id 가
//! 그대로 유효하다고 가정한다), 외부 문서는 서식표를 먼저 현재 문서로 옮겨 심고
//! 문단이 참조하는 id 를 전부 새 번호로 바꿔야 한다.

use crate::error::HwpError;
use crate::model::bin_data::{BinData, BinDataContent, BinDataType};
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::event::DocumentEvent;
use crate::model::image::Picture;
use crate::model::paragraph::{CharShapeRef, Paragraph};
use crate::model::shape::ShapeObject;
use crate::model::style::HeadType;
use std::collections::HashMap;

impl crate::document_core::DocumentCore {
    /// 외부 문서의 본문 문단을 현재 문서 `section_idx`/`para_idx`/`char_offset` 에 삽입한다.
    ///
    /// 세 단계다.
    /// 1. **서식표 이식** — 외부 DocInfo 의 표(바이너리·글꼴·테두리/배경·탭·글자모양·
    ///    번호매기기·글머리표·문단모양·스타일)를 현재 DocInfo 로 옮기며
    ///    `외부 id → 현재 id` 대응표를 만든다. 값이 같은 항목이 이미 있으면 그것을
    ///    재사용하므로 같은 조각을 여러 번 붙여넣어도 표가 늘지 않는다.
    /// 2. **참조 재작성** — 외부 문단과 그 컨트롤(표 셀·그림·도형·캡션·각주·머리말 …)이
    ///    물고 있는 id 를 대응표로 전부 바꾼다. 재귀로 들어가지 않으면 셀 안 글자가
    ///    엉뚱한 글꼴로 그려진다.
    /// 3. **삽입** — `paste_internal_native` 와 같은 방식으로 커서에서 문단을 나눠 끼우고
    ///    reflow · vpos 재계산 · 재구성을 돌린다.
    ///
    /// 반환: `{"ok":true,"paraIdx":N,"charOffset":M}` 형태의 JSON 문자열.
    pub fn paste_foreign_document_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        foreign: Document,
    ) -> Result<String, HwpError> {
        // 인덱스 검증 — `paste_internal_native` 와 같은 문구를 쓴다(스튜디오가 같은 처리).
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }

        // 외부 구역이 여럿이면 이어 붙인다 — 커서 한 곳에 넣는 작업이라 구역 경계를
        // 옮겨 놓을 자리가 없다(구역 설정 컨트롤은 아래에서 떨어져 나간다).
        let mut foreign_paras: Vec<Paragraph> = foreign
            .sections
            .iter()
            .flat_map(|section| section.paragraphs.iter().cloned())
            .collect();
        if foreign_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"외부 문서에 문단이 없다\"}".to_string());
        }

        // ① 서식표 이식. 바이너리 두 목록은 통째로 꺼내 들고 다닌다 — 글꼴(임베드)·
        // 테두리(그림 채우기)·그림/OLE 이 모두 같은 풀을 건드리는데, `self.document` 의
        // 다른 필드를 동시에 빌리면 대여 검사가 막는다.
        let mut pool = BinPool::take(&mut self.document);
        let id_map = merge_foreign_doc_info(&mut self.document, &foreign, &mut pool);

        // **대상이 빈 문서면 원본의 용지·여백을 가져온다.**
        //
        // 붙여넣기는 원칙적으로 대상 문서의 쪽 설정을 따른다(그래서 아래에서 구역 정의를
        // 떼어낸다). 그러나 빈 문서에 문서 전체를 붙이는 경우에는 그 원칙이 사용자가 기대하는
        // 결과와 어긋난다 — 새 문서 양식은 머리말 5mm, 원본은 10mm 라 본문이 18.9px 위에서
        // 시작하고 쪽 경계가 줄줄이 밀렸다. 내용이 있는 문서에 붙일 때는 종전대로 대상 설정을
        // 지킨다(남의 문서 서식을 붙여넣기가 바꾸면 안 된다).
        // 🔴 빈 문서라도 첫 문단은 자기 구역 정의를 갖고 있으므로 그것은 "내용"으로 세지 않는다.
        let target_is_empty = self.document.sections.len() == 1
            && self.document.sections[section_idx]
                .paragraphs
                .iter()
                .all(|p| {
                    p.text.trim().is_empty()
                        && p.controls
                            .iter()
                            .all(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))
                });
        if target_is_empty {
            // 🔴 용지·여백의 정본은 문단 컨트롤이 아니라 `Section.section_def.page_def` 다.
            // 컨트롤만 갈아 끼웠을 때 `spaceColumns`·`tabStop` 은 따라왔는데 `<hp:margin>` 만
            // 대상 값으로 남았다(실측). 참조 id 가 없는 순수 치수 묶음이라 그대로 복사한다.
            if let Some(foreign_section) = foreign.sections.first() {
                self.document.sections[section_idx].section_def.page_def =
                    foreign_section.section_def.page_def.clone();
            }
        }
        // ② 참조 재작성. 구역 정의는 아래에서 떼어내지만, 빈 문서에 옮겨 심을 때를 위해
        // **재작성까지 마친 뒤에** 꺼낸다(서식 참조가 외부 문서 번호로 남으면 안 된다).
        {
            let mut ctx = RemapCtx {
                map: &id_map,
                foreign: &foreign,
                pool: &mut pool,
                next_field_id: next_free_field_id(&self.document),
            };
            ctx.paragraphs(&mut foreign_paras);
        }
        // 구역 정의(secd)·단 정의(cold)는 본문 한가운데 놓일 수 없으므로 떼어낸다.
        for para in &mut foreign_paras {
            strip_section_scoped_controls(para);
        }

        let imported_bin = pool.imported();
        pool.restore(&mut self.document);
        // DocInfo 모델이 바뀌었으므로 저장 시 원본 바이트를 재사용하면 안 된다.
        self.document.doc_info.raw_stream_dirty = true;
        if imported_bin {
            self.bump_bin_data_epoch();
        }
        // 새 서식이 들어왔다. 아래 reflow/compose 가 `self.styles` 를 보므로 먼저 해소한다.
        self.rebuild_resolved_styles();

        // ③ 삽입 — 이하 절차는 `paste_internal_native` 와 같다.
        self.document.sections[section_idx].raw_stream = None;

        // 커서 문단이 **비어 있으면** 첫 외부 문단의 문단모양·글자모양을
        // 물려받는다.
        //
        // 한글 문서의 첫 문단은 구역 정의만 든 빈 문단이고 줄간격이 0% 라 높이를 차지하지
        // 않는다. 그런데 붙여넣기는 구역 정의를 떼어내므로 그 문단은 "빈 문단"이 되어
        // `merge_from` 이 조기 반환하고, 대상 문서의 빈 문단(줄간격 160%·15pt)이 그대로
        // 남아 **본문 전체를 한 줄만큼 밀어낸다**(실측: 원본 첫 줄 130.8 ↔ 붙여넣기 149.9px).
        // 서식만 물려받으면 그 문단이 원본처럼 높이 0 으로 접힌다.
        {
            let target = &self.document.sections[section_idx].paragraphs[para_idx];
            let target_para_is_blank = target.text.trim().is_empty()
                && target
                    .controls
                    .iter()
                    .all(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)));
            if target_para_is_blank && char_offset == 0 {
                let (shape_id, char_shapes) = {
                    let first = &foreign_paras[0];
                    (first.para_shape_id, first.char_shapes.clone())
                };
                let target = &mut self.document.sections[section_idx].paragraphs[para_idx];
                target.para_shape_id = shape_id;
                if !char_shapes.is_empty() {
                    target.char_shapes = char_shapes;
                }
            }
        }

        let foreign_count = foreign_paras.len();

        if foreign_count == 1 && foreign_paras[0].controls.is_empty() {
            // 단일 문단 텍스트 — 커서 문단 안에 글자만 흘려 넣는다(문단모양은 대상 것이 이긴다).
            let text = foreign_paras[0].text.clone();
            let char_shapes = foreign_paras[0].char_shapes.clone();
            let char_offsets = foreign_paras[0].char_offsets.clone();
            let new_chars = text.chars().count();

            self.document.sections[section_idx].paragraphs[para_idx]
                .insert_text_at(char_offset, &text);
            self.apply_clipboard_char_shapes(
                section_idx,
                para_idx,
                char_offset,
                &char_shapes,
                &char_offsets,
                new_chars,
            );

            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();

            let new_offset = char_offset + new_chars;
            self.event_log.push(DocumentEvent::ContentPasted {
                section: section_idx,
                para: para_idx,
            });
            return Ok(crate::document_core::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}",
                para_idx, new_offset
            )));
        }

        // 다중 문단 또는 컨트롤 포함 — 커서에서 문단을 나누고 사이에 끼운다.
        let right_half =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
        self.document.sections[section_idx].paragraphs[para_idx].merge_from(&foreign_paras[0]);

        let mut insert_idx = para_idx + 1;
        for para in foreign_paras.iter().skip(1) {
            self.document.sections[section_idx]
                .paragraphs
                .insert(insert_idx, para.clone());
            insert_idx += 1;
        }

        let last_para_idx = insert_idx - 1;
        let merge_point =
            self.document.sections[section_idx].paragraphs[last_para_idx].merge_from(&right_half);

        for i in para_idx..=last_para_idx {
            self.reflow_paragraph(section_idx, i);
        }

        // [Task #2299] 삽입 구간은 원본 문서의 좌표를 물고 오므로 흐름에 다시 연결한다.
        // 방치하면 이후 편집의 vpos 재계산이 이를 저장 단/쪽 리셋으로 오인한다.
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            Some(para_idx + 1..last_para_idx + 1),
            None,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );

        self.recompose_paragraph(section_idx, para_idx);
        for i in para_idx + 1..=last_para_idx {
            self.insert_composed_paragraph(section_idx, i);
        }
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted {
            section: section_idx,
            para: para_idx,
        });
        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":{}",
            last_para_idx, merge_point
        )))
    }
}

// ───────────────────────────── 서식표 대응 ─────────────────────────────

/// 외부 DocInfo 항목을 현재 표에서 찾을 때 쓰는 **구조 동등성 열쇠**.
///
/// `Font`·`BorderFill`·`Numbering`·`Bullet`·`Style` 에는 `PartialEq` 가 없다(모델 정의).
/// 대신 DocInfo 항목은 전부 `serde::Serialize` 라 직렬화 결과를 한 잣대로 쓴다.
/// `raw_data`(라운드트립용 원본 바이트)는 뺀다 — `CharShape`/`ParaShape` 의 `PartialEq`
/// 가 같은 이유로 제외하는 것과 같은 계약이다.
fn table_key<T: serde::Serialize>(value: &T) -> Option<String> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::Object(mut obj)) => {
            obj.remove("raw_data");
            Some(serde_json::Value::Object(obj).to_string())
        }
        Ok(other) => Some(other.to_string()),
        // 직렬화 실패는 "같은 것이 없다" 로 접는다 — 남의 서식을 잘못 재사용하는 것보다
        // 항목 하나가 더 느는 편이 안전하다.
        Err(_) => None,
    }
}

fn build_table_index<T: serde::Serialize>(items: &[T]) -> HashMap<String, usize> {
    let mut index = HashMap::new();
    for (idx, item) in items.iter().enumerate() {
        if let Some(key) = table_key(item) {
            // 같은 값이 여럿이면 **가장 앞 번호**를 쓴다.
            index.entry(key).or_insert(idx);
        }
    }
    index
}

/// 값이 같은 항목이 있으면 그 번호를, 없으면 새로 붙이고 그 번호를 돌려준다.
fn find_or_push<T: serde::Serialize>(
    dst: &mut Vec<T>,
    index: &mut HashMap<String, usize>,
    candidate: T,
) -> usize {
    if let Some(key) = table_key(&candidate) {
        if let Some(&found) = index.get(&key) {
            return found;
        }
        let new_idx = dst.len();
        index.insert(key, new_idx);
        dst.push(candidate);
        return new_idx;
    }
    let new_idx = dst.len();
    dst.push(candidate);
    new_idx
}

/// `외부 id → 현재 id` 대응표.
///
/// 표마다 기준이 다르다. 글자모양·문단모양·탭·스타일은 0-based 색인이고,
/// 테두리/배경과 문단머리(번호매기기·글머리표)는 **1-based 참조**다(0 = 없음).
/// 글꼴은 언어 슬롯마다 목록이 따로라 슬롯별 표를 따로 둔다.
#[derive(Default)]
struct ForeignIdMap {
    /// `[언어 슬롯][외부 글꼴 번호] = 현재 글꼴 번호`
    fonts: Vec<Vec<u16>>,
    char_shapes: Vec<u32>,
    para_shapes: Vec<u16>,
    /// 0-based 색인끼리의 대응(참조는 1-based 라 `border_fill_ref` 로 변환한다).
    border_fills: Vec<u16>,
    tab_defs: Vec<u16>,
    numberings: Vec<u16>,
    bullets: Vec<u16>,
    styles: Vec<u8>,
}

impl ForeignIdMap {
    /// 대응이 없으면 원래 값을 돌려준다 — 참조가 표 밖을 가리키던 문서를 그대로 통과시켜
    /// 붙여넣기 자체가 실패하지 않게 한다(렌더는 이미 범위 밖 id 를 폴백 처리한다).
    fn font(&self, slot: usize, old: u16) -> u16 {
        self.fonts
            .get(slot)
            .and_then(|s| s.get(old as usize))
            .copied()
            .unwrap_or(old)
    }

    fn char_shape(&self, old: u32) -> u32 {
        self.char_shapes.get(old as usize).copied().unwrap_or(old)
    }

    fn para_shape(&self, old: u16) -> u16 {
        self.para_shapes.get(old as usize).copied().unwrap_or(old)
    }

    fn style(&self, old: u8) -> u8 {
        self.styles.get(old as usize).copied().unwrap_or(old)
    }

    fn tab_def(&self, old: u16) -> u16 {
        self.tab_defs.get(old as usize).copied().unwrap_or(old)
    }

    /// 테두리/배경 참조(1-based, 0 = 없음).
    fn border_fill_ref(&self, old: u16) -> u16 {
        if old == 0 {
            return 0;
        }
        self.border_fills
            .get((old - 1) as usize)
            .and_then(|idx| idx.checked_add(1))
            .unwrap_or(old)
    }

    /// 문단머리 참조(1-based, 0 = 없음). `head_type` 에 따라 보는 표가 갈린다 —
    /// 글머리표면 `bullets`, 개요/번호면 `numberings` 다(같은 필드, 다른 표).
    fn numbering_ref(&self, old: u16, head_type: HeadType) -> u16 {
        if old == 0 {
            return 0;
        }
        let table = if head_type == HeadType::Bullet {
            &self.bullets
        } else {
            &self.numberings
        };
        table
            .get((old - 1) as usize)
            .and_then(|idx| idx.checked_add(1))
            .unwrap_or(old)
    }
}

/// 바이너리 데이터 풀 — 현재 문서에서 꺼내 든 두 목록과 `외부 id → 현재 id` 대응.
///
/// `BinData`(DocInfo 레코드)와 `BinDataContent`(실제 바이트)는 **같은 순번**이 곧 참조
/// id(1-based)라는 규칙으로 묶여 있다(`renderer::layout::utils::find_bin_data`).
/// 그래서 새 항목은 두 목록에 함께 붙이고 `BinDataContent.id` 도 명시한다 —
/// 순번 조회가 빗나가도 id 검색 폴백이 살아 있게 하려는 것이다.
struct BinPool {
    list: Vec<BinData>,
    content: Vec<BinDataContent>,
    map: HashMap<u16, u16>,
    next_storage_id: u16,
    imported: bool,
}

impl BinPool {
    fn take(doc: &mut Document) -> Self {
        let list = std::mem::take(&mut doc.doc_info.bin_data_list);
        let content = std::mem::take(&mut doc.bin_data_content);
        // storage id 는 저장 시 `BIN%04X` 스트림 이름이 되므로 기존 값과 겹치면 안 된다
        // (순번 채번은 구멍 있는 문서에서 충돌한다 — `Document::next_bin_data_storage_id`).
        let next_storage_id = content
            .iter()
            .map(|c| c.id)
            .chain(list.iter().map(|b| b.storage_id))
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        BinPool {
            list,
            content,
            map: HashMap::new(),
            next_storage_id,
            imported: false,
        }
    }

    fn restore(self, doc: &mut Document) {
        doc.doc_info.bin_data_list = self.list;
        doc.bin_data_content = self.content;
    }

    fn imported(&self) -> bool {
        self.imported
    }

    /// 외부 바이너리 참조를 현재 문서 번호로 옮긴다(없으면 그때 실어 들인다).
    fn map_id(&mut self, foreign: &Document, old: u16) -> u16 {
        if old == 0 {
            return 0;
        }
        if let Some(&new_id) = self.map.get(&old) {
            return new_id;
        }
        let Some(src) = find_foreign_bin_content(foreign, old) else {
            // 원본 바이트가 없는 참조는 0(그림 없음)으로 접는다 — 남겨 두면 현재 문서의
            // 엉뚱한 그림을 가리킨다.
            self.map.insert(old, 0);
            return 0;
        };
        let new_id = u16::try_from(self.content.len().max(self.list.len()) + 1).unwrap_or(0);
        if new_id == 0 {
            self.map.insert(old, 0);
            return 0;
        }
        let mut record = foreign
            .doc_info
            .bin_data_list
            .get((old - 1) as usize)
            .cloned()
            .unwrap_or(BinData {
                data_type: BinDataType::Embedding,
                extension: Some(src.extension.clone()),
                ..Default::default()
            });
        record.raw_data = None;
        record.storage_id = self.next_storage_id;
        self.next_storage_id = self.next_storage_id.saturating_add(1);

        self.list.push(record);
        self.content.push(BinDataContent {
            id: new_id,
            data: src.data.clone(),
            extension: src.extension.clone(),
        });
        self.imported = true;
        self.map.insert(old, new_id);
        new_id
    }
}

/// 외부 문서에서 1-based 참조로 바이너리를 찾는다(`find_bin_data_index` 와 같은 규칙).
fn find_foreign_bin_content(foreign: &Document, id: u16) -> Option<&BinDataContent> {
    if id == 0 {
        return None;
    }
    if let Some(found) = foreign.bin_data_content.get((id - 1) as usize) {
        return Some(found);
    }
    foreign.bin_data_content.iter().find(|c| c.id == id)
}

/// 외부 DocInfo 를 현재 DocInfo 로 옮겨 심고 대응표를 만든다.
///
/// 순서가 곧 의존 관계다 — 바이너리 → 글꼴 → 테두리/배경 → 탭 → 글자모양 →
/// 번호매기기·글머리표 → 문단모양 → 스타일. 앞 단계의 대응표가 없으면 뒤 단계에서
/// 참조를 옮길 수 없다.
///
/// 모든 이식본은 `raw_data` 를 비운다. 원본 바이트는 **외부 문서의 id 공간**을 담고
/// 있어서, 남겨 두면 직렬화기가 필드 대신 그 바이트를 써 재작성한 번호가 사라진다
/// (HTML 붙여넣기에서 같은 결함이 이미 회귀 고정돼 있다 — `html_import.rs` 시험).
fn merge_foreign_doc_info(
    current: &mut Document,
    foreign: &Document,
    pool: &mut BinPool,
) -> ForeignIdMap {
    let mut map = ForeignIdMap::default();

    // ① 글꼴 — 🔴 언어 슬롯마다 목록이 다르고 같은 번호가 슬롯마다 다른 글꼴이다.
    // 슬롯 하나를 골라 7칸에 복사하면 한자·일어 구간이 엉뚱한 글꼴로 그려진다.
    while current.doc_info.font_faces.len() < foreign.doc_info.font_faces.len() {
        current.doc_info.font_faces.push(Vec::new());
    }
    for (slot, foreign_slot) in foreign.doc_info.font_faces.iter().enumerate() {
        let dst = &mut current.doc_info.font_faces[slot];
        let mut index = build_table_index(dst);
        let mut slot_map = Vec::with_capacity(foreign_slot.len());
        for font in foreign_slot {
            let mut candidate = font.clone();
            candidate.raw_data = None;
            if let Some(bin) = candidate.resolved_bin_data_id {
                candidate.resolved_bin_data_id = Some(pool.map_id(foreign, bin));
            }
            let idx = find_or_push(dst, &mut index, candidate);
            slot_map.push(u16::try_from(idx).unwrap_or(0));
        }
        map.fonts.push(slot_map);
    }

    // ② 테두리/배경 — 그림 채우기가 바이너리를 참조한다.
    {
        let dst = &mut current.doc_info.border_fills;
        let mut index = build_table_index(dst);
        for border_fill in &foreign.doc_info.border_fills {
            let mut candidate = border_fill.clone();
            candidate.raw_data = None;
            if let Some(image) = &mut candidate.fill.image {
                image.bin_data_id = pool.map_id(foreign, image.bin_data_id);
            }
            let idx = find_or_push(dst, &mut index, candidate);
            map.border_fills.push(u16::try_from(idx).unwrap_or(0));
        }
    }

    // ③ 탭 정의 (문단모양이 0-based 로 참조한다).
    {
        let dst = &mut current.doc_info.tab_defs;
        let mut index = build_table_index(dst);
        for tab_def in &foreign.doc_info.tab_defs {
            let mut candidate = tab_def.clone();
            candidate.raw_data = None;
            let idx = find_or_push(dst, &mut index, candidate);
            map.tab_defs.push(u16::try_from(idx).unwrap_or(0));
        }
    }

    // ④ 글자모양 — 글꼴(슬롯별)과 테두리/배경을 참조한다.
    {
        let dst = &mut current.doc_info.char_shapes;
        let mut index = build_table_index(dst);
        for char_shape in &foreign.doc_info.char_shapes {
            let mut candidate = char_shape.clone();
            candidate.raw_data = None;
            for (slot, font_id) in candidate.font_ids.iter_mut().enumerate() {
                *font_id = map.font(slot, *font_id);
            }
            candidate.border_fill_id = map.border_fill_ref(candidate.border_fill_id);
            let idx = find_or_push(dst, &mut index, candidate);
            map.char_shapes.push(u32::try_from(idx).unwrap_or(0));
        }
    }

    // ⑤ 번호매기기 — 수준별 머리 모양이 글자모양을 참조한다.
    {
        let dst = &mut current.doc_info.numberings;
        let mut index = build_table_index(dst);
        for numbering in &foreign.doc_info.numberings {
            let mut candidate = numbering.clone();
            candidate.raw_data = None;
            for head in candidate.heads.iter_mut() {
                head.char_shape_id = map.char_shape(head.char_shape_id);
            }
            let idx = find_or_push(dst, &mut index, candidate);
            map.numberings.push(u16::try_from(idx).unwrap_or(0));
        }
    }

    // ⑥ 글머리표.
    {
        let dst = &mut current.doc_info.bullets;
        let mut index = build_table_index(dst);
        for bullet in &foreign.doc_info.bullets {
            let mut candidate = bullet.clone();
            candidate.raw_data = None;
            candidate.char_shape_id = map.char_shape(candidate.char_shape_id);
            let idx = find_or_push(dst, &mut index, candidate);
            map.bullets.push(u16::try_from(idx).unwrap_or(0));
        }
        current.doc_info.bullet_count = current.doc_info.bullets.len() as u32;
    }

    // ⑦ 문단모양 — 탭·문단머리·테두리를 참조한다.
    {
        let dst = &mut current.doc_info.para_shapes;
        let mut index = build_table_index(dst);
        for para_shape in &foreign.doc_info.para_shapes {
            let mut candidate = para_shape.clone();
            candidate.raw_data = None;
            candidate.tab_def_id = map.tab_def(candidate.tab_def_id);
            candidate.numbering_id = map.numbering_ref(candidate.numbering_id, candidate.head_type);
            candidate.border_fill_id = map.border_fill_ref(candidate.border_fill_id);
            let idx = find_or_push(dst, &mut index, candidate);
            map.para_shapes.push(u16::try_from(idx).unwrap_or(0));
        }
    }

    // ⑧ 스타일 — 문단모양·글자모양을 참조한다.
    let styles_before = current.doc_info.styles.len();
    {
        let dst = &mut current.doc_info.styles;
        let mut index = build_table_index(dst);
        for style in &foreign.doc_info.styles {
            let mut candidate = style.clone();
            candidate.raw_data = None;
            candidate.para_shape_id = map.para_shape(candidate.para_shape_id);
            candidate.char_shape_id =
                u16::try_from(map.char_shape(candidate.char_shape_id as u32)).unwrap_or(0);
            let idx = find_or_push(dst, &mut index, candidate);
            // 스타일 참조는 u8 이다. 256개를 넘기면 바탕글(0)로 접는다 — 실문서에서 이
            // 한계를 넘는 경우는 없고, 넘겼을 때 엉뚱한 스타일을 가리키는 편이 더 나쁘다.
            map.styles.push(u8::try_from(idx).unwrap_or(0));
        }
    }
    // `next_style_id` 는 스타일 표 자기 자신을 가리켜 앞의 한 번으로는 풀 수 없다
    // (뒤 번호를 가리키는 앞 스타일이 있다). 전부 심은 뒤 새로 생긴 것만 되짚는다.
    for (foreign_idx, &new_idx) in map.styles.iter().enumerate() {
        if (new_idx as usize) < styles_before {
            continue; // 재사용한 기존 스타일은 이미 현재 문서 번호를 쓴다.
        }
        let next = foreign
            .doc_info
            .styles
            .get(foreign_idx)
            .map(|s| s.next_style_id)
            .unwrap_or(0);
        if let Some(style) = current.doc_info.styles.get_mut(new_idx as usize) {
            style.next_style_id = map.style(next);
        }
    }

    // ⑨ 메모 모양 — 모델에 목록이 없고 개수와 원본 XML 블록만 보존한다.
    // 개수는 큰 쪽을 쓰고, 현재 문서에 블록이 없을 때만 외부 블록을 받아 둔다
    // (두 블록을 합치려면 XML 을 파싱해야 하는데 그건 이 경로의 범위 밖이다).
    current.doc_info.memo_shape_count = current
        .doc_info
        .memo_shape_count
        .max(foreign.doc_info.memo_shape_count);
    if current.doc_info.memo_properties_xml.is_none() {
        current
            .doc_info
            .memo_properties_xml
            .clone_from(&foreign.doc_info.memo_properties_xml);
    }

    map
}

// ───────────────────────────── 참조 재작성 ─────────────────────────────

/// 문단 트리 전체를 훑으며 id 를 대응표로 바꾸는 순회 상태.
struct RemapCtx<'a> {
    map: &'a ForeignIdMap,
    foreign: &'a Document,
    pool: &'a mut BinPool,
    /// 누름틀 필드 id 는 문서 안에서 유일해야 한다 — 현재 문서 최대값 다음부터 채번한다.
    next_field_id: u32,
}

impl RemapCtx<'_> {
    fn paragraphs(&mut self, paragraphs: &mut [Paragraph]) {
        for para in paragraphs {
            para.para_shape_id = self.map.para_shape(para.para_shape_id);
            para.style_id = self.map.style(para.style_id);
            for char_shape in &mut para.char_shapes {
                char_shape.char_shape_id = self.map.char_shape(char_shape.char_shape_id);
            }
            for ctrl in &mut para.controls {
                self.control(ctrl);
            }
        }
    }

    fn control(&mut self, ctrl: &mut Control) {
        match ctrl {
            Control::Table(table) => {
                table.border_fill_id = self.map.border_fill_ref(table.border_fill_id);
                for zone in &mut table.zones {
                    zone.border_fill_id = self.map.border_fill_ref(zone.border_fill_id);
                }
                for cell in &mut table.cells {
                    cell.border_fill_id = self.map.border_fill_ref(cell.border_fill_id);
                    self.paragraphs(&mut cell.paragraphs);
                }
                if let Some(caption) = &mut table.caption {
                    self.paragraphs(&mut caption.paragraphs);
                }
            }
            Control::Picture(picture) => self.picture(picture),
            Control::Shape(shape) => self.shape(shape),
            Control::Header(header) => self.paragraphs(&mut header.paragraphs),
            Control::Footer(footer) => self.paragraphs(&mut footer.paragraphs),
            Control::Footnote(footnote) => self.paragraphs(&mut footnote.paragraphs),
            Control::Endnote(endnote) => self.paragraphs(&mut endnote.paragraphs),
            Control::HiddenComment(comment) => self.paragraphs(&mut comment.paragraphs),
            Control::Field(field) => {
                field.field_id = self.next_field_id;
                self.next_field_id = self.next_field_id.saturating_add(1).max(1);
                if let Some(residue) = &mut field.guide_residue {
                    residue.char_shape_id = self.map.char_shape(residue.char_shape_id);
                }
                self.paragraphs(&mut field.memo_paragraphs);
            }
            Control::CharOverlap(overlap) => {
                for id in &mut overlap.char_shape_ids {
                    *id = self.map.char_shape(*id);
                }
            }
            // 덧말의 `styleIDRef` 는 스타일 표를 가리킨다(HWP5 컨트롤 데이터의 style id).
            Control::Ruby(ruby) => {
                if let Ok(old) = u8::try_from(ruby.style_id_ref) {
                    ruby.style_id_ref = u16::from(self.map.style(old));
                }
            }
            _ => {}
        }
    }

    fn picture(&mut self, picture: &mut Picture) {
        picture.image_attr.bin_data_id = self
            .pool
            .map_id(self.foreign, picture.image_attr.bin_data_id);
        if let Some(caption) = &mut picture.caption {
            self.paragraphs(&mut caption.paragraphs);
        }
    }

    fn shape(&mut self, shape: &mut ShapeObject) {
        match shape {
            ShapeObject::Picture(picture) => self.picture(picture),
            ShapeObject::Ole(ole) => {
                if let Ok(old) = u16::try_from(ole.bin_data_id) {
                    ole.bin_data_id = u32::from(self.pool.map_id(self.foreign, old));
                }
                if let Some(caption) = &mut ole.caption {
                    self.paragraphs(&mut caption.paragraphs);
                }
            }
            ShapeObject::Group(group) => {
                for child in &mut group.children {
                    self.shape(child);
                }
                if let Some(caption) = &mut group.caption {
                    self.paragraphs(&mut caption.paragraphs);
                }
            }
            ShapeObject::Chart(chart) => {
                if let Some(caption) = &mut chart.caption {
                    self.paragraphs(&mut caption.paragraphs);
                }
            }
            _ => {}
        }
        // 그룹·그림은 `drawing_mut()` 이 None 이라 위 갈래가 캡션을 맡는다. 나머지는
        // 채우기(그림)·글상자·캡션이 여기 달려 있다 — 두 경로가 겹치지 않는다.
        if let Some(drawing) = shape.drawing_mut() {
            if let Some(image) = &mut drawing.fill.image {
                image.bin_data_id = self.pool.map_id(self.foreign, image.bin_data_id);
            }
            if let Some(text_box) = &mut drawing.text_box {
                self.paragraphs(&mut text_box.paragraphs);
            }
            if let Some(caption) = &mut drawing.caption {
                self.paragraphs(&mut caption.paragraphs);
            }
        }
    }
}

/// 현재 문서에서 아직 쓰지 않은 누름틀 필드 id.
fn next_free_field_id(doc: &Document) -> u32 {
    let mut max_id = 0u32;
    for section in &doc.sections {
        collect_max_field_id(&section.paragraphs, &mut max_id);
    }
    max_id.saturating_add(1).max(1)
}

fn collect_max_field_id(paragraphs: &[Paragraph], max_id: &mut u32) {
    for para in paragraphs {
        for ctrl in &para.controls {
            collect_max_field_id_from_control(ctrl, max_id);
        }
    }
}

/// `RemapCtx::control` 과 같은 문단 트리를 훑어 field id 최대값을 수집한다.
///
/// 새 필드 id는 이 순회가 본 문서에서 이미 쓰인 모든 id보다 커야 한다. 재작성 경로가
/// 닿는 하위 문단을 하나라도 빼면 붙여넣은 필드가 기존 field id와 충돌한다.
fn collect_max_field_id_from_control(ctrl: &Control, max_id: &mut u32) {
    match ctrl {
        Control::Field(field) => {
            *max_id = (*max_id).max(field.field_id);
            collect_max_field_id(&field.memo_paragraphs, max_id);
        }
        Control::Table(table) => {
            for cell in &table.cells {
                collect_max_field_id(&cell.paragraphs, max_id);
            }
            if let Some(caption) = &table.caption {
                collect_max_field_id(&caption.paragraphs, max_id);
            }
        }
        Control::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                collect_max_field_id(&caption.paragraphs, max_id);
            }
        }
        Control::Shape(shape) => collect_max_field_id_from_shape(shape, max_id),
        Control::Header(header) => collect_max_field_id(&header.paragraphs, max_id),
        Control::Footer(footer) => collect_max_field_id(&footer.paragraphs, max_id),
        Control::Footnote(footnote) => collect_max_field_id(&footnote.paragraphs, max_id),
        Control::Endnote(endnote) => collect_max_field_id(&endnote.paragraphs, max_id),
        Control::HiddenComment(comment) => collect_max_field_id(&comment.paragraphs, max_id),
        _ => {}
    }
}

fn collect_max_field_id_from_shape(shape: &crate::model::shape::ShapeObject, max_id: &mut u32) {
    match shape {
        crate::model::shape::ShapeObject::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                collect_max_field_id(&caption.paragraphs, max_id);
            }
        }
        crate::model::shape::ShapeObject::Ole(ole) => {
            if let Some(caption) = &ole.caption {
                collect_max_field_id(&caption.paragraphs, max_id);
            }
        }
        crate::model::shape::ShapeObject::Group(group) => {
            for child in &group.children {
                collect_max_field_id_from_shape(child, max_id);
            }
            if let Some(caption) = &group.caption {
                collect_max_field_id(&caption.paragraphs, max_id);
            }
        }
        crate::model::shape::ShapeObject::Chart(chart) => {
            if let Some(caption) = &chart.caption {
                collect_max_field_id(&caption.paragraphs, max_id);
            }
        }
        _ => {}
    }
    if let Some(drawing) = shape.drawing() {
        if let Some(text_box) = &drawing.text_box {
            collect_max_field_id(&text_box.paragraphs, max_id);
        }
        if let Some(caption) = &drawing.caption {
            collect_max_field_id(&caption.paragraphs, max_id);
        }
    }
}

/// 구역 정의(secd)·단 정의(cold)를 떼어낸다.
///
/// 이 둘은 구역 첫 문단에만 붙는 **구역 범위** 컨트롤이라 본문 한가운데 끼면 그 자리에
/// 구역이 갈린다. 내부 클립보드도 같은 정규화를 거치므로(`clipboard.rs` 의
/// `strip_structural_controls_for_text_clipboard`) 같은 함수를 쓴다.
///
/// 그 함수는 컨트롤만 떼고 텍스트 축의 자리표시자는 두는데, 한글 클립보드 조각은
/// **언제나** `secPr` 을 갖고 있어(HWPX 파서가 `\u{0002}` 한 글자와 8 코드유닛을 함께
/// 세운다) 자리표시자를 남기면 커서 자리에 보이지 않는 글자가 하나 들어간다.
/// 그래서 선행 슬롯만 함께 지운다.
fn strip_section_scoped_controls(para: &mut Paragraph) {
    let leading = para
        .controls
        .iter()
        .take_while(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))
        .count();
    let has_structural = para
        .controls
        .iter()
        .any(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)));
    if !has_structural {
        return;
    }
    super::clipboard::strip_structural_controls_for_text_clipboard(para);
    drop_leading_control_slots(para, leading);
}

/// 문단 맨 앞의 컨트롤 자리표시자 `n` 개를 텍스트 축에서 지운다.
///
/// 슬롯 하나는 글자 `\u{0002}` 하나 + 코드유닛 8칸이다. 그 규칙이 그대로 지켜진
/// 문단에서만 손댄다 — 어긋난 문단은 손대는 쪽이 더 위험하므로 그냥 둔다.
fn drop_leading_control_slots(para: &mut Paragraph, max_slots: usize) {
    if max_slots == 0 {
        return;
    }
    let chars: Vec<char> = para.text.chars().collect();
    let slots = chars
        .iter()
        .take(max_slots)
        .take_while(|&&c| c == '\u{0002}')
        .count();
    if slots == 0 || para.char_offsets.len() != chars.len() {
        return;
    }
    if (0..slots).any(|i| para.char_offsets[i] != (i as u32) * 8) {
        return;
    }

    let shift = (slots as u32) * 8;
    para.text = chars[slots..].iter().collect();
    para.char_offsets = para.char_offsets[slots..]
        .iter()
        .map(|off| off.saturating_sub(shift))
        .collect();
    para.char_count = para.char_count.saturating_sub(shift);

    // 글자모양·범위태그는 코드유닛 축, 필드/제목표시는 글자 축이다.
    let mut kept: Vec<CharShapeRef> = Vec::with_capacity(para.char_shapes.len());
    let mut carried: Option<u32> = None;
    for cs in &para.char_shapes {
        if cs.start_pos < shift {
            carried = Some(cs.char_shape_id);
            continue;
        }
        if cs.start_pos == shift {
            carried = None;
        }
        kept.push(CharShapeRef {
            start_pos: cs.start_pos - shift,
            char_shape_id: cs.char_shape_id,
        });
    }
    if let Some(char_shape_id) = carried {
        kept.insert(
            0,
            CharShapeRef {
                start_pos: 0,
                char_shape_id,
            },
        );
    }
    para.char_shapes = kept;

    for seg in &mut para.line_segs {
        seg.text_start = seg.text_start.saturating_sub(shift);
    }
    for tag in &mut para.range_tags {
        tag.start = tag.start.saturating_sub(shift);
        tag.end = tag.end.saturating_sub(shift);
    }
    for range in &mut para.field_ranges {
        range.start_char_idx = range.start_char_idx.saturating_sub(slots);
        range.end_char_idx = range.end_char_idx.saturating_sub(slots);
    }
    for mark in &mut para.title_marks {
        mark.char_idx = mark.char_idx.saturating_sub(slots);
    }
}

impl crate::document_core::DocumentCore {
    /// 한글 클립보드 문서모델(JSON) → HWPX 조각 → 파싱 → 현재 문서에 삽입.
    ///
    /// 이 함수가 붙여넣기 경로의 진입점이다. 어느 단계에서든 실패하면 오류를 돌려주고,
    /// 호출한 쪽(스튜디오)은 종전 HTML 붙여넣기로 되돌아간다.
    pub fn paste_hwp_json_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        let parts = crate::document_core::hwpjson::hwpjson_to_hwpx_parts(json)?;
        let foreign = parts_to_document(&parts)?;
        let result =
            self.paste_foreign_document_native(section_idx, para_idx, char_offset, foreign)?;
        // 위임 대상이 이미 패스스루를 무효화하지만, 그 계약은 이 함수 본문에서도 보이게 둔다
        // (#2724 가드는 `pub fn (&mut self)` 마다 본문에서 직접 확인한다). 붙여넣기가 IR 을
        // 바꿨으므로 저장 시 원본 바이트를 그대로 내보내면 안 된다.
        self.document.doc_info.raw_stream_dirty = true;
        Ok(result)
    }
}

/// HWPX 조각(header/section XML + 그림) → 메모리 안 `Document`.
///
/// zip 패키지를 만들지 않는다 — 파서가 XML 문자열 진입점을 이미 갖고 있어
/// (`parse_hwpx_header` / `parse_hwpx_section`) 그대로 태우는 편이 빠르고 실패 지점도 적다.
fn parts_to_document(
    parts: &crate::document_core::hwpjson::HwpxParts,
) -> Result<Document, HwpError> {
    let (doc_info, doc_properties) =
        crate::parser::hwpx::header::parse_hwpx_header(&parts.header_xml)?;
    let section = crate::parser::hwpx::section::parse_hwpx_section(&parts.section_xml)?;
    let mut doc = Document {
        doc_info,
        doc_properties,
        sections: vec![section],
        is_hwpx_variant: true,
        ..Default::default()
    };
    for (id, mime, bytes) in &parts.bins {
        // 확장자는 MIME 뒤쪽을 쓴다(image/png → png). 파서·직렬화가 확장자로 형식을 판단한다.
        let extension = mime.rsplit('/').next().unwrap_or("png").to_string();
        doc.bin_data_content
            .push(crate::model::bin_data::BinDataContent {
                id: *id,
                data: crate::model::bin_data::BinDataBytes::from_shared(bytes.clone()),
                extension,
            });
    }
    Ok(doc)
}
