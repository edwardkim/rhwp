# 구현계획서: HWPX Serializer 완성 — 표/이미지/스타일/글꼴 직렬화

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **작성일**: 2026-04-17
- **선행 문서**:
  - 수행계획서 `mydocs/plans/task_m100_182.md`
  - OWPML 참조 `mydocs/tech/hwpx_hancom_reference.md`
  - DVC 참조 `mydocs/tech/hwpx_dvc_reference.md`

## 1. 구현 원칙 (수행계획서 기반)

1. Two-pass + `SerializeContext` (ID 풀 + 참조 단언)
2. 빈 문서 특수 분기 제거 (거짓-양성 차단)
3. 누적 라운드트립 하네스 우선 (Stage별 회귀 방지)
4. IR 의미 비교 (`IrDiff`)
5. 3-way BinData 단언
6. #181(SVG)과 독립 진행
7. 한컴 OWPML을 스펙 참조원 활용
8. 한컴 DVC를 Stage 5 보조 게이트로 활용

## 2. Stage 0 — 라운드트립 하네스 + 분기 제거 + 한컴 기본값 상수화

### 2.1 파일별 변경

#### (1) `src/serializer/hwpx/mod.rs`

**삭제**: 61-66행의 빈 문서 특수 분기
```rust
// 삭제 전
if doc.sections.len() == 1 && doc.bin_data_content.is_empty() {
    z.write_deflated("Contents/content.hpf", EMPTY_CONTENT_HPF.as_bytes())?;
} else {
    let content_hpf = content::write_content_hpf(&section_hrefs, &[])?;
    z.write_deflated("Contents/content.hpf", &content_hpf)?;
}

// 삭제 후 (항상 동적 경로)
let content_hpf = content::write_content_hpf(&section_hrefs, ctx.bin_data_entries())?;
z.write_deflated("Contents/content.hpf", &content_hpf)?;
```

**추가**: 1-pass 스캔 호출
```rust
pub fn serialize_hwpx(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let ctx = SerializeContext::collect_from_document(doc);
    // ... 2-pass write (header, section, content.hpf, manifest) ...
    ctx.assert_all_refs_resolved()?;  // 단언
    z.finish()
}
```

#### (2) `src/serializer/hwpx/context.rs` (신규)

```rust
//! 1-pass 스캔으로 ID 풀을 구성하고, 2-pass 쓰기에서 참조 정합성을 단언한다.
use std::collections::{HashMap, HashSet};
use crate::model::document::Document;
use super::SerializeError;

pub struct IdPool<T: Copy + Eq + std::hash::Hash> {
    registered: HashSet<T>,
    referenced: HashSet<T>,
}

impl<T: Copy + Eq + std::hash::Hash> IdPool<T> {
    pub fn new() -> Self { Self { registered: HashSet::new(), referenced: HashSet::new() } }
    pub fn register(&mut self, id: T) { self.registered.insert(id); }
    pub fn reference(&mut self, id: T) { self.referenced.insert(id); }
    pub fn unresolved(&self) -> impl Iterator<Item = &T> {
        self.referenced.difference(&self.registered)
    }
}

pub struct BinDataEntry {
    pub id: String,           // "image1"
    pub href: String,         // "BinData/image1.png"
    pub media_type: String,   // "image/png"
    pub bytes_len: usize,
}

pub struct SerializeContext {
    pub char_shape_ids: IdPool<u32>,
    pub para_shape_ids: IdPool<u16>,
    pub border_fill_ids: IdPool<u16>,
    pub tab_pr_ids: IdPool<u16>,
    pub numbering_ids: IdPool<u16>,
    pub style_ids: IdPool<u16>,
    pub font_ids_per_lang: [Vec<FontRef>; 7],  // HANGUL/LATIN/HANJA/JAPANESE/OTHER/SYMBOL/USER
    pub bin_data_map: HashMap<u16, BinDataEntry>,
}

#[derive(Clone)]
pub struct FontRef {
    pub face: String,
    pub type_: u32,
    pub is_embedded: bool,
}

impl SerializeContext {
    pub fn collect_from_document(doc: &Document) -> Self { /* 1-pass */ }
    pub fn bin_data_entries(&self) -> Vec<BinDataEntry> { /* ... */ }
    pub fn resolve_bin_id(&self, bin_data_id: u16) -> Option<&str> { /* ... */ }
    pub fn assert_all_refs_resolved(&self) -> Result<(), SerializeError> {
        // char_shape/para_shape/border_fill/tab_pr/numbering/style 모두 검증
        // bin_data: 3-way (ref ↔ manifest ↔ zip entry) 는 mod.rs에서 최종 단언
    }
}
```

#### (3) `src/serializer/hwpx/roundtrip.rs` (신규)

