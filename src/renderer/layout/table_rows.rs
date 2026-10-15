//! 불변 표를 빌린 계산 범위에서만 살아 있는 행 탐색 결과.
//!
//! 캐시는 Weak만 보유한다. Strong 소유자는 항상 같은 Table의 borrow를
//! 함께 보유하므로 임시 표가 소멸한 뒤 주소가 재사용돼도 결과가 남지 않는다.

use crate::model::table::Table;
use crate::model::HwpUnit;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Default)]
struct RowMembers {
    starting: Vec<usize>,
    single_by_column: Vec<usize>,
}

struct RowIndex {
    rows: HashMap<usize, RowMembers>,
    spanning: Vec<usize>,
    raw_heights: OnceLock<Arc<Vec<HwpUnit>>>,
    declared_shrink: OnceLock<Arc<Vec<HwpUnit>>>,
}

impl RowIndex {
    fn new(table: &Table) -> Self {
        let mut rows = HashMap::<usize, RowMembers>::new();
        let mut spanning = Vec::new();
        for (index, cell) in table.cells.iter().enumerate() {
            let row = rows.entry(usize::from(cell.row)).or_default();
            row.starting.push(index);
            if cell.row_span == 1 {
                row.single_by_column.push(index);
            } else if cell.row_span > 1 {
                spanning.push(index);
            }
        }
        for row in rows.values_mut() {
            // 원래 row_cut_cell_order와 같은 안정 정렬. 중복 열의 원본 순서도 보존한다.
            row.single_by_column
                .sort_by_key(|&index| table.cells[index].col);
        }
        Self {
            rows,
            spanning,
            raw_heights: OnceLock::new(),
            declared_shrink: OnceLock::new(),
        }
    }
}

pub(crate) struct TableRows<'a> {
    table: &'a Table,
    index: Arc<RowIndex>,
}

// Drop glue alone does not keep the Table borrow live: NLL may end it after
// the last field use while the Arc still owns an index. An explicit destructor
// makes drop checking retain that borrow until this strong owner is dropped.
impl Drop for TableRows<'_> {
    fn drop(&mut self) {}
}

impl TableRows<'_> {
    pub(super) fn starting(&self, row: usize) -> &[usize] {
        self.index
            .rows
            .get(&row)
            .map_or(&[], |members| &members.starting)
    }

    pub(super) fn single_by_column(&self, row: usize) -> &[usize] {
        self.index
            .rows
            .get(&row)
            .map_or(&[], |members| &members.single_by_column)
    }

    pub(super) fn spanning(&self) -> &[usize] {
        &self.index.spanning
    }

    pub(super) fn raw_heights(&self) -> Arc<Vec<HwpUnit>> {
        Arc::clone(
            self.index
                .raw_heights
                .get_or_init(|| Arc::new(self.table.get_raw_row_heights())),
        )
    }

    pub(super) fn declared_shrink(&self) -> Arc<Vec<HwpUnit>> {
        Arc::clone(
            self.index
                .declared_shrink
                .get_or_init(|| Arc::new(self.table.rowspan_declared_overflow_shrink())),
        )
    }
}

#[derive(Default)]
pub(super) struct TableRowIndexCache {
    entries: RefCell<HashMap<usize, Weak<RowIndex>>>,
}

impl TableRowIndexCache {
    pub(super) fn lookup<'a>(&self, table: &'a Table) -> Option<TableRows<'a>> {
        let key = table as *const Table as usize;
        let index = self.entries.borrow().get(&key)?.upgrade()?;
        Some(TableRows { table, index })
    }

    pub(super) fn scope<'a>(&self, table: &'a Table) -> TableRows<'a> {
        if let Some(rows) = self.lookup(table) {
            return rows;
        }
        let index = Arc::new(RowIndex::new(table));
        self.entries
            .borrow_mut()
            .insert(table as *const Table as usize, Arc::downgrade(&index));
        TableRows { table, index }
    }

    pub(super) fn clear(&self) {
        self.entries.borrow_mut().clear();
    }
}
