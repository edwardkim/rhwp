//! Native paper settings for the section that owns a page or an explicit section.

use serde::{Deserialize, Serialize};

use crate::document_core::native_interaction::NativeTextPosition;
use crate::model::document::Section;
use crate::model::page::{BindingMethod, PageDef};

use super::editor::{NativeEditOutcome, NativeEditor};
use super::{NativeDocumentSession, NativeHostError};

pub const MIN_PAPER_SIZE: u32 = 720;
pub const MAX_PAPER_SIZE: u32 = 288_000;

/// Dimensions use HWPUNIT (7200 per inch). The renderer swaps width and height
/// when `landscape` is set. Binding is 0 (single), 1 (duplex) or 2 (top flip).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePageSetupSettings {
    pub width: u32,
    pub height: u32,
    pub margin_left: u32,
    pub margin_right: u32,
    pub margin_top: u32,
    pub margin_bottom: u32,
    pub margin_header: u32,
    pub margin_footer: u32,
    pub margin_gutter: u32,
    pub landscape: bool,
    pub binding: u8,
}

impl From<&PageDef> for NativePageSetupSettings {
    fn from(page: &PageDef) -> Self {
        Self {
            width: page.width,
            height: page.height,
            margin_left: page.margin_left,
            margin_right: page.margin_right,
            margin_top: page.margin_top,
            margin_bottom: page.margin_bottom,
            margin_header: page.margin_header,
            margin_footer: page.margin_footer,
            margin_gutter: page.margin_gutter,
            landscape: page.landscape,
            binding: match page.binding {
                BindingMethod::SingleSided => 0,
                BindingMethod::DuplexSided => 1,
                BindingMethod::TopFlip => 2,
            },
        }
    }
}

impl NativePageSetupSettings {
    fn validate(&self) -> Result<(), NativeHostError> {
        if !(MIN_PAPER_SIZE..=MAX_PAPER_SIZE).contains(&self.width)
            || !(MIN_PAPER_SIZE..=MAX_PAPER_SIZE).contains(&self.height)
        {
            return Err(NativeHostError::InvalidArgument(format!(
                "paper dimensions must be {MIN_PAPER_SIZE}..={MAX_PAPER_SIZE} HWPUNIT"
            )));
        }
        if self.binding > 2 {
            return Err(NativeHostError::InvalidArgument(
                "paper binding must be 0, 1 or 2".into(),
            ));
        }

        let (width, height) = if self.landscape {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        // Match PageAreas: the gutter is lateral and header/footer margins
        // contribute to the top/bottom of the body, respectively.
        let horizontal = u64::from(self.margin_left)
            + u64::from(self.margin_right)
            + u64::from(self.margin_gutter);
        let vertical = u64::from(self.margin_top)
            + u64::from(self.margin_bottom)
            + u64::from(self.margin_header)
            + u64::from(self.margin_footer);
        if horizontal >= u64::from(width) || vertical >= u64::from(height) {
            return Err(NativeHostError::InvalidArgument(
                "paper margins must leave space for the body".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePageSetup {
    pub section: u32,
    pub settings: NativePageSetupSettings,
    /// A supported position for `beginEdit` when this section has editable body text.
    pub start_position: Option<NativeTextPosition>,
}

impl NativeDocumentSession {
    /// Read one zero-based page or section without starting an editor.
    pub fn page_setup(
        &self,
        expected_revision: u64,
        page: Option<u32>,
        section: Option<u32>,
    ) -> Result<NativePageSetup, NativeHostError> {
        self.check_revision(expected_revision)?;
        let section = match (page, section) {
            (Some(page), None) => {
                self.check_page(page)?;
                self.core.find_page(page)?.0.section_index as u32
            }
            (None, Some(section)) => section,
            _ => {
                return Err(NativeHostError::InvalidArgument(
                    "pageSetup requires exactly one of page or section".into(),
                ));
            }
        };
        let model = self.page_setup_section(section)?;
        let start_position = model.paragraphs.iter().enumerate().find_map(|(index, _)| {
            let position = NativeTextPosition {
                section,
                paragraph: u32::try_from(index).ok()?,
                scalar_offset: 0,
                cell_path: Vec::new(),
            };
            self.core
                .check_native_interaction_support(&position)
                .ok()
                .map(|()| position)
        });
        Ok(NativePageSetup {
            section,
            settings: (&model.section_def.page_def).into(),
            start_position,
        })
    }

    fn page_setup_section(&self, section: u32) -> Result<&Section, NativeHostError> {
        self.core
            .document()
            .sections
            .get(section as usize)
            .ok_or_else(|| NativeHostError::InvalidArgument("section index out of range".into()))
    }
}

impl NativeEditor {
    pub(super) fn set_page_setup(
        &mut self,
        revision: u64,
        section: u32,
        settings: NativePageSetupSettings,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let model = self.session().page_setup_section(section)?;
        if NativePageSetupSettings::from(&model.section_def.page_def) == settings {
            return Ok(self.outcome(false, false));
        }
        settings.validate()?;
        let json = serde_json::to_string(&settings).map_err(|error| {
            NativeHostError::InvalidArgument(format!("invalid page settings: {error}"))
        })?;
        let selection = self.selection().clone();
        self.edit_explicit_ranges_transaction(revision, |core| {
            core.set_page_def_native(section as usize, &json)?;
            Ok(selection)
        })
    }
}
