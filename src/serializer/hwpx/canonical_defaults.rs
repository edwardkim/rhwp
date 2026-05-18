//! HWPX 속성 기본값 및 enum 상수 테이블.
//!
//! Default values and enum definitions referenced from
//! [hancom-io/hwpx-owpml-model](https://github.com/hancom-io/hwpx-owpml-model)
//! (Apache License 2.0, © 2022 Hancom Inc.).
//!
//! 각 상수는 한컴 OWPML `Class/` 하위 C++ 클래스의 constructor 초기화 리스트 또는
//! `enumdef.h`에서 추출되었다. rhwp는 코드를 복사하지 않고 스펙 정보만 참조한다.
//!
//! 상세: `mydocs/tech/hwpx_hancom_reference.md`

#![allow(dead_code)]

// =====================================================================
// CharShapeType (Class/Head/CharShapeType.cpp:31)
// =====================================================================
pub const CHARSHAPE_HEIGHT: u32 = 1000;
pub const CHARSHAPE_TEXT_COLOR: u32 = 0x000000;
pub const CHARSHAPE_SHADE_COLOR: u32 = 0xFFFFFF;
pub const CHARSHAPE_USE_FONT_SPACE: bool = false;
pub const CHARSHAPE_USE_KERNING: bool = false;
pub const CHARSHAPE_SYM_MARK: u32 = 0; // SMT_NONE

// =====================================================================
// ParaShapeType (Class/Head/ParaShapeType.cpp:31)
// ⚠️ snapToGrid 는 기본값이 true — 유일한 true 기본값 속성
// =====================================================================
pub const PARASHAPE_SNAP_TO_GRID: bool = true;
pub const PARASHAPE_FONT_LINE_HEIGHT: bool = false;
pub const PARASHAPE_SUPPRESS_LINE_NUMBERS: bool = false;
pub const PARASHAPE_CHECKED: bool = false;
pub const PARASHAPE_CONDENSE: u32 = 0;
pub const PARASHAPE_TAB_PR_ID_REF: u16 = 0;

// =====================================================================
// BorderFillType (Class/Head/BorderFillType.cpp:31)
// =====================================================================
pub const BORDERFILL_THREE_D: bool = false;
pub const BORDERFILL_SHADOW: bool = false;
pub const BORDERFILL_BREAK_CELL_SEPARATE_LINE: bool = false;
pub const BORDERFILL_CENTER_LINE: u32 = 0;

// =====================================================================
// BreakSetting (Class/Head/breakSetting.cpp:32)
// =====================================================================
pub const BREAKSETTING_WIDOW_ORPHAN: bool = false;
pub const BREAKSETTING_KEEP_WITH_NEXT: bool = false;
pub const BREAKSETTING_KEEP_LINES: bool = false;
pub const BREAKSETTING_PAGE_BREAK_BEFORE: bool = false;
pub const BREAKSETTING_BREAK_NON_LATIN_WORD: u32 = 0;
pub const BREAKSETTING_LINE_WRAP: u32 = 0;

// =====================================================================
// Visibility (Class/Para/visibility.cpp:49)
// =====================================================================
pub const VISIBILITY_HIDE_FIRST_HEADER: bool = false;
pub const VISIBILITY_HIDE_FIRST_FOOTER: bool = false;
pub const VISIBILITY_HIDE_FIRST_MASTER_PAGE: bool = false;
pub const VISIBILITY_HIDE_FIRST_PAGE_NUM: bool = false;
pub const VISIBILITY_HIDE_FIRST_EMPTY_LINE: bool = false;
pub const VISIBILITY_SHOW_LINE_NUMBER: bool = false;

// =====================================================================
// CellSpan (Class/Para/cellSpan.cpp:43)
// ⚠️ colSpan·rowSpan 기본값이 1 — 0 아님
// =====================================================================
pub const CELLSPAN_COL_SPAN: u32 = 1;
pub const CELLSPAN_ROW_SPAN: u32 = 1;

// =====================================================================
// RunType (Class/Para/RunType.cpp:43)
// ⚠️ charPrIDRef 기본값이 (UINT)-1 (u32::MAX) — 0 아님
// =====================================================================
pub const RUN_CHAR_PR_ID_REF_UNSET: u32 = u32::MAX;

// =====================================================================
// TableType (Class/Para/TableType.cpp:32)
// =====================================================================
pub const TABLE_REPEAT_HEADER: bool = false;
pub const TABLE_NO_ADJUST: bool = false;

// =====================================================================
// PictureType (Class/Para/PictureType.cpp:41)
// =====================================================================
pub const PICTURE_REVERSE: bool = false;

// =====================================================================
// Sz (Class/Para/sz.cpp:45)
// =====================================================================
pub const SZ_WIDTH_REL_TO: u32 = 0; // ABSOLUTE
pub const SZ_HEIGHT_REL_TO: u32 = 0;
pub const SZ_PROTECT: bool = false;

// =====================================================================
// NumberingType (Class/Head/NumberingType.cpp:31)
// ⚠️ start 기본값이 1 — 0 아님
// =====================================================================
pub const NUMBERING_START: i32 = 1;

// =====================================================================
// PageBorderFill (Class/Para/pageBorderFill.cpp:46)
// =====================================================================
pub const PAGE_BORDER_HEADER_INSIDE: bool = false;
pub const PAGE_BORDER_FOOTER_INSIDE: bool = false;

