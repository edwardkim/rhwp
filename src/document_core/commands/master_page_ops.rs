//! 바탕쪽 만들기·지우기·조회.
//!
//! 바탕쪽 문단의 글 편집은 머리말·꼬리말 편집 API가 `applyTo = MASTER_PAGE_APPLY_BASE + n`
//! 으로 맡는다(`header_footer_ops.rs`). 이 모듈은 바탕쪽 목록 자체와 모델 접근을 둔다.

use crate::document_core::{
    header_footer_apply_from_u8, header_footer_apply_to_u8, DocumentCore, MASTER_PAGE_APPLY_BASE,
};
use crate::error::HwpError;
use crate::model::header_footer::{HeaderFooterApply, MasterPage};
use crate::model::paragraph::Paragraph;
use crate::parser::tags::HWPTAG_LIST_HEADER;

/// 바탕쪽 하나의 조회 JSON 조각.
fn master_page_json(index: usize, master: &MasterPage) -> String {
    format!(
        "{{\"index\":{},\"applyTo\":{},\"isExtension\":{},\"overlap\":{},\"replaceBase\":{},\"front\":{},\"pageNumber\":{},\"paraCount\":{}}}",
        index,
        header_footer_apply_to_u8(master.apply_to),
        master.is_extension,
        // 확장 바탕쪽은 대신 그리지 않을 때만 겹친다(HWPX pageDuplicate).
        master.overlap && !(master.is_extension && master.replace_base),
        master.replace_base,
        master.page_front,
        master
            .optional_page_number
            .map_or("null".to_string(), |n| n.to_string()),
        master.paragraphs.len()
    )
}