```rust
//! HWPX 바이트 → parse → serialize → parse → 원본 IR과 비교 (IrDiff)
use crate::parser::hwpx::parse_hwpx;
use crate::model::document::Document;
use super::serialize_hwpx;

pub struct IrDiff {
    pub differences: Vec<IrDifference>,
}

pub enum IrDifference {
    SectionCount { expected: usize, actual: usize },
    ParagraphText { section: usize, para: usize, expected: String, actual: String },
    CharShapeCount { expected: usize, actual: usize },
    // ... 필드별
}

impl IrDiff {
    pub fn is_empty(&self) -> bool { self.differences.is_empty() }
    pub fn allowed(&self, allow: IrDiffAllow) -> bool { /* 관용 규칙 */ }
}

pub struct IrDiffAllow {
    pub shape_raw: bool,  // 도형 raw 바이트 diff 허용
}

pub fn roundtrip_ir_diff(hwpx_bytes: &[u8]) -> Result<IrDiff, SerializeError> {
    let doc1 = parse_hwpx(hwpx_bytes).map_err(|_| SerializeError::ParseFailed)?;
    let out = serialize_hwpx(&doc1)?;
    let doc2 = parse_hwpx(&out).map_err(|_| SerializeError::ParseFailed)?;
    Ok(diff_documents(&doc1, &doc2))
}

fn diff_documents(a: &Document, b: &Document) -> IrDiff { /* 필드 단위 비교 */ }
```

#### (4) `src/serializer/hwpx/fixtures.rs` (신규)

```rust
//! 테스트 대조용 상수 + 빈 Document 기본값 생성
use crate::model::document::DocInfo;

pub const EMPTY_HEADER_XML: &str = include_str!("templates/empty_header.xml");
pub const EMPTY_SECTION0_XML: &str = include_str!("templates/empty_section0.xml");

pub fn default_doc_info_resources() -> DocInfo {
    // 빈 Document도 한컴 템플릿과 등가 출력되도록
    // charPr/paraPr/borderFill/style 각 최소 1개를 canonical_defaults 기반으로 미리 채움
}
```

#### (5) `src/serializer/hwpx/canonical_defaults.rs` (신규)

```rust
//! 한컴 OWPML 공식 기본값 상수 테이블
//! 
//! Default values referenced from hancom-io/hwpx-owpml-model (Apache License 2.0, © Hancom Inc.)
//! See mydocs/tech/hwpx_hancom_reference.md for details.

// ===== CharShapeType (CharShapeType.cpp:31) =====
pub const CHARSHAPE_HEIGHT: u32 = 1000;
pub const CHARSHAPE_TEXT_COLOR: u32 = 0x000000;
pub const CHARSHAPE_SHADE_COLOR: u32 = 0xFFFFFF;
pub const CHARSHAPE_USE_FONT_SPACE: bool = false;
pub const CHARSHAPE_USE_KERNING: bool = false;
pub const CHARSHAPE_SYM_MARK: u32 = 0;  // SMT_NONE

// ===== ParaShapeType (ParaShapeType.cpp:31) — ★ snapToGrid=true (예외) =====
pub const PARASHAPE_SNAP_TO_GRID: bool = true;
pub const PARASHAPE_FONT_LINE_HEIGHT: bool = false;
pub const PARASHAPE_SUPPRESS_LINE_NUMBERS: bool = false;
pub const PARASHAPE_CHECKED: bool = false;
pub const PARASHAPE_CONDENSE: u32 = 0;
pub const PARASHAPE_TAB_PR_ID_REF: u16 = 0;

// ===== BorderFillType (BorderFillType.cpp:31) =====
pub const BORDERFILL_THREE_D: bool = false;
pub const BORDERFILL_SHADOW: bool = false;
pub const BORDERFILL_BREAK_CELL_SEPARATE_LINE: bool = false;
pub const BORDERFILL_CENTER_LINE: u32 = 0;

// ===== BreakSetting (breakSetting.cpp:32) =====
pub const BREAKSETTING_WIDOW_ORPHAN: bool = false;
pub const BREAKSETTING_KEEP_WITH_NEXT: bool = false;
pub const BREAKSETTING_KEEP_LINES: bool = false;
pub const BREAKSETTING_PAGE_BREAK_BEFORE: bool = false;
pub const BREAKSETTING_BREAK_NON_LATIN_WORD: u32 = 0;
pub const BREAKSETTING_LINE_WRAP: u32 = 0;

// ===== Visibility (visibility.cpp:49) =====
pub const VISIBILITY_HIDE_FIRST_HEADER: bool = false;
pub const VISIBILITY_HIDE_FIRST_FOOTER: bool = false;
pub const VISIBILITY_HIDE_FIRST_MASTER_PAGE: bool = false;
pub const VISIBILITY_HIDE_FIRST_PAGE_NUM: bool = false;
pub const VISIBILITY_HIDE_FIRST_EMPTY_LINE: bool = false;
pub const VISIBILITY_SHOW_LINE_NUMBER: bool = false;

// ===== CellSpan (cellSpan.cpp:43) — ★ colSpan=1, rowSpan=1 =====
pub const CELLSPAN_COL_SPAN: u32 = 1;
pub const CELLSPAN_ROW_SPAN: u32 = 1;

// ===== RunType (RunType.cpp:43) — ★ charPrIDRef=-1 (특수값) =====
pub const RUN_CHAR_PR_ID_REF_UNSET: u32 = u32::MAX;  // (UINT)-1

// ===== TableType (TableType.cpp:32) =====
pub const TABLE_REPEAT_HEADER: bool = false;
pub const TABLE_NO_ADJUST: bool = false;

// ===== PictureType (PictureType.cpp:41) =====
pub const PICTURE_REVERSE: bool = false;

// ===== Sz (sz.cpp:45) =====
pub const SZ_WIDTH_REL_TO: u32 = 0;   // ABSOLUTE
pub const SZ_HEIGHT_REL_TO: u32 = 0;
pub const SZ_PROTECT: bool = false;

// ===== NumberingType (NumberingType.cpp:31) — ★ start=1 =====
pub const NUMBERING_START: i32 = 1;

// ===== PageBorderFill (pageBorderFill.cpp:46) =====
pub const PAGE_BORDER_HEADER_INSIDE: bool = false;
pub const PAGE_BORDER_FOOTER_INSIDE: bool = false;

// ===== 주요 Enum =====

/// LSTYPE (lineSpacing type) — enumdef.h:588
pub const LS_PERCENT: u32 = 0;
pub const LS_FIXED: u32 = 1;
pub const LS_BETWEEN_LINES: u32 = 2;
pub const LS_AT_LEAST: u32 = 3;

/// ALIGNHORZ — enumdef.h:484
pub const AH_JUSTIFY: u32 = 0;
pub const AH_LEFT: u32 = 1;
pub const AH_RIGHT: u32 = 2;
pub const AH_CENTER: u32 = 3;
pub const AH_DISTRIBUTE: u32 = 4;
pub const AH_DISTRIBUTE_SPACE: u32 = 5;

/// ALIGNVERT — enumdef.h:506
pub const AV_BASELINE: u32 = 0;
pub const AV_TOP: u32 = 1;
pub const AV_CENTER: u32 = 2;
pub const AV_BOTTOM: u32 = 3;

/// FONTFACELANGTYPE — enumdef.h:42
pub const FLT_HANGUL: usize = 0;
pub const FLT_LATIN: usize = 1;
pub const FLT_HANJA: usize = 2;
pub const FLT_JAPANESE: usize = 3;
pub const FLT_OTHER: usize = 4;
pub const FLT_SYMBOL: usize = 5;
pub const FLT_USER: usize = 6;

/// VERTRELTOTYPE — enumdef.h:1310
pub const VRT_PAPER: u32 = 0;
pub const VRT_PAGE: u32 = 1;
pub const VRT_PARA: u32 = 2;

/// HORZRELTOTYPE — enumdef.h:1326
pub const HRT_PAPER: u32 = 0;
pub const HRT_PAGE: u32 = 1;
pub const HRT_COLUMN: u32 = 2;
pub const HRT_PARA: u32 = 3;

/// ASOTEXTWRAPTYPE — enumdef.h:1877
pub const ASOTWT_SQUARE: u32 = 0;
pub const ASOTWT_TOP_AND_BOTTOM: u32 = 1;
pub const ASOTWT_BEHIND_TEXT: u32 = 2;
pub const ASOTWT_IN_FRONT_OF_TEXT: u32 = 3;

/// TABLEPAGEBREAKTYPE — enumdef.h:1954
pub const TPBT_NONE: u32 = 0;
pub const TPBT_TABLE: u32 = 1;
pub const TPBT_CELL: u32 = 2;
```

