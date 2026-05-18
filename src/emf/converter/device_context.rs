//! DeviceContext + DcStack + ObjectTable.
//!
//! EMF는 GDI 의미론을 따르며, SaveDC/RestoreDC로 상태 스택을 관리하고,
//! CreatePen/Brush/Font + SelectObject/DeleteObject로 그래픽 객체 핸들을 관리한다.

use std::collections::HashMap;

use crate::emf::parser::objects::{LogBrush, LogFontW, LogPen, XForm};

/// 그래픽 객체 핸들이 참조하는 구체 객체.
#[derive(Debug, Clone)]
pub enum GraphicsObject {
    Pen(LogPen),
    Brush(LogBrush),
    Font(LogFontW),
}

/// 렌더 상태 스냅샷(GDI Device Context 대응).
#[derive(Debug, Clone)]
pub struct DeviceContext {
    pub pen: Option<LogPen>,
    pub brush: Option<LogBrush>,
    pub font: Option<LogFontW>,
    pub text_color: u32,
    pub bk_color: u32,
    pub bk_mode: u32,    // 1=Transparent, 2=Opaque
    pub text_align: u32, // bitflags
    pub map_mode: u32,

    // 좌표계
    pub world_xform: XForm,
    pub window_org: (i32, i32),
    pub window_ext: (i32, i32),
    pub viewport_org: (i32, i32),
    pub viewport_ext: (i32, i32),
    pub current_pos: (i32, i32),
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self {
            pen: None,
            brush: None,
            font: None,
            text_color: 0x00_00_00_00,
            bk_color: 0x00_FF_FF_FF,
            bk_mode: 2, // Opaque
            text_align: 0,
            map_mode: 1, // MM_TEXT
            world_xform: XForm::identity(),
            window_org: (0, 0),
            window_ext: (1, 1),
            viewport_org: (0, 0),
            viewport_ext: (1, 1),
            current_pos: (0, 0),
        }
    }
}

/// SaveDC/RestoreDC 스택.
#[derive(Debug, Default)]
pub struct DcStack {
    current: DeviceContext,
    stack: Vec<DeviceContext>,
}

impl DcStack {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&mut self) {
        self.stack.push(self.current.clone());
    }

    /// EMR_RESTOREDC `iRelative` 규약:
    /// - 음수: 상대(−1 = 가장 최근 Save)
    /// - 양수: 절대 깊이 (1 기반)
    ///
    /// 단계 11은 음수(상대)만 지원. pop 개수 = `|relative|`.
    pub fn restore(&mut self, relative: i32) -> bool {
        if relative == 0 {
            return false;
        }
        let n = if relative < 0 {
            (-relative) as usize
        } else {
            return false;
        };
        if self.stack.len() < n {
            return false;
        }
        let target_idx = self.stack.len() - n;
        self.stack.truncate(target_idx + 1);
        if let Some(dc) = self.stack.pop() {
            self.current = dc;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn current(&self) -> &DeviceContext {
        &self.current
    }
    pub fn current_mut(&mut self) -> &mut DeviceContext {
        &mut self.current
    }
    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

/// 객체 핸들 테이블.
#[derive(Debug, Default)]
pub struct ObjectTable {
    handles: HashMap<u32, GraphicsObject>,
}

impl ObjectTable {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, handle: u32, obj: GraphicsObject) {
        self.handles.insert(handle, obj);
    }
    #[must_use]
    pub fn get(&self, handle: u32) -> Option<&GraphicsObject> {
        self.handles.get(&handle)
    }
    pub fn remove(&mut self, handle: u32) -> Option<GraphicsObject> {
        self.handles.remove(&handle)
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.handles.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}
