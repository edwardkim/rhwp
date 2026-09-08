//! 페이지 범위 추출 — 대형 문서의 결함을 이분법으로 좁히기 위한 도구.
//!
//! 384쪽 문서가 저장 후 한컴에서 열리지 않을 때(#3565), "어떤 요소가 방아쇠인가"를
//! 알려면 문서를 절반씩 잘라 재현 여부를 봐야 한다. 그때 필요한 것이 이 기능이다.
//!
//! **쪽 단위로 자르되 문단 단위로 지운다** — 한 문단이 여러 쪽에 걸치면 그 문단이 닿는
//! 쪽 중 하나라도 범위 안이면 남긴다. 잘라 낸 결과의 쪽수가 요청 범위와 정확히 같을
//! 필요는 없다(레이아웃이 다시 흐르므로). 이 도구의 목적은 **재현 최소화**이지 정밀한
//! 페이지 오려내기가 아니다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;

/// 추출 결과 요약.
#[derive(Debug, Clone, PartialEq)]
pub struct PageExtractReport {
    /// 원본 쪽수
    pub pages_before: u32,
    /// 추출 후 쪽수
    pub pages_after: u32,
    /// 남긴 문단 수
    pub kept: usize,
    /// 지운 문단 수
    pub removed: usize,
    /// [#6840] 정리 전 `BinData` 항목 수
    pub bin_data_before: usize,
    /// [#6840] 정리 후 `BinData` 항목 수
    pub bin_data_after: usize,
}

impl DocumentCore {
    /// [#3565] `from`..=`to`(1 기준) 쪽에 걸친 문단만 남기고 나머지를 지운다.
    ///
    /// 범위 밖이거나 `from > to` 면 오류. 남길 문단이 없으면 구역의 마지막 문단은
    /// 지울 수 없다는 기존 제약에 걸리므로, 각 구역에 최소 1개는 남긴다.
    pub fn extract_page_range(
        &mut self,
        from: u32,
        to: u32,
    ) -> Result<PageExtractReport, HwpError> {
        if from == 0 || to < from {
            return Err(HwpError::RenderError(format!(
                "쪽 범위가 잘못됐습니다: {from}..{to} (1 기준, from <= to)"
            )));
        }
        let pages_before = self.page_count();
        if from > pages_before {
            return Err(HwpError::RenderError(format!(
                "시작 쪽 {from} 이 문서 쪽수 {pages_before} 를 넘습니다"
            )));
        }

        // 구역별로 "남길 문단" 집합을 만든다. 페이지 번호는 구역을 가로질러 이어진다.
        let keep = self.paragraphs_touching_pages(from, to);

        let mut kept = 0usize;
        let mut removed = 0usize;
        for sec_idx in (0..self.document.sections.len()).rev() {
            let total = self.document.sections[sec_idx].paragraphs.len();
            let sec_keep = keep.get(&sec_idx);
            // 인덱스가 밀리지 않도록 뒤에서부터 지운다.
            for pi in (0..total).rev() {
                let want = sec_keep.is_some_and(|s| s.contains(&pi));
                if want {
                    kept += 1;
                    continue;
                }
                // 구역의 마지막 한 문단은 남긴다(기존 제약).
                if self.document.sections[sec_idx].paragraphs.len() <= 1 {
                    kept += 1;
                    continue;
                }
                if self.delete_paragraph_native(sec_idx, pi).is_ok() {
                    removed += 1;
                } else {
                    kept += 1;
                }
            }
        }

        // [#6840] 남은 문단이 참조하지 않는 그림은 버린다. 이 정리가 없으면 쪽을 한
        // 장만 남겨도 `BinData` 가 그대로 따라가 파일이 거의 줄지 않고(6.5MB → 5.2MB),
        // 지운 쪽에 있던 그림 원본이 산출물 안에 남는다 — 부분 제출에서는 유출이다.
        let bin_data = super::bin_data_prune::prune_unreferenced_bin_data(&mut self.document);

        self.invalidate_page_tree_cache();
        self.paginate_if_needed();
        Ok(PageExtractReport {
            pages_before,
            pages_after: self.page_count(),
            kept,
            removed,
            bin_data_before: bin_data.before,
            bin_data_after: bin_data.after,
        })
    }

    /// `from`..=`to` 쪽에 조금이라도 걸치는 문단을 구역별로 모은다.
    fn paragraphs_touching_pages(
        &self,
        from: u32,
        to: u32,
    ) -> std::collections::HashMap<usize, std::collections::HashSet<usize>> {
        let mut keep: std::collections::HashMap<usize, std::collections::HashSet<usize>> =
            Default::default();
        let mut global_page = 0u32;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            let body_len = self.section_render_paragraphs(sec_idx).len();
            for page in &pr.pages {
                global_page += 1;
                if global_page < from || global_page > to {
                    continue;
                }
                let set = keep.entry(sec_idx).or_default();
                for col in &page.column_contents {
                    for item in &col.items {
                        // 미주 문단(para_index >= body_len)은 본문 인덱스가 아니라 건너뛴다.
                        // EndnoteSeparator 는 usize::MAX 를 주므로 같은 조건으로 걸러진다.
                        let pi = item.para_index();
                        if pi < body_len {
                            set.insert(pi);
                        }
                    }
                }
            }
        }
        keep
    }
}