#### (6) `tests/hwpx_roundtrip_integration.rs` (신규)

```rust
//! 샘플별 HWPX 라운드트립 IR diff 테스트. Stage별 누적.

use rhwp::serializer::hwpx::roundtrip::{roundtrip_ir_diff, IrDiffAllow};

#[test]
fn stage0_blank_hwpx_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/blank_hwpx.hwpx");
    let diff = roundtrip_ir_diff(bytes).expect("roundtrip");
    assert!(diff.is_empty(), "IR diff on blank: {:?}", diff);
}

// Stage 1~5에서 누적 추가:
// #[test] fn stage1_ref_empty_roundtrip() { ... }
// #[test] fn stage2_ref_text_roundtrip() { ... }
// #[test] fn stage3_ref_table_roundtrip() { ... }
// #[test] fn stage4_pic_in_head_roundtrip() { ... }
// #[test] fn stage5_real_doc_roundtrip() { ... }
```

#### (7) `THIRD_PARTY_LICENSES.md` (이미 반영됨 ✅)

참조 오픈소스 섹션에 hancom-io/hwpx-owpml-model, hancom-io/dvc 명시 완료 (2026-04-17). 코드 직접 복사 없이 스펙·설계만 참조하는 범위임을 명확히 기술.

### 2.2 완료 기준

- [ ] 기존 단위 테스트 10개 green
- [ ] `stage0_blank_hwpx_roundtrip` IrDiff 0 통과
- [ ] `ctx.assert_all_refs_resolved()` 전 경로 통과
- [ ] `canonical_defaults.rs`에 주요 20+ 상수 등록
- [ ] 분기 `mod.rs:61-66` 제거 확인 (`rg "bin_data_content.is_empty" src/serializer/`)
- [x] `THIRD_PARTY_LICENSES.md`에 한컴 참조 오픈소스 명시 (2026-04-17 완료)

### 2.3 단계별 보고서

`mydocs/working/task_m100_182_stage0.md` 작성 후 승인 요청

