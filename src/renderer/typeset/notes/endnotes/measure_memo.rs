//! 같은 상태 세대에서 반복된 미주 단 측정 질의를 다시 배치하지 않는다.
//!
//! fit 판정들은 같은 문단을 두고 같은 단과 같은 후보를 여러 번 실제 배치 경로로
//! 재생한다. 측정은 상태를 바꾸지 않으므로 상태 세대가 같고 후보·예산이 같으면
//! 결과도 같다. 세대가 바뀌면 이전 결과를 모두 버린다.

use crate::renderer::typeset::{MeasuredTable, PageItem, Paragraph, ResolvedStyleSet};

use super::measure::EndnoteColumnGeometry;

/// 한 세대에서 보관하는 질의 수. 판정 한 번의 재질의만 받으면 충분하다.
const ENTRY_CAPACITY: usize = 64;

/// 측정 결과를 정하는 caller 입력. 상태 값은 세대로 대신한다.
#[derive(PartialEq)]
pub(super) struct ColumnQuery {
    /// 같은 구역 조판 동안 빌린 문단·스타일의 주소. 세대 안에서 바뀌지 않는다.
    paragraphs: (usize, usize),
    styles: usize,
    available: u64,
    column_width: u64,
    extra_items: String,
    sequential_compact_rewind: bool,
    /// 아직 방출하지 않은 문단은 caller가 매번 만들 수 있으므로 내용으로 비교한다.
    future: Option<(String, i32, bool)>,
    /// 본문 표 측정은 구역 원장의 같은 조각이다. 임시로 이어 붙인 목록도
    /// 주소·길이·소유 순서가 같으면 같은 세대에서 같은 원장이다.
    measured_tables: (usize, usize, Vec<(usize, usize)>),
}

impl ColumnQuery {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        column_width: f64,
        extra_items: &[PageItem],
        sequential_compact_rewind: bool,
        future: Option<(&[Paragraph], i32, bool)>,
        measured_tables: &[MeasuredTable],
    ) -> Self {
        Self {
            paragraphs: (paragraphs.as_ptr() as usize, paragraphs.len()),
            styles: styles as *const ResolvedStyleSet as usize,
            available: available.to_bits(),
            column_width: column_width.to_bits(),
            extra_items: format!("{extra_items:?}"),
            sequential_compact_rewind,
            future: future.map(|(tail, start, compact)| (format!("{tail:?}"), start, compact)),
            measured_tables: (
                measured_tables.as_ptr() as usize,
                measured_tables.len(),
                measured_tables
                    .iter()
                    .map(|table| (table.para_index, table.control_index))
                    .collect(),
            ),
        }
    }
}

/// 잉크 측정 질의. 구역 셀 문맥도 scratch 배치에 실리므로 함께 비교한다.
#[derive(PartialEq)]
pub(super) struct ExtentQuery {
    paragraphs: (usize, usize),
    styles: usize,
    available: u64,
    column_width: u64,
    extra_item: String,
    section_column_spacing_hu: Option<i32>,
}

impl ExtentQuery {
    pub(super) fn new(
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        column_width: f64,
        extra_item: Option<&PageItem>,
        section_column_spacing_hu: Option<i32>,
    ) -> Self {
        Self {
            paragraphs: (paragraphs.as_ptr() as usize, paragraphs.len()),
            styles: styles as *const ResolvedStyleSet as usize,
            available: available.to_bits(),
            column_width: column_width.to_bits(),
            extra_item: format!("{extra_item:?}"),
            section_column_spacing_hu,
        }
    }
}

#[derive(Default)]
pub(in crate::renderer::typeset) struct EndnoteMeasureMemo {
    revision: u64,
    columns: Vec<(ColumnQuery, Option<EndnoteColumnGeometry>)>,
    extents: Vec<(ExtentQuery, Option<(f64, Option<f64>)>)>,
}

impl EndnoteMeasureMemo {
    fn sync(&mut self, revision: u64) {
        if self.revision != revision {
            self.revision = revision;
            self.columns.clear();
            self.extents.clear();
        }
    }

    pub(super) fn column(
        &mut self,
        revision: u64,
        query: &ColumnQuery,
    ) -> Option<Option<EndnoteColumnGeometry>> {
        self.sync(revision);
        self.columns
            .iter()
            .find_map(|(key, value)| (key == query).then_some(*value))
    }

    pub(super) fn store_column(
        &mut self,
        revision: u64,
        query: ColumnQuery,
        value: Option<EndnoteColumnGeometry>,
    ) {
        self.sync(revision);
        if self.columns.len() >= ENTRY_CAPACITY {
            self.columns.clear();
        }
        self.columns.push((query, value));
    }

    pub(super) fn extent(
        &mut self,
        revision: u64,
        query: &ExtentQuery,
    ) -> Option<Option<(f64, Option<f64>)>> {
        self.sync(revision);
        self.extents
            .iter()
            .find_map(|(key, value)| (key == query).then_some(*value))
    }

    pub(super) fn store_extent(
        &mut self,
        revision: u64,
        query: ExtentQuery,
        value: Option<(f64, Option<f64>)>,
    ) {
        self.sync(revision);
        if self.extents.len() >= ENTRY_CAPACITY {
            self.extents.clear();
        }
        self.extents.push((query, value));
    }
}
