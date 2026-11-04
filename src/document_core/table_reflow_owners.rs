//! 재조판 표식의 수명과 셀 텍스트 편집에서 얻은 복원 권한을 함께 관리한다.

use super::TableTextReflowKey;
use std::{collections::HashSet, ops::Deref};

#[derive(Default)]
pub(crate) struct TableReflowOwners {
    active: HashSet<TableTextReflowKey>,
    cell_text: HashSet<TableTextReflowKey>,
}

impl Deref for TableReflowOwners {
    type Target = HashSet<TableTextReflowKey>;

    fn deref(&self) -> &Self::Target {
        &self.active
    }
}

impl TableReflowOwners {
    /// 상속·구조 변경의 권위는 셀 텍스트 복원으로 해제할 수 없다.
    pub(crate) fn insert(&mut self, key: TableTextReflowKey) -> bool {
        self.cell_text.remove(&key);
        self.active.insert(key)
    }

    pub(crate) fn insert_cell_text(&mut self, key: TableTextReflowKey) {
        if !self.active.contains(&key) || self.cell_text.contains(&key) {
            self.cell_text.insert(key);
        }
        self.active.insert(key);
    }

    pub(crate) fn remove(&mut self, key: &TableTextReflowKey) -> bool {
        self.cell_text.remove(key);
        self.active.remove(key)
    }

    pub(crate) fn restore_cell_text(&mut self, key: &TableTextReflowKey) -> bool {
        if !self.cell_text.contains(key) {
            return false;
        }
        self.remove(key)
    }

    pub(crate) fn has_cell_text(&self) -> bool {
        !self.cell_text.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.cell_text.clear();
        self.active.clear();
    }

    pub(crate) fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.active.try_reserve(additional)?;
        self.cell_text.try_reserve(additional)
    }
}

impl Extend<TableTextReflowKey> for TableReflowOwners {
    fn extend<T: IntoIterator<Item = TableTextReflowKey>>(&mut self, iter: T) {
        for key in iter {
            self.insert(key);
        }
    }
}
