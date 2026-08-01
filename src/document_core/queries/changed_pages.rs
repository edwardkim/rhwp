//! [#3712] changedPages — 변경 문단 집합이 걸친 페이지 조회 (#3630 P3).
//!
//! 에이전트는 편집 결과를 눈으로 못 본다. 편집 봉투가 "몇 쪽이 바뀌었나"를
//! 지정해야 그 쪽만 렌더해 검증하는 루프가 상수 비용으로 닫힌다.

use std::collections::{BTreeSet, HashMap};

use crate::document_core::DocumentCore;
use crate::renderer::pagination::PageItem;

impl DocumentCore {
    /// 변경 문단 집합 → 그 문단들이 걸친 **모든** 글로벌 페이지 (0 기준, 오름차순).
    ///
    /// grep 의 페이지 인덱스는 첫 쪽만 기억하지만(인용은 시작 위치 기준), 눈검증은
    /// 누락이 거짓 통과를 만들므로 분할 표·다쪽 문단이 걸친 쪽 전부를 담는다 —
    /// 상위집합은 렌더 한 번 더일 뿐이다. 대상 문단이 하나라도 조판 커버리지에
    /// 없으면 None — 부분 목록 금지 (#3630 P3).
    pub fn pages_covering_paragraphs(&mut self, targets: &[(usize, usize)]) -> Option<Vec<u32>> {
        if targets.is_empty() {
            return Some(Vec::new());
        }
        // 편집 커맨드는 recompose 로 dirty 만 남긴다 — 저장 직전 조판으로 맞춘다.
        self.paginate_if_needed();
        let mut index: HashMap<(usize, usize), BTreeSet<u32>> = HashMap::new();
        let mut global_offset = 0u32;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for (local_i, page) in pr.pages.iter().enumerate() {
                let global_page = global_offset + local_i as u32;
                for col in &page.column_contents {
                    for item in &col.items {
                        let para_index = match item {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => Some(*para_index),
                            _ => None,
                        };
                        if let Some(p) = para_index {
                            index.entry((sec_idx, p)).or_default().insert(global_page);
                        }
                    }
                }
            }
            global_offset += pr.pages.len() as u32;
        }
        let mut pages: BTreeSet<u32> = BTreeSet::new();
        for t in targets {
            pages.extend(index.get(t)?);
        }
        Some(pages.into_iter().collect())
    }
}
