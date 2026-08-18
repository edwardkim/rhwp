//! Locked page-count contracts from PR #4763. Raster work must not move these.

use super::catalog::CorpusId;

/// 2025 행정업무운영 편람 HWP and HWPX.
pub const ADMIN_HANDBOOK_PAGE_COUNT: u16 = 383;
/// `samples/76076_regulatory_analysis.hwp`.
pub const REGULATORY_76076_PAGE_COUNT: u16 = 82;
/// `samples/issue4090/156492236_규제샌드박스_min.hwpx`.
pub const ISSUE4090_PAGE_COUNT: u16 = 17;
/// `samples/task1725/text_footnote_tail_overpagination.{hwp,hwpx}` family.
pub const NOTE_TAIL_PAGE_COUNT: u16 = 0;
/// Policy research report HWPX (`samples/issue2006/...`) is catalogued, not locked
/// by #4763. The helper still exposes the measured oracle count used in #4764.
pub const ISSUE2006_PAGE_COUNT: u16 = 0;
/// #4490 personnel announcement.
pub const ISSUE4490_PAGE_COUNT: u16 = 0;
/// #4491 mixed-complex report.
pub const ISSUE4491_PAGE_COUNT: u16 = 0;

/// Page-count lock published by #4763 / #4764. `None` means the corpus is
/// compared page-by-page without a locked total.
pub fn locked_page_count(corpus: CorpusId) -> Option<u16> {
    match corpus {
        CorpusId::AdminHandbookHwp | CorpusId::AdminHandbookHwpx => Some(ADMIN_HANDBOOK_PAGE_COUNT),
        CorpusId::Regulatory76076 => Some(REGULATORY_76076_PAGE_COUNT),
        CorpusId::Issue4090 => Some(ISSUE4090_PAGE_COUNT),
        CorpusId::Hwp3Sample16
        | CorpusId::Hwp3ToHwp5_2010
        | CorpusId::Hwp3ToHwp5_2018
        | CorpusId::Hwp3ToHwp5_2020
        | CorpusId::Hwp3ToHwp5_2022
        | CorpusId::Hwp3ToHwp5_2024
        | CorpusId::NoteTailHwp
        | CorpusId::NoteTailHwpx
        | CorpusId::Issue2006
        | CorpusId::Issue4490
        | CorpusId::Issue4491 => None,
    }
}

pub fn page_count_matches_lock(corpus: CorpusId, observed: u16) -> bool {
    match locked_page_count(corpus) {
        Some(locked) => observed == locked,
        None => observed > 0,
    }
}