---

## 3. Stage 1 — header.xml IR 기반 동적 생성

### 3.1 파일별 변경

#### (1) `src/serializer/hwpx/header.rs` (13줄 → ~400줄)

```rust
use super::context::SerializeContext;
use super::utils::*;
use super::canonical_defaults::*;
use crate::model::document::Document;

pub fn write_header(doc: &Document, ctx: &SerializeContext) -> Result<Vec<u8>, SerializeError> {
    let mut w = quick_xml::Writer::new(Vec::new());
    write_xml_decl(&mut w)?;
    
    // <hh:head secCnt="N" version="...">
    let sec_cnt = doc.sections.len().to_string();
    start_tag_attrs(&mut w, "hh:head", &[("secCnt", &sec_cnt), ("version", "1.31")])?;
    
    write_begin_num(&mut w, &doc.doc_properties)?;
    write_ref_list(&mut w, doc, ctx)?;
    write_fontfaces(&mut w, doc)?;
    write_border_fills(&mut w, doc)?;
    write_char_properties(&mut w, doc)?;
    write_tab_properties(&mut w, doc)?;
    write_numberings(&mut w, doc)?;
    write_para_properties(&mut w, doc)?;
    write_styles(&mut w, doc)?;
    write_compat_doc(&mut w, doc)?;
    write_doc_option(&mut w, doc)?;
    
    end_tag(&mut w, "hh:head")?;
    Ok(w.into_inner())
}

// --- 각 writer 함수 ---

fn write_char_properties(w: &mut Writer, doc: &Document) -> Result<()> {
    // <hh:charProperties itemCnt="N">
    let cnt = doc.doc_info.char_shapes.len();
    start_tag_attrs(w, "hh:charProperties", &[("itemCnt", &cnt.to_string())])?;
    
    for (id, cs) in doc.doc_info.char_shapes.iter().enumerate() {
        // canonical 속성 순서 (CharShapeType.cpp:79-86):
        // id, height, textColor, shadeColor, useFontSpace, useKerning, symMark, borderFillIDRef
        let attrs = vec![
            ("id", id.to_string()),
            ("height", cs.base_size.to_string()),
            ("textColor", format!("#{:06X}", cs.text_color)),
            ("shadeColor", format!("#{:06X}", cs.shade_color)),
            ("useFontSpace", bool_str(cs.use_font_space)),
            ("useKerning", bool_str(cs.use_kerning)),
            ("symMark", cs.sym_mark.to_string()),
            ("borderFillIDRef", cs.border_fill_id.to_string()),
        ];
        start_tag_attrs(w, "hh:charPr", &attrs_ref(&attrs))?;
        
        // canonical 자식 순서 (CharShapeType.cpp:59-73):
        // fontRef, ratio, spacing, relSz, offset, italic, bold, underline, strikeout, outline, shadow, emboss, engrave, supscript, subscript
        write_font_ref(w, &cs.font_ids)?;
        write_ratio(w, &cs.ratios)?;
        write_spacing(w, &cs.spacings)?;
        write_rel_sz(w, &cs.relative_sizes)?;
        write_offset(w, &cs.char_offsets)?;
        if cs.italic { empty_tag(w, "hh:italic")?; }
        if cs.bold { empty_tag(w, "hh:bold")?; }
        // ... underline/strikeout/outline/shadow/emboss/engrave/supscript/subscript
        
        end_tag(w, "hh:charPr")?;
    }
    end_tag(w, "hh:charProperties")?;
    Ok(())
}

fn write_para_properties(w: &mut Writer, doc: &Document) -> Result<()> {
    // <hh:paraProperties itemCnt="N">
    // 각 <hh:paraPr> 속성 순서 (ParaShapeType.cpp:62-68):
    // id, tabPrIDRef, condense, fontLineHeight, snapToGrid, suppressLineNumbers, checked
    // 자식 순서 (ParaShapeType.cpp:50-56):
    // align, heading, breakSetting, margin, lineSpacing, border, autoSpacing
    // ...
}

fn write_border_fills(w: &mut Writer, doc: &Document) -> Result<()> {
    // <hh:borderFills itemCnt="N">
    // 각 <hh:borderFill> 속성 순서 (BorderFillType.cpp:64-68):
    // id, threeD, shadow, centerLine, breakCellSeparateLine
    // 자식 순서 (BorderFillType.cpp:51-58):
    // slash, backSlash, leftBorder, rightBorder, topBorder, bottomBorder, diagonal, fillBrush
    // ...
}

fn write_fontfaces(w: &mut Writer, doc: &Document) -> Result<()> {
    // <hh:fontfaces itemCnt="7">
    //   <hh:fontface lang="HANGUL" fontCnt="N"> ... </hh:fontface>
    //   ... (7개 언어)
    // FONTFACELANGTYPE 순서: HANGUL, LATIN, HANJA, JAPANESE, OTHER, SYMBOL, USER
}

fn write_styles(w: &mut Writer, doc: &Document) -> Result<()> {
    // <hh:styles itemCnt="N">
    // 속성 순서 (StyleType.cpp):
    // id, type, name, engName, paraPrIDRef, charPrIDRef, nextStyleIDRef, langID, lockForm
}

fn write_numberings(w: &mut Writer, doc: &Document) -> Result<()> { /* ... */ }
fn write_tab_properties(w: &mut Writer, doc: &Document) -> Result<()> { /* ... */ }
fn write_begin_num(w: &mut Writer, props: &DocProperties) -> Result<()> { /* ... */ }
fn write_ref_list(w: &mut Writer, doc: &Document, ctx: &SerializeContext) -> Result<()> { /* ... */ }
fn write_compat_doc(w: &mut Writer, doc: &Document) -> Result<()> { /* ... */ }
fn write_doc_option(w: &mut Writer, doc: &Document) -> Result<()> { /* ... */ }
```