// =====================================================================
// BeginNum (Class/Head/beginNum.cpp:32)
// =====================================================================
pub const BEGIN_NUM_PAGE: u32 = 0;
pub const BEGIN_NUM_FOOTNOTE: u32 = 0;
pub const BEGIN_NUM_ENDNOTE: u32 = 0;
pub const BEGIN_NUM_PIC: u32 = 0;
pub const BEGIN_NUM_TBL: u32 = 0;
pub const BEGIN_NUM_EQUATION: u32 = 0;

// =====================================================================
// Font (Class/Head/font.cpp:31)
// =====================================================================
pub const FONT_IS_EMBEDDED: bool = false;
pub const FONT_TYPE: u32 = 0;

// =====================================================================
// Enum: LSTYPE — lineSpacing type (enumdef.h:588)
// =====================================================================
pub const LS_PERCENT: u32 = 0;
pub const LS_FIXED: u32 = 1;
pub const LS_BETWEEN_LINES: u32 = 2;
pub const LS_AT_LEAST: u32 = 3;

// =====================================================================
// Enum: ALIGNHORZ (enumdef.h:484)
// =====================================================================
pub const AH_JUSTIFY: u32 = 0;
pub const AH_LEFT: u32 = 1;
pub const AH_RIGHT: u32 = 2;
pub const AH_CENTER: u32 = 3;
pub const AH_DISTRIBUTE: u32 = 4;
pub const AH_DISTRIBUTE_SPACE: u32 = 5;

// =====================================================================
// Enum: ALIGNVERT (enumdef.h:506)
// =====================================================================
pub const AV_BASELINE: u32 = 0;
pub const AV_TOP: u32 = 1;
pub const AV_CENTER: u32 = 2;
pub const AV_BOTTOM: u32 = 3;

// =====================================================================
// Enum: FONTFACELANGTYPE (enumdef.h:42)
// =====================================================================
pub const FLT_HANGUL: usize = 0;
pub const FLT_LATIN: usize = 1;
pub const FLT_HANJA: usize = 2;
pub const FLT_JAPANESE: usize = 3;
pub const FLT_OTHER: usize = 4;
pub const FLT_SYMBOL: usize = 5;
pub const FLT_USER: usize = 6;

/// HWPX `<hh:fontface lang="...">` 의 lang 속성 문자열 (인덱스 순)
pub const FONTFACE_LANG_NAMES: [&str; 7] = [
    "HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER",
];

// =====================================================================
// Enum: VERTRELTOTYPE (enumdef.h:1310)
// =====================================================================
pub const VRT_PAPER: u32 = 0;
pub const VRT_PAGE: u32 = 1;
pub const VRT_PARA: u32 = 2;

// =====================================================================
// Enum: HORZRELTOTYPE (enumdef.h:1326)
// =====================================================================
pub const HRT_PAPER: u32 = 0;
pub const HRT_PAGE: u32 = 1;
pub const HRT_COLUMN: u32 = 2;
pub const HRT_PARA: u32 = 3;

// =====================================================================
// Enum: ASOTEXTWRAPTYPE — textWrap (enumdef.h:1877)
// =====================================================================
pub const ASOTWT_SQUARE: u32 = 0;
pub const ASOTWT_TOP_AND_BOTTOM: u32 = 1;
pub const ASOTWT_BEHIND_TEXT: u32 = 2;
pub const ASOTWT_IN_FRONT_OF_TEXT: u32 = 3;

// =====================================================================
// Enum: TABLEPAGEBREAKTYPE (enumdef.h:1954)
// =====================================================================
pub const TPBT_NONE: u32 = 0;
pub const TPBT_TABLE: u32 = 1;
pub const TPBT_CELL: u32 = 2;

// =====================================================================
// Enum: SYMBOLMARKTYPE (enumdef.h:83)
// =====================================================================
pub const SMT_NONE: u32 = 0;
pub const SMT_DOT_ABOVE: u32 = 1;
pub const SMT_RING_ABOVE: u32 = 2;
pub const SMT_TILDE: u32 = 3;

// =====================================================================
// Enum: STYLETYPE (enumdef.h:120)
// =====================================================================
pub const ST_PARA: u32 = 0;
pub const ST_CHAR: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_to_grid_is_the_only_true_default() {
        assert!(PARASHAPE_SNAP_TO_GRID);
        assert!(!PARASHAPE_FONT_LINE_HEIGHT);
        assert!(!PARASHAPE_SUPPRESS_LINE_NUMBERS);
        assert!(!PARASHAPE_CHECKED);
        assert!(!CHARSHAPE_USE_FONT_SPACE);
        assert!(!CHARSHAPE_USE_KERNING);
    }

    #[test]
    fn cell_span_defaults_are_one() {
        assert_eq!(CELLSPAN_COL_SPAN, 1);
        assert_eq!(CELLSPAN_ROW_SPAN, 1);
    }

    #[test]
    fn run_char_pr_id_ref_unset_is_u32_max() {
        assert_eq!(RUN_CHAR_PR_ID_REF_UNSET, u32::MAX);
    }

    #[test]
    fn numbering_start_is_one() {
        assert_eq!(NUMBERING_START, 1);
    }

    #[test]
    fn fontface_lang_names_match_indices() {
        assert_eq!(FONTFACE_LANG_NAMES[FLT_HANGUL], "HANGUL");
        assert_eq!(FONTFACE_LANG_NAMES[FLT_LATIN], "LATIN");
        assert_eq!(FONTFACE_LANG_NAMES[FLT_USER], "USER");
    }
}
