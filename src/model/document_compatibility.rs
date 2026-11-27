//! HWP 5 표 55·56과 HWPX `compatibleDocument` 사이의 문서 호환 설정.

use super::document::{DocInfo, RawRecord};
use crate::parser::tags::{HWPTAG_COMPATIBLE_DOCUMENT, HWPTAG_LAYOUT_COMPATIBILITY};
use sha2::{Digest, Sha256};

// HWP 5 표 56의 그룹과 비트 순서. 한컴의 HWP → HWPX 저장에서 실제로
// 방출되는 요소만 기록한다. 공개 XML에 나타나지 않는 비트는 원본 레코드에 보존한다.
const HWPX_LAYOUT_FLAGS: &[(usize, u32, &str)] = &[
    // 글자
    (0, 1 << 0, "hh:applyFontWeightToBold"),
    (0, 1 << 1, "hh:useInnerUnderline"),
    (0, 1 << 2, "hh:fixedUnderlineWidth"),
    (0, 1 << 3, "hh:doNotApplyStrikeoutWithUnderline"),
    (0, 1 << 5, "hh:useLowercaseStrikeout"),
    (0, 1 << 6, "hh:extendLineheightToOffset"),
    (0, 1 << 8, "hh:treatQuotationAsLatin"),
    (0, 1 << 9, "hh:doNotApplyDiacSymMarkOfNoneAndSix"),
    (0, 1 << 11, "hh:adjustSpaceKerningMinus50"),
    // 문단
    (1, 1 << 0, "hh:doNotAlignWhitespaceOnRight"),
    (1, 1 << 1, "hh:doNotAdjustWordInJustify"),
    (1, 1 << 2, "hh:baseCharUnitOnEAsian"),
    (1, 1 << 3, "hh:baseCharUnitOfIndentOnFirstChar"),
    (1, 1 << 4, "hh:adjustLineheightToFont"),
    (1, 1 << 5, "hh:adjustBaselineInFixedLinespacing"),
    (1, 1 << 6, "hh:applyPrevspacingBeneathObject"),
    (1, 1 << 7, "hh:applyNextspacingOfLastPara"),
    (1, 1 << 8, "hh:applyAtLeastToPercent100Pct"),
    (1, 1 << 9, "hh:doNotApplyAutoSpaceEAsianEng"),
    (1, 1 << 10, "hh:doNotApplyAutoSpaceEAsianNum"),
    (1, 1 << 11, "hh:adjustParaBorderfillToSpacing"),
    (1, 1 << 12, "hh:connectParaBorderfillOfEqualBorder"),
    (1, 1 << 13, "hh:adjustParaBorderOffsetWithBorder"),
    (1, 1 << 14, "hh:extendLineheightToParaBorderOffset"),
    (1, 1 << 15, "hh:applyParaBorderToOutside"),
    (1, 1 << 16, "hh:applyMinColumnWidthTo1mm"),
    (1, 1 << 17, "hh:applyTabPosBasedOnSegment"),
    (1, 1 << 18, "hh:breakTabOverLine"),
    (1, 1 << 19, "hh:adjustVertPosOfLine"),
    (1, 1 << 20, "hh:doNotApplyWhiteSpaceHeight"),
    (1, 1 << 21, "hh:doNotAlignLastPeriod"),
    (1, 1 << 22, "hh:doNotAlignLastForbidden"),
    (1, 1 << 23, "hh:adjustMarginFromAdjustLineheight"),
    // 구역
    (2, 1 << 0, "hh:baseLineSpacingOnLineGrid"),
    (2, 1 << 1, "hh:applyCharSpacingToCharGrid"),
    (2, 1 << 2, "hh:doNotApplyGridInHeaderFooter"),
    (2, 1 << 3, "hh:applyExtendHeaderFooterEachSection"),
    (2, 1 << 4, "hh:doNotApplyHeaderFooterAtNoSpace"),
    (2, 1 << 5, "hh:doNotApplyColSeparatorAtNoGap"),
    (2, 1 << 6, "hh:doNotApplyLinegridAtNoLinespacing"),
    // 개체
    (3, 1 << 0, "hh:doNotApplyImageEffect"),
    (3, 1 << 1, "hh:doNotApplyShapeComment"),
    (3, 1 << 2, "hh:doNotAdjustEmptyAnchorLine"),
    (3, 1 << 3, "hh:overlapBothAllowOverlap"),
    (3, 1 << 4, "hh:doNotApplyVertOffsetOfForward"),
    (3, 1 << 5, "hh:extendVertLimitToPageMargins"),
    (3, 1 << 6, "hh:doNotHoldAnchorOfTable"),
    (3, 1 << 7, "hh:doNotFormattingAtBeneathAnchor"),
    (3, 1 << 8, "hh:adjustBaselineOfObjectToBottom"),
    (3, 1 << 9, "hh:adjustObjectHorizontally"),
    (3, 1 << 10, "hh:doNotAdjustGroupingCtrlCodePos"),
    // 필드
    (4, 1 << 0, "hh:doNotApplyExtensionCharCompose"),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum CompatibilityTarget {
    #[default]
    Hwp201x = 0,
    Hwp200x = 1,
    MsWord = 2,
    Hunmin = 3,
}

impl CompatibilityTarget {
    pub(crate) fn from_xml(name: &str) -> Option<Self> {
        match name {
            "HWP201X" => Some(Self::Hwp201x),
            "HWP200X" => Some(Self::Hwp200x),
            "MS_WORD" => Some(Self::MsWord),
            "Hunmin" => Some(Self::Hunmin),
            _ => None,
        }
    }

    pub(crate) fn xml_name(self) -> &'static str {
        match self {
            Self::Hwp201x => "HWP201X",
            Self::Hwp200x => "HWP200X",
            Self::MsWord => "MS_WORD",
            Self::Hunmin => "Hunmin",
        }
    }

    /// 한컴 2024에서 각 대상의 HWPX를 HWP로 변환하여 확인한 표 56 레코드.
    pub(crate) fn preset(self) -> DocumentCompatibility {
        let flags = match self {
            Self::Hwp201x => [0; 5],
            Self::Hwp200x => [0x020c, 0x0700, 0x30, 0x13, 1],
            Self::MsWord => [0x09e3, 0x00cd_f8ff, 0x4f, 0x07ec, 0],
            Self::Hunmin => [0, 0x31_0001, 0, 0x24, 0],
        };
        DocumentCompatibility {
            target: self,
            flags,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DocumentCompatibility {
    pub(crate) target: CompatibilityTarget,
    pub(crate) flags: [u32; 5],
}

impl DocumentCompatibility {
    pub(crate) fn from_records(info: &DocInfo) -> Self {
        let mut compat = Self::default();
        for record in &info.extra_records {
            match record.tag_id {
                HWPTAG_COMPATIBLE_DOCUMENT => {
                    compat.target = match record
                        .data
                        .get(..4)
                        .and_then(|value| value.try_into().ok())
                        .map(u32::from_le_bytes)
                    {
                        Some(1) => CompatibilityTarget::Hwp200x,
                        Some(2) => CompatibilityTarget::MsWord,
                        Some(3) => CompatibilityTarget::Hunmin,
                        _ => CompatibilityTarget::Hwp201x,
                    };
                }
                HWPTAG_LAYOUT_COMPATIBILITY => {
                    for (flag, bytes) in compat.flags.iter_mut().zip(record.data.chunks_exact(4)) {
                        *flag = u32::from_le_bytes(bytes.try_into().expect("four-byte chunk"));
                    }
                }
                _ => {}
            }
        }
        compat
    }

    pub(crate) fn xml_layout_flags(self) -> impl Iterator<Item = &'static str> {
        HWPX_LAYOUT_FLAGS
            .iter()
            .filter_map(move |&(group, mask, name)| (self.flags[group] & mask != 0).then_some(name))
    }

    pub(crate) const HWPX_ENTRY: &str = "META-INF/rhwp-layout-compatibility";
    pub(crate) const ENCODED_SIZE: usize = 4 + 32 + 20;

    /// 한컴은 Word 대상에 비트가 전부 0인 HWP도 빈 layoutCompatibility로 내보낸다.
    /// 반대로 이 빈 요소를 한컴에서 읽으면 대상 프로그램의 기본 비트가 된다.
    /// HWP의 개별 비트가 기본 프리셋과 다를 때만 원본 20바이트를 헤더에 결속한다.
    pub(crate) fn nonpreset_bytes(self, header: &[u8]) -> Option<[u8; Self::ENCODED_SIZE]> {
        if self == self.target.preset() {
            return None;
        }
        let mut data = [0; Self::ENCODED_SIZE];
        data[..4].copy_from_slice(b"RLC1");
        data[4..36].copy_from_slice(&Sha256::digest(header));
        for (slot, flag) in data[36..].chunks_exact_mut(4).zip(self.flags) {
            slot.copy_from_slice(&flag.to_le_bytes());
        }
        Some(data)
    }

    pub(crate) fn restore_nonpreset(info: &mut DocInfo, header: &[u8], data: &[u8]) {
        if data.len() != Self::ENCODED_SIZE
            || &data[..4] != b"RLC1"
            || data[4..36] != Sha256::digest(header)[..]
        {
            return;
        }
        let Some(record) = info
            .extra_records
            .iter_mut()
            .find(|r| r.tag_id == HWPTAG_LAYOUT_COMPATIBILITY)
        else {
            return;
        };
        record.data.clear();
        record.data.extend_from_slice(&data[36..]);
    }

    /// 표 55의 문서 수준 레코드와 표 56의 자식은 변경 추적 설정보다 앞에 온다.
    pub(crate) fn insert_hwpx_records(self, info: &mut DocInfo, before: usize) {
        let target = RawRecord {
            tag_id: HWPTAG_COMPATIBLE_DOCUMENT,
            level: 0,
            data: (self.target as u32).to_le_bytes().to_vec(),
        };
        let layout = RawRecord {
            tag_id: HWPTAG_LAYOUT_COMPATIBILITY,
            level: 1,
            data: self.flags.into_iter().flat_map(u32::to_le_bytes).collect(),
        };
        info.extra_records.splice(before..before, [target, layout]);
    }
}