#### (2) `src/serializer/hwpx/fixtures.rs` 확장

`default_doc_info_resources()`에서 빈 Document도 한컴 템플릿과 등가 출력하도록 canonical_defaults 기반 최소 리소스 채움.

### 3.2 완료 기준

- [ ] Stage 0 하네스 유지
- [ ] `blank_hwpx.hwpx` + `ref_empty.hwpx` 라운드트립 IrDiff 0
- [ ] 단위 테스트: `char_shapes.len() == N` → `<hh:charPr>` 정확히 N개 출력
- [ ] 단위 테스트: canonical 속성 순서 일치 (CharShapeType.cpp 기준)
- [ ] `charPrIDRef/paraPrIDRef/borderFillIDRef/tabPrIDRef/fontRef/styleID` 전 참조 resolve
- [ ] `mydocs/working/task_m100_182_stage1.md` 작성

### 3.3 테스트 케이스

```rust
#[test]
fn header_writer_preserves_char_shape_count() {
    let mut doc = Document::default();
    // char_shapes 5개 추가
    for _ in 0..5 { doc.doc_info.char_shapes.push(CharShape::default()); }
    let ctx = SerializeContext::collect_from_document(&doc);
    let bytes = write_header(&doc, &ctx).unwrap();
    let xml = std::str::from_utf8(&bytes).unwrap();
    assert_eq!(xml.matches("<hh:charPr ").count(), 5);
    assert!(xml.contains(r#"<hh:charProperties itemCnt="5">"#));
}

#[test]
fn header_writer_canonical_attr_order_charpr() {
    // CharShapeType.cpp:79-86 기준 속성 순서 검증
    // id → height → textColor → shadeColor → useFontSpace → useKerning → symMark → borderFillIDRef
}

#[test]
fn stage1_ref_empty_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/ref/ref_empty.hwpx");
    let diff = roundtrip_ir_diff(bytes).unwrap();
    assert!(diff.is_empty());
}
```

---

## 4. Stage 2 — section.xml 동적화 + charPrIDRef 매핑

### 4.1 파일별 변경

#### (1) `src/serializer/hwpx/section.rs`

**삭제**: `empty_section0.xml` 템플릿 치환 코드 전체
**교체**: `SectionWriter`가 `<hs:sec>` 루트부터 빌드

```rust
pub fn write_section(sec: &Section, doc: &Document, idx: usize, ctx: &SerializeContext)
    -> Result<Vec<u8>, SerializeError>
{
    let mut w = Writer::new(Vec::new());
    write_xml_decl(&mut w)?;
    start_tag(&mut w, "hs:sec")?;
    
    let mut vert_cursor = 0;
    for (pi, para) in sec.paragraphs.iter().enumerate() {
        paragraph::write_paragraph(&mut w, para, doc, ctx, pi, &mut vert_cursor)?;
    }
    
    end_tag(&mut w, "hs:sec")?;
    Ok(w.into_inner())
}
```

#### (2) `src/serializer/hwpx/paragraph.rs` (신규)

```rust
use crate::model::paragraph::Paragraph;
use super::context::SerializeContext;

pub enum ParagraphChild<'a> {
    Text { text: &'a str, char_pr_id: u32 },
    Tab { width: u32, leader: u8, type_: u8 },
    LineBreak,
    // Stage 3: Table(&'a Table)
    // Stage 4: Picture(&'a Picture)
    // Stage 5: Shape, Field
}

pub fn write_paragraph(w: &mut Writer, para: &Paragraph, doc: &Document,
                      ctx: &SerializeContext, pi: usize, vert_cursor: &mut i64)
    -> Result<(), SerializeError>
{
    // <hp:p> 속성: id, paraPrIDRef, styleIDRef, pageBreak, columnBreak, merged
    let attrs = vec![
        ("id", pi.to_string()),
        ("paraPrIDRef", para.para_shape_id.to_string()),
        ("styleIDRef", para.style_id.to_string()),
        ("pageBreak", page_break_attr(para)),
        ("columnBreak", column_break_attr(para)),
        ("merged", "0".to_string()),
    ];
    start_tag_attrs(w, "hp:p", &attrs_ref(&attrs))?;
    
    // <hp:run> 범위별 분할 (char_shape_refs 기반)
    let runs = split_runs_by_char_shape(para);
    for run in runs {
        start_tag_attrs(w, "hp:run", &[("charPrIDRef", &run.char_pr_id.to_string())])?;
        write_run_content(w, para, &run, ctx)?;
        end_tag(w, "hp:run")?;
    }
    
    // <hp:linesegarray> / <hp:lineseg> 생성 (벌크)
    write_lineseg_array(w, para, vert_cursor)?;
    
    end_tag(w, "hp:p")?;
    Ok(())
}

fn split_runs_by_char_shape(para: &Paragraph) -> Vec<RunSpan> {
    // char_shape_refs: UTF-16 offset → char_shape_id 범위별 분할
}

fn write_run_content(w: &mut Writer, para: &Paragraph, run: &RunSpan,
                     ctx: &SerializeContext) -> Result<(), SerializeError>
{
    // <hp:t> 안에 text/tab/lineBreak 혼합 콘텐츠 직렬화
    // 기존 section.rs:60-119 render_paragraph_parts 로직 이식
}
```