impl DocumentCore {
    pub(crate) fn master_page_ref(
        &self,
        section_idx: usize,
        slot: usize,
    ) -> Result<&MasterPage, HwpError> {
        self.document
            .sections
            .get(section_idx)
            .and_then(|section| section.section_def.master_pages.get(slot))
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "구역 {}의 {}번째 바탕쪽이 없습니다",
                    section_idx, slot
                ))
            })
    }

    /// 고칠 바탕쪽. HWP 원본 레코드가 가진 바탕쪽은 모델이 저장을 맡도록 소유를 넘긴다.
    pub(crate) fn master_page_mut(
        &mut self,
        section_idx: usize,
        slot: usize,
    ) -> Result<&mut MasterPage, HwpError> {
        self.master_page_ref(section_idx, slot)?;
        self.release_raw_master_pages(section_idx)?;
        Ok(&mut self.document.sections[section_idx].section_def.master_pages[slot])
    }

    /// HWP에서 읽은 바탕쪽은 원본 레코드를 그대로 다시 쓴다. 바탕쪽을 고치기 전에 그 레코드(구역 정의
    /// 자식의 기본 바탕쪽과 구역 끝 목록의 확장 바탕쪽)를 버려 모델에서 저장하게 한다. 구역 끝의 메모
    /// 목록은 그대로 둔다.
    fn release_raw_master_pages(&mut self, section_idx: usize) -> Result<(), HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let records = &section.section_def.extra_child_records;
        let top_level = records
            .iter()
            .filter(|raw| raw.tag_id == HWPTAG_LIST_HEADER)
            .map(|raw| raw.level)
            .min();
        let base_start = top_level.map(|top_level| {
            records
                .iter()
                .position(|raw| raw.tag_id == HWPTAG_LIST_HEADER && raw.level == top_level)
                .unwrap_or(records.len())
        });
        if let (Some(top_level), Some(start)) = (top_level, base_start) {
            // 영역이 0×0인 목록은 바탕쪽이 아니다(파서가 건너뛴 메모 목록). 함께 버리면 잃으므로 막는다.
            let all_masters = records[start..]
                .iter()
                .filter(|raw| raw.tag_id == HWPTAG_LIST_HEADER && raw.level == top_level)
                .all(|raw| raw.data.get(8..16).is_some_and(|area| area != [0u8; 8]));
            if !all_masters {
                return Err(HwpError::RenderError(
                    "바탕쪽이 아닌 원본 목록이 함께 저장된 구역입니다".to_string(),
                ));
            }
        }
        let tail_mask = section
            .trailing_lists
            .as_ref()
            .map(|tail| tail.master_list_mask())
            .filter(|mask| mask.contains(&true));
        if base_start.is_none() && tail_mask.is_none() {
            return Ok(());
        }
        // 원본을 버리기 전에 구역 끝 목록의 봉인이 맞았으면, 버린 뒤의 투영으로 다시 봉인한다.
        // 이 구역의 바탕쪽은 이제 모델이 저장하므로 끝 목록과 어긋나지 않는다.
        let sealed = crate::model::section_tail::validate_preservation(&self.document).is_ok();
        let section = &mut self.document.sections[section_idx];
        if let Some(start) = base_start {
            section.section_def.extra_child_records.truncate(start);
        }
        if let Some(mask) = tail_mask {
            if let Some(tail) = section.trailing_lists.as_mut() {
                let mut flags = mask.into_iter();
                tail.records.retain(|_| !flags.next().unwrap_or(false));
                if tail.records.is_empty() {
                    section.trailing_lists = None;
                }
            }
        }
        section.raw_stream = None;
        if sealed {
            let digest = crate::model::section_tail::projection_digest(&self.document);
            for section in &mut self.document.sections {
                if let Some(tail) = section.trailing_lists.as_mut() {
                    tail.projection_seal = Some(digest);
                }
            }
        }
        Ok(())
    }

    /// 구역의 바탕쪽 목록. 반환: `{"ok":true,"masterPages":[{index,applyTo,isExtension,...}]}`
    pub fn get_master_pages_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let items: Vec<String> = section
            .section_def
            .master_pages
            .iter()
            .enumerate()
            .map(|(index, master)| master_page_json(index, master))
            .collect();
        Ok(format!(
            "{{\"ok\":true,\"masterPages\":[{}]}}",
            items.join(",")
        ))
    }

    /// 쪽에서 고칠 바탕쪽. 그 쪽에 그린 바탕쪽이 있으면 그것을(겹쳐 그린 마지막 쪽·임의 쪽 바탕쪽이
    /// 있으면 한컴 12.30처럼 그것을), 감춰 그리지 않았으면 그 쪽 구역(없으면 앞 구역)에서 쪽의 홀짝에
    /// 맞는 바탕쪽을 고른다. 한컴 도움말 "바탕쪽 고치기"처럼 현재 구역에 바탕쪽이 없으면 앞 구역으로
    /// 거슬러 찾는다.
    ///
    /// 반환: `{"ok":true,"sectionIndex":N,"applyTo":16+n}` 또는 `{"ok":true,"found":false}`
    pub fn get_master_page_edit_target_native(&self, page_num: u32) -> Result<String, HwpError> {
        let (page, _, _) = self.find_page(page_num)?;
        let target = page
            .extra_master_pages
            .last()
            .or(page.active_master_page.as_ref())
            .map(|r| (r.section_index, r.master_page_index))
            .or_else(|| {
                let odd = page.page_number % 2 == 1;
                (0..=page.section_index).rev().find_map(|section_idx| {
                    let masters = &self.document.sections[section_idx].section_def.master_pages;
                    let base = |apply| {
                        masters
                            .iter()
                            .position(|m| !m.is_extension && m.apply_to == apply)
                    };
                    base(if odd {
                        HeaderFooterApply::Odd
                    } else {
                        HeaderFooterApply::Even
                    })
                    .or_else(|| base(HeaderFooterApply::Both))
                    .or_else(|| (!masters.is_empty()).then_some(0))
                    .map(|slot| (section_idx, slot))
                })
            });
        Ok(match target {
            Some((section_idx, slot)) => format!(
                "{{\"ok\":true,\"found\":true,\"sectionIndex\":{},\"applyTo\":{}}}",
                section_idx,
                usize::from(MASTER_PAGE_APPLY_BASE) + slot
            ),
            None => "{\"ok\":true,\"found\":false}".to_string(),
        })
    }

    /// 빈 문단 하나를 가진 바탕쪽을 만든다. 같은 종류(같은 기본 종류, 마지막 쪽, 같은 임의 쪽)가 있으면
    /// 오류다. 글 영역은 한컴처럼 구역 용지의 본문 영역이다(머리말·꼬리말 여백 제외).
    ///
    /// options:
    /// - `{"applyTo":0|1|2}`: 기본 바탕쪽(0 양 쪽, 1 짝수 쪽, 2 홀수 쪽)
    /// - `{"kind":"last"}`: 구역의 마지막 쪽 바탕쪽
    /// - `{"kind":"page","pageNumber":n}`: 구역 안 n째 쪽(1부터) 바탕쪽
    /// - 마지막 쪽·임의 쪽은 `"overlap":true`면 기본 바탕쪽 위에 겹쳐 그리고, 아니면 기본 바탕쪽 대신 그린다.
    /// - `"front":true`는 한컴 "인쇄할 때 바탕쪽 맨 위로 보내기"다.
    ///
    /// 반환: `{"ok":true,"applyTo":16+n}`
    pub fn create_master_page_native(
        &mut self,
        section_idx: usize,
        options_json: &str,
    ) -> Result<String, HwpError> {
        use crate::document_core::helpers::{json_bool, json_str, json_u16};
        let invalid = || HwpError::RenderError("바탕쪽 종류를 확인해 주세요".to_string());
        // 확장 바탕쪽의 지정 쪽: None은 마지막 쪽이다.
        let extension: Option<Option<u16>> = match json_str(options_json, "kind").as_deref() {
            None => None,
            Some("last") => Some(None),
            Some("page") => Some(Some(
                json_u16(options_json, "pageNumber")
                    .filter(|number| (1..u16::MAX - 3).contains(number))
                    .ok_or_else(invalid)?,
            )),
            Some(_) => return Err(invalid()),
        };
        let apply = match extension {
            Some(_) => HeaderFooterApply::Both,
            None => header_footer_apply_from_u8(
                json_u16(options_json, "applyTo")
                    .filter(|code| *code <= 2)
                    .ok_or_else(invalid)? as u8,
            ),
        };
        let overlap = extension.is_some() && json_bool(options_json, "overlap").unwrap_or(false);
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        if section
            .section_def
            .master_pages
            .iter()
            .any(|m| match extension {
                None => !m.is_extension && m.apply_to == apply,
                Some(page) => m.is_extension && m.optional_page_number == page,
            })
        {
            return Err(HwpError::RenderError(
                "이미 같은 종류의 바탕쪽이 있습니다".to_string(),
            ));
        }
        let page = &section.section_def.page_def;
        let text_width = (page.width as i64
            - page.margin_left as i64
            - page.margin_right as i64
            - page.margin_gutter as i64)
            .max(0) as u32;
        let text_height = (page.height as i64
            - page.margin_top as i64
            - page.margin_bottom as i64
            - page.margin_header as i64
            - page.margin_footer as i64)
            .max(0) as u32;
        self.release_raw_master_pages(section_idx)?;
        let masters = &mut self.document.sections[section_idx].section_def.master_pages;
        // 한컴 12.30처럼 양 쪽·짝수 쪽·홀수 쪽, 구역 마지막 쪽, 구역 안 쪽 번호 차례로 둔다(이전·이후
        // 바탕쪽 차례이자 저장 차례). 확장 바탕쪽은 만든 차례와 관계없이 이 차례로 저장된다.
        let rank = |is_extension: bool, apply: HeaderFooterApply, page: Option<u16>| {
            if is_extension {
                3 + u32::from(page.unwrap_or(0))
            } else {
                match apply {
                    HeaderFooterApply::Both => 0,
                    HeaderFooterApply::Even => 1,
                    HeaderFooterApply::Odd => 2,
                }
            }
        };
        let new_rank = rank(extension.is_some(), apply, extension.flatten());
        let slot = masters
            .iter()
            .position(|m| rank(m.is_extension, m.apply_to, m.optional_page_number) > new_rank)
            .unwrap_or(masters.len());
        masters.insert(
            slot,
            MasterPage {
                apply_to: apply,
                is_extension: extension.is_some(),
                overlap,
                replace_base: extension.is_some() && !overlap,
                ext_flags: match extension {
                    None => 0,
                    Some(page) => 0x02 | u16::from(overlap) | if page.is_some() { 0x04 } else { 0 },
                },
                page_front: json_bool(options_json, "front").unwrap_or(false),
                optional_page_number: extension.flatten(),
                paragraphs: vec![Paragraph::default()],
                text_width,
                text_height,
                hwpx_page_number: Some(extension.flatten().unwrap_or(0)),
                ..Default::default()
            },
        );
        self.document.sections[section_idx].raw_stream = None;
        self.sync_section_def_controls_from_section(section_idx);
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        Ok(format!(
            "{{\"ok\":true,\"applyTo\":{}}}",
            usize::from(MASTER_PAGE_APPLY_BASE) + slot
        ))
    }

    /// 바탕쪽 하나를 지운다. `apply_to`는 `16 + n`이다.
    pub fn delete_master_page_native(
        &mut self,
        section_idx: usize,
        apply_to: u8,
    ) -> Result<String, HwpError> {
        let slot = crate::document_core::master_page_slot(apply_to)
            .ok_or_else(|| HwpError::RenderError("바탕쪽 번호를 확인해 주세요".to_string()))?;
        self.master_page_mut(section_idx, slot)?;
        self.document.sections[section_idx]
            .section_def
            .master_pages
            .remove(slot);
        self.document.sections[section_idx].raw_stream = None;
        self.sync_section_def_controls_from_section(section_idx);
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        Ok("{\"ok\":true}".to_string())
    }
}