### 4.2 완료 기준

- [ ] Stage 0, 1 하네스 유지
- [ ] 기존 문단/탭/줄바꿈 테스트 6개 통과
- [ ] `ref_text.hwpx` 라운드트립 IrDiff 0
- [ ] 다중 run 샘플: run 개수·charPrIDRef 값 원본 일치
- [ ] `mydocs/working/task_m100_182_stage2.md` 작성

---

## 5. Stage 3 — Table 직렬화 (`<hp:tbl>`)

### 5.1 파일별 변경

#### (1) `src/serializer/hwpx/table.rs` (신규)

```rust
use crate::model::control::table::Table;
use super::canonical_defaults::*;

pub fn write_table(w: &mut Writer, table: &Table, doc: &Document,
                   ctx: &SerializeContext) -> Result<(), SerializeError>
{
    // <hp:tbl> 속성 순서 (TableType.cpp:41-48 + 95-101):
    // 부모(AbstractShapeObjectType):
    //   id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle
    // 자신:
    //   pageBreak, repeatHeader, rowCnt, colCnt, cellSpacing, borderFillIDRef, noAdjust
    
    // ⚠️ table.attr 비트 연산 사용 금지! table.common 기반으로만
    let attrs = vec![
        ("id", table.common.instance_id.to_string()),
        ("zOrder", table.common.z_order.to_string()),
        ("numberingType", "NONE".to_string()),
        ("textWrap", text_wrap_str(table.common.text_wrap)),
        ("textFlow", text_flow_str(table.common.text_flow)),
        ("lock", bool_str(table.common.lock)),
        ("dropcapstyle", "None".to_string()),
        ("pageBreak", page_break_str(table.page_break)),
        ("repeatHeader", bool_str(table.repeat_header)),
        ("rowCnt", table.row_count.to_string()),
        ("colCnt", table.col_count.to_string()),
        ("cellSpacing", table.cell_spacing.to_string()),
        ("borderFillIDRef", table.border_fill_id.to_string()),
        ("noAdjust", bool_str(TABLE_NO_ADJUST)),
    ];
    start_tag_attrs(w, "hp:tbl", &attrs_ref(&attrs))?;
    
    // 자식 순서 (TableType.cpp:75-87):
    // 부모: sz, pos, outMargin, caption, shapeComment, parameterset, metaTag
    // 자신: inMargin, cellzoneList, tr, label
    write_sz(w, &table.common)?;
    write_pos(w, &table.common)?;
    write_out_margin(w, &table.outer_margin)?;
    // caption은 옵셔널
    write_in_margin(w, &table.inner_margin)?;
    // cellzoneList는 옵셔널
    
    // <hp:tr> 루프
    for row in &table.rows {
        start_tag(w, "hp:tr")?;
        for cell in &row.cells {
            write_cell(w, cell, doc, ctx)?;
        }
        end_tag(w, "hp:tr")?;
    }
    
    end_tag(w, "hp:tbl")?;
    
    // ctx에 border_fill_id 참조 등록
    Ok(())
}

fn write_cell(w: &mut Writer, cell: &Cell, doc: &Document,
              ctx: &SerializeContext) -> Result<(), SerializeError>
{
    // <hp:tc> 속성: name, header, hasMargin, protect, editable, dirty, borderFillIDRef
    // 자식 순서: cellAddr, cellSpan, cellSz, cellMargin, subList
    // subList 내부는 write_paragraph 재귀
}
```

#### (2) `src/serializer/hwpx/paragraph.rs` 확장

```rust
pub enum ParagraphChild<'a> {
    // ... 기존
    Table(&'a Table),   // Stage 3에서 추가
}

// dispatcher에서 Control::Table(t) → table::write_table 호출
```

### 5.2 완료 기준

- [ ] Stage 0, 1, 2 하네스 유지
- [ ] `hwp_table_test.hwp` HWPX 경로 라운드트립 IrDiff 0 (cell, span, border_fill_id, rowCnt, colCnt)
- [ ] `ref_table.hwpx` 라운드트립 IrDiff 0
- [ ] 중첩 표 inner paragraph 보존 검증
- [ ] `borderFillIDRef` 미등록 시 `assert_all_refs_resolved` 실패로 잡힘
- [ ] `mydocs/working/task_m100_182_stage3.md` 작성

---

## 6. Stage 4 — Picture + BinData ZIP 엔트리

### 6.1 파일별 변경

#### (1) `src/serializer/hwpx/picture.rs` (신규)

```rust
pub fn write_picture(w: &mut Writer, pic: &Picture, doc: &Document,
                     ctx: &SerializeContext) -> Result<(), SerializeError>
{
    // <hp:pic> 속성 (PictureType.cpp + 부모 AbstractShapeComponentType + AbstractShapeObjectType):
    // id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle, reverse
    
    // 자식 순서 (PictureType.cpp:79-102):
    // sz, pos, outMargin, caption, shapeComment, parameterset, metaTag, 
    // offset, orgSz, curSz, flip, rotationInfo, renderingInfo, lineShape,
    // imgRect, imgClip, effects, inMargin, imgDim, img
    
    // <hc:img binaryItemIDRef="..."> — 3-way 단언 필수
    let bin_manifest_id = ctx.resolve_bin_id(pic.image_attr.bin_data_id)
        .ok_or(SerializeError::UnresolvedBinData(pic.image_attr.bin_data_id))?;
    
    empty_tag_attrs(w, "hc:img", &[
        ("binaryItemIDRef", bin_manifest_id),
        ("bright", &pic.image_attr.bright.to_string()),
        ("contrast", &pic.image_attr.contrast.to_string()),
        ("effect", &pic.image_attr.effect.to_string()),
        ("alpha", &pic.image_attr.alpha.to_string()),
    ])?;
    
    Ok(())
}
```

#### (2) `src/serializer/hwpx/manifest.rs` (신규)

```rust
//! META-INF/manifest.xml 동적 생성 (BinData 엔트리 반영)

pub fn write_manifest_xml(ctx: &SerializeContext) -> Result<Vec<u8>, SerializeError> {
    // BinData 엔트리를 media-type과 함께 manifest에 추가
}
```

#### (3) `src/serializer/hwpx/mod.rs` 수정

```rust
pub fn serialize_hwpx(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let ctx = SerializeContext::collect_from_document(doc);
    let mut z = HwpxZipWriter::new();
    
    z.write_stored("mimetype", b"application/hwp+zip")?;
    z.write_deflated("version.xml", VERSION_XML.as_bytes())?;
    z.write_deflated("Contents/header.xml", &header::write_header(doc, &ctx)?)?;
    
    let section_hrefs: Vec<String> = (0..doc.sections.len())
        .map(|i| format!("Contents/section{}.xml", i)).collect();
    for (i, sec) in doc.sections.iter().enumerate() {
        let xml = section::write_section(sec, doc, i, &ctx)?;
        z.write_deflated(&section_hrefs[i], &xml)?;
    }
    
    // ✅ BinData 엔트리 (Stage 4 추가)
    for entry in ctx.bin_data_entries() {
        let bytes = doc.bin_data_content.iter()
            .find(|b| b.id == entry.bin_data_id)
            .ok_or(SerializeError::MissingBinData)?;
        z.write_deflated(&entry.href, &bytes.data)?;
    }
    
    z.write_deflated("Preview/PrvText.txt", PRV_TEXT)?;
    z.write_deflated("Preview/PrvImage.png", PRV_IMAGE_PNG)?;
    z.write_deflated("settings.xml", SETTINGS_XML.as_bytes())?;
    z.write_deflated("META-INF/container.rdf", META_INF_CONTAINER_RDF.as_bytes())?;
    z.write_deflated("Contents/content.hpf", 
                     &content::write_content_hpf(&section_hrefs, &ctx.bin_data_entries())?)?;
    z.write_deflated("META-INF/container.xml", META_INF_CONTAINER_XML.as_bytes())?;
    z.write_deflated("META-INF/manifest.xml", &manifest::write_manifest_xml(&ctx)?)?;
    
    // ✅ 3-way 단언
    ctx.assert_all_refs_resolved()?;
    assert_bin_data_3way(&ctx, &z)?;
    
    z.finish()
}

fn assert_bin_data_3way(ctx: &SerializeContext, z: &HwpxZipWriter) 
    -> Result<(), SerializeError> 
{
    // 1. <hp:pic> binaryItemIDRef 집합 (ctx에서)
    // 2. content.hpf opf:item id 집합
    // 3. ZIP entry BinData/* 경로 집합
    // 세 집합의 합집합 == 교집합을 단언
}
```

### 6.2 완료 기준

- [ ] Stage 0~3 하네스 유지
- [ ] `pic-in-head-01.hwp`/`pic-crop-01.hwp` 라운드트립: `bin_data_content.len()`, `bin_data_id`, PNG Blake3 해시 일치
- [ ] ZIP 내 BinData/* 개수 == `doc.bin_data_content.len()`
- [ ] 3-way 단언 (binaryItemIDRef ↔ content.hpf item ↔ ZIP entry) 합집합=교집합
- [ ] `mydocs/working/task_m100_182_stage4.md` 작성

---

## 7. Stage 5 — 도형·필드 + 대형 실문서 스모크 + DVC 보조 검증

### 7.1 파일별 변경

#### (1) `src/serializer/hwpx/shape.rs` (신규)

- `<hp:rect>`, `<hp:line>`, `<hp:container>`, `<hp:textart>` 등
- `Control::Rectangle/Line/...` dispatcher 연결

#### (2) `src/serializer/hwpx/field.rs` (신규)

- `<hp:fieldBegin>/<hp:fieldEnd>`
- 각주(footnote), 미주(endnote) 최소 세트

#### (3) 대형 샘플 하네스 추가

```rust
#[test]
fn stage5_2025_1q_roundtrip() {
    let bytes = include_bytes!("../samples/hwpx/2025년 1분기 해외직접투자 보도자료f.hwpx");
    let diff = roundtrip_ir_diff(bytes).unwrap();
    assert!(diff.allowed(IrDiffAllow { shape_raw: true, ..Default::default() }));
}
```

### 7.2 DVC 보조 검증 (Windows VM 수동, 보조 게이트)

```bash
# Windows VM에서
DVCModel.exe -j --file=result.json -s \
    D:\rhwp_stage5_output.hwpx test.json
# 통과하면 Stage 5 보조 게이트 OK
```

### 7.3 완료 기준

- [ ] Stage 0~4 하네스 유지
- [ ] 대형 샘플 3건 (2024/2025 보도자료) `IrDiff::allowed(shape_raw=true)` 통과
- [ ] 한컴2020 수동 오픈 성공
- [ ] DVC 보조 검증 기본 규칙 통과 (선택적)
- [ ] `mydocs/working/task_m100_182_stage5.md` 작성
- [ ] 최종 보고서 `mydocs/report/task_m100_182_report.md` 작성

---

## 8. 검증 전략 종합

| 층 | 도구 | 범위 |
|---|---|---|
| 단위 | `cargo test` | 각 Writer 구조체 단독 속성·자식 순서 검증 |
| 통합 | `cargo test --test hwpx_roundtrip_integration` | Stage별 누적 샘플 IrDiff assertion |
| 참조 단언 | `ctx.assert_all_refs_resolved()` | charPrIDRef·paraPrIDRef·borderFillIDRef 전 참조 resolve |
| 3-way | `assert_bin_data_3way()` | `<hp:pic>` binaryItemIDRef ↔ manifest ↔ ZIP entry 동일 집합 |
| 수동 | 한컴2020 오픈 | Stage 3/4/5 주요 샘플 수동 렌더링 확인 |
| 보조 | DVC (Windows VM) | Stage 5 서식 규칙 준수 확인 |

## 9. Canonical XML 속성 순서 요약 (한컴 OWPML 기준)

### charPr
- 속성: id, height, textColor, shadeColor, useFontSpace, useKerning, symMark, borderFillIDRef
- 자식: fontRef, ratio, spacing, relSz, offset, italic, bold, underline, strikeout, outline, shadow, emboss, engrave, supscript, subscript

### paraPr
- 속성: id, tabPrIDRef, condense, fontLineHeight, snapToGrid, suppressLineNumbers, checked
- 자식: align, heading, breakSetting, margin, lineSpacing, border, autoSpacing

### borderFill
- 속성: id, threeD, shadow, centerLine, breakCellSeparateLine
- 자식: slash, backSlash, leftBorder, rightBorder, topBorder, bottomBorder, diagonal, fillBrush

### tbl
- 속성: id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle, pageBreak, repeatHeader, rowCnt, colCnt, cellSpacing, borderFillIDRef, noAdjust
- 자식: sz, pos, outMargin, caption, shapeComment, parameterset, metaTag, inMargin, cellzoneList, tr, label

### pic
- 속성: id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle, reverse
- 자식: sz, pos, outMargin, caption, shapeComment, parameterset, metaTag, offset, orgSz, curSz, flip, rotationInfo, renderingInfo, lineShape, imgRect, imgClip, effects, inMargin, imgDim, img

출처: 한컴 OWPML `Class/Head/`, `Class/Para/` 각 클래스 `.cpp` 파일의 `WriteElement()`, `InitMap()` 함수.

## 10. 위험 요소 재확인

| 위험 | 감지 | 완화 |
|---|---|---|
| charPrIDRef 매핑 누락 | `ctx.assert_all_refs_resolved()` | 1-pass 스캔에서 자동 등록 |
| 속성 순서 불일치 | 단위 테스트 + IrDiff | canonical_defaults + 각 writer의 고정 순서 |
| 빈 문서 거짓-양성 | Stage 0 하네스 | 분기 제거 + default_doc_info_resources |
| 3-way BinData 누락 | `assert_bin_data_3way` | 집합 동일성 단언 |
| table.attr 비트 연산 오용 | 코드 리뷰 | HWPX 경로에서는 table.common만 사용 (테스트로 강제) |
| snapToGrid false 출력 (기본값과 불일치) | Stage 1 단위 테스트 | IR 기본값을 canonical_defaults와 일치시킴 |

## 11. 승인 요청

본 구현계획서 승인 후:
1. Stage 0 착수 (기반 공사)
2. Stage 0 완료 → 단계별 보고서 → 승인 → Stage 1
3. Stage 1 완료 → 보고서 → 승인 → Stage 2
4. ... (각 Stage마다 승인 게이트)
5. Stage 5 완료 → 최종 보고서 → 승인 → devel merge
