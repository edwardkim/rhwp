//! 직렬화 컨텍스트 — 1-pass 스캔으로 ID 풀을 구성하고 2-pass 쓰기에서 참조 정합성을 단언.
//!
//! ## 배경
//!
//! HWPX 직렬화에서 가장 큰 함정은 **한 파일(section.xml)에서 쓴 ID가 다른 파일(header.xml)에
//! 등록되지 않은** 상태로 출력되는 경우다. 예: `<hp:run charPrIDRef="3">` 를 썼는데
//! header의 `<hh:charPr id="3">` 가 누락되면 한컴2020이 조용히 스타일을 엉키게 렌더링한다.
//!
//! `SerializeContext`는 이를 구조적으로 방지한다:
//! 1. **1-pass**: Document IR을 훑어 모든 ID를 `registered`에 등록
//! 2. **2-pass**: 각 writer가 ID를 사용할 때 `reference`에 기록
//! 3. **단언**: `assert_all_refs_resolved()` 가 `referenced - registered` 가 공집합임을 확인
//!
//! Stage 0 에서는 뼈대 구조만 둔다. 실제 스캔 로직은 Stage 1~4에서 writer가 추가될 때 함께 확장한다.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::model::control::Control;
use crate::model::document::Document;
use crate::serializer::content_loss::{ContentLossReport, SerializedFormat};
use crate::serializer::SerializeError;

/// 양방향 ID 풀 — 등록된 ID와 참조된 ID를 추적한다.
#[derive(Debug, Default)]
pub struct IdPool<T: Copy + Eq + std::hash::Hash> {
    registered: HashSet<T>,
    referenced: HashSet<T>,
}

impl<T: Copy + Eq + std::hash::Hash> IdPool<T> {
    pub fn new() -> Self {
        Self {
            registered: HashSet::new(),
            referenced: HashSet::new(),
        }
    }

    /// header/DocInfo에서 정의되는 ID를 등록.
    pub fn register(&mut self, id: T) {
        self.registered.insert(id);
    }

    /// section/기타 writer가 ID를 참조할 때 호출.
    pub fn reference(&mut self, id: T) {
        self.referenced.insert(id);
    }

    pub fn is_registered(&self, id: &T) -> bool {
        self.registered.contains(id)
    }

    /// `referenced - registered`: 참조됐으나 등록되지 않은 ID.
    pub fn unresolved(&self) -> Vec<T> {
        self.referenced
            .difference(&self.registered)
            .copied()
            .collect()
    }

    pub fn registered_count(&self) -> usize {
        self.registered.len()
    }
}

/// HWPX manifest + ZIP entry용 BinData 엔트리.
#[derive(Debug, Clone)]
pub struct BinDataEntry {
    /// content.hpf 의 `opf:item id` (예: "image1")
    pub manifest_id: String,
    /// ZIP 엔트리 경로 (예: "BinData/image1.png") 또는 외부 참조 원본 경로
    /// (`is_embedded=false`, 예: `D:\다운로드\...`)
    pub href: String,
    /// MIME 타입 (예: "image/png")
    pub media_type: String,
    /// IR 상의 bin_data_id (storage_id) — 매핑 역추적용
    pub bin_data_id: u16,
    /// content.hpf `isEmbeded` — false 면 ZIP 엔트리가 없는 항목이다:
    /// 외부 파일 참조(#1891) 또는 스트림 부재로 콘텐츠가 없는 항목(#3526).
    pub is_embedded: bool,
}

/// [#3546] OOXML 차트 파트 — BinData 매니페스트 대상이 아니라
/// `Chart/chartN.xml` 원형 경로로 방출되는 항목.
#[derive(Debug, Clone)]
pub struct ChartPartEntry {
    /// ZIP 엔트리 경로 (예: "Chart/chart1.xml")
    pub href: String,
    /// IR 상의 bin_data_id (= 60000 + N, HWPX 파서 주입 규약)
    pub bin_data_id: u16,
}

/// 1-pass 스캔으로 구축되는 직렬화 컨텍스트.
#[derive(Debug)]
pub struct SerializeContext {
    pub char_shape_ids: IdPool<u32>,
    pub para_shape_ids: IdPool<u16>,
    pub border_fill_ids: IdPool<u16>,
    pub tab_pr_ids: IdPool<u16>,
    pub numbering_ids: IdPool<u16>,
    pub style_ids: IdPool<u16>,
    /// `bin_data_id` (IR) → manifest 엔트리 매핑
    pub bin_data_map: HashMap<u16, BinDataEntry>,
    /// [#3546] OOXML 차트 파트 — Chart/chartN.xml 원형 방출 목록.
    /// 원본 content.hpf 는 Chart 파트를 나열하지 않으므로 manifest·3-way
    /// 단언 대상 밖이다.
    pub chart_entries: Vec<ChartPartEntry>,
    /// 문서 전역 문단 ID 카운터 — `<hp:p id="...">` 에 발급한다.
    para_id_counter: u32,
    /// subList(셀·글상자) 직렬화 중첩 깊이 (#1379 3단계).
    ///
    /// 본문 경로는 colPr 를 섹션 템플릿 첫 run 에서 처리하므로 인라인 미방출이
    /// 정합이지만, 셀·글상자 subList 의 colPr 는 원본 XML 에 인라인으로 존재한다.
    /// `render_control_slot` 의 ColumnDef 방출을 subList 경로(depth > 0)로 한정한다.
    pub sub_list_depth: u32,
    /// 본문 첫 문단의 첫 ColumnDef(섹션 템플릿 colPr 앵커가 흡수하는 단 정의)의
    /// **인라인 XML 방출만** 1회 억제하기 위한 consume-once 플래그 (#1584).
    ///
    /// ColumnDef 는 char-offset 슬롯(8유닛)을 점유하므로 `slots` 에는 그대로 남겨
    /// 위치 정합을 보존하되, 첫 ColumnDef 의 `<hp:colPr>` XML 은 템플릿이 이미
    /// 방출했으므로 중복 방지를 위해 건너뛴다. `write_section` 이 첫 문단 렌더 직전
    /// true 로 설정하고, 첫 본문 ColumnDef 방출 시 `render_control_slot` 이 소거한다.
    pub body_coldef_template_pending: bool,
    /// 이번 HWPX 산출물에서 발생한 사용자 내용 손실 (#4430).
    ///
    /// ID 풀과 마찬가지로 한 번의 직렬화 생명주기에만 속하며, 완료 시 바이트와 함께
    /// `SerializedDocument`로 이동한다.
    pub content_loss: ContentLossReport,
}

impl Default for SerializeContext {
    fn default() -> Self {
        Self {
            char_shape_ids: IdPool::default(),
            para_shape_ids: IdPool::default(),
            border_fill_ids: IdPool::default(),
            tab_pr_ids: IdPool::default(),
            numbering_ids: IdPool::default(),
            style_ids: IdPool::default(),
            bin_data_map: HashMap::new(),
            chart_entries: Vec::new(),
            para_id_counter: 0,
            sub_list_depth: 0,
            body_coldef_template_pending: false,
            content_loss: ContentLossReport::new(SerializedFormat::Hwpx),
        }
    }
}

impl SerializeContext {
    /// Document IR 전체를 1-pass 스캔하여 ID 풀을 채운다.
    ///
    /// Stage 0에서는 최소 등록(header.xml 리소스만)만 수행한다. Stage 1~4에서
    /// 각 writer가 추가되면서 `reference()` 호출과 스캔 범위가 확장된다.
    pub fn collect_from_document(doc: &Document) -> Self {
        let mut ctx = Self::default();

        // CharShape, ParaShape, BorderFill, TabDef, Numbering, Style, Font
        // 목록은 배열 인덱스가 곧 HWPX `id` 속성이 된다.
        for (idx, _) in doc.doc_info.char_shapes.iter().enumerate() {
            ctx.char_shape_ids.register(idx as u32);
        }
        for (idx, _) in doc.doc_info.para_shapes.iter().enumerate() {
            ctx.para_shape_ids.register(idx as u16);
        }
        // [#1384] borderFill id 는 1-based 방출(header.rs write_border_fill: idx+1)
        // 이고 borderFillIDRef 도 1-based 참조이므로, 등록도 1-based 로 맞춘다.
        // 종전 `idx`(0-based) 등록이라 마지막 id(예: exam_social 31)가 등록 범위
        // (0~30) 밖으로 빠져 SERIALIZE_FAIL(미등록 borderFillIDRef)을 유발했다.
        // 인라인 등록(표/셀, 아래)은 IR 값(1-based) 그대로라 본래 정합 — 이로써 통일.
        for (idx, _) in doc.doc_info.border_fills.iter().enumerate() {
            ctx.border_fill_ids.register((idx + 1) as u16);
        }
        for (idx, _) in doc.doc_info.tab_defs.iter().enumerate() {
            ctx.tab_pr_ids.register(idx as u16);
        }
        // [#1409] numbering id 는 1-based 방출(header.rs write_numbering: id+1)이고
        // 실물도 1-based 이므로 등록도 1-based 로 맞춘다 (#1384 borderFill 동형).
        // numbering 은 reference 검사가 없어 현재 미표면화이나, 등록 축 일관성 +
        // HWP5 변환·미래 검사 활성화 대비.
        for (idx, _) in doc.doc_info.numberings.iter().enumerate() {
            ctx.numbering_ids.register((idx + 1) as u16);
        }
        for (idx, _) in doc.doc_info.styles.iter().enumerate() {
            ctx.style_ids.register(idx as u16);
        }
        // [#1933 보강] style 0 은 `effective_style_id`/`write_styles` 계약상 "항상
        // 등록됨" 취급이다 — `<hh:styles>` 블록 자체가 없는(styles 비어있는) 정상
        // HWPX 도 파라그래프가 암묵적 기본 style 0을 참조할 수 있고, `write_styles`
        // 는 그 경우 블록을 그대로 생략한다(header.rs). 종전에는 `doc_info.styles`
        // 가 비어 있으면 0도 등록되지 않아, 있는 그대로 되돌려 쓰는 정상적인
        // round-trip이 `assert_all_refs_resolved` 에서 "미등록 styleIDRef: [0]" 로
        // 하드 실패했다(예: styles 블록이 없는 샘플의 HWPX 자기 왕복).
        ctx.style_ids.register(0);

        // 인라인 컨트롤(표/그림 등)의 borderFillIDRef를 사전 등록하여
        // assert_all_refs_resolved 검증 시 누락 방지. 중첩 표(셀 안의 표)와
        // 글상자·머리말/꼬리말·각주/미주 안의 표까지 재귀한다(아래
        // `register_border_fills_in_paragraphs` 참조, `table_extract::collect_from_paragraph`
        // 와 같은 위상의 재귀).
        for sec in &doc.sections {
            register_border_fills_in_paragraphs(&mut ctx, &sec.paragraphs, 0);
        }

        // BinData: bin_data_content의 storage_id → manifest 엔트리 생성.
        // manifest id 는 반드시 `image{bin_data_id}` — HWPX 파서(section.rs)가
        // binaryItemIDRef 의 숫자를 그대로 bin_data_id 로 파싱하므로(숫자 불변식),
        // 순번(i+1) 명명은 링크 항목으로 id 에 구멍이 있는 문서(#1891 73504)에서
        // 이름과 id 가 어긋나 재파스 그림 참조가 엉킨다.
        for bd in doc.bin_data_content.iter() {
            // [#3546] OOXML 차트 파트(HWPX 파서가 60000+N 으로 주입)는 BinData 가
            // 아니다 — 원본은 Chart/chartN.xml 이고 content.hpf 도 나열하지 않는다.
            // manifest 등록 없이 원형 경로로 별도 방출한다.
            if bd.extension == "ooxml_chart" {
                if let Some(n) = bd.id.checked_sub(60000).filter(|n| *n >= 1) {
                    ctx.chart_entries.push(ChartPartEntry {
                        href: format!("Chart/chart{}.xml", n),
                        bin_data_id: bd.id,
                    });
                    continue;
                }
            }
            // 빈 확장자는 원본과 동일하게 확장자 없이(`image{id}.`) 재직렬화한다.
            // 예전엔 `.bin` 기본값을 붙였으나(#1981), 원본이 확장자 없는 BinData
            // (`BinData/image13.` 등, OLE·미상 임베드)를 담은 경우 라운드트립 확장자
            // 멀티셋이 `bin` vs `""` 로 어긋나 PKG_FAIL 이 났다. 원본 형태를 보존한다.
            let manifest_id = format!("image{}", bd.id);
            let ext = bd.extension.as_str();
            let href = format!("BinData/{}.{}", manifest_id, ext);
            let media_type = mime_from_ext(ext);
            ctx.bin_data_map.insert(
                bd.id,
                BinDataEntry {
                    manifest_id,
                    href,
                    media_type: media_type.to_string(),
                    bin_data_id: bd.id,
                    is_embedded: true,
                },
            );
        }

        // 콘텐츠 없는 BinData: 바이트가 없어도 manifest 항목과 참조는 보존해야
        // 한다 (미등록이면 `write_img` 가 Err 를 반환해(picture.rs) 해당 <hp:pic>
        // 이 통째로 드롭되고 레이아웃 앵커까지 사라져 렌더가 갈라진다).
        //
        // [#1891] 은 외부 참조(Link)만 이 구멍을 막았으나, Embedding/Storage 도
        // 스트림이 없으면(parser/mod.rs 가 "BinData 스트림 없음" 경고 후 skip)
        // bin_data_content 가 비어 위 루프에 걸리지 않는다. 그 결과 두 루프를
        // 모두 빠져나가 같은 드롭이 재현됐다(#3526 hwpspec.hwp bin_data_id=37).
        // 따라서 data_type 을 가리지 않고 "아직 등록되지 않은 모든 항목"으로 넓힌다.
        //
        // ZIP 엔트리는 만들지 않고 content.hpf 에 isEmbeded="0" + href(외부 경로,
        // 없으면 빈 문자열)로만 방출한다 — mod.rs 가 ZIP 쓰기와 3-way 단언에서,
        // package_check 가 엔트리 실재 검사에서 각각 제외하므로 패키지는 정합하다.
        // 명명은 위와 같은 숫자 불변식(`image{storage_id}`)을 따른다.
        for bd in &doc.doc_info.bin_data_list {
            // storage_id=0 은 "참조 없는 placeholder pic" 센티널(#1567)과 겹치므로
            // 등록하지 않는다 (HWP5 Link 항목은 storage_id 미부여일 수 있음).
            if bd.storage_id == 0 || ctx.bin_data_map.contains_key(&bd.storage_id) {
                continue;
            }
            let ext = bd.extension.as_deref().unwrap_or("");
            ctx.bin_data_map.insert(
                bd.storage_id,
                BinDataEntry {
                    manifest_id: format!("image{}", bd.storage_id),
                    href: bd.abs_path.clone().unwrap_or_default(),
                    media_type: mime_from_ext(ext).to_string(),
                    bin_data_id: bd.storage_id,
                    is_embedded: false,
                },
            );
        }

        ctx
    }

    /// manifest·content.hpf 출력용 엔트리 목록 (삽입 순서 보존을 위해 `bin_data_id` 정렬).
    pub fn bin_data_entries(&self) -> Vec<BinDataEntry> {
        let mut v: Vec<_> = self.bin_data_map.values().cloned().collect();
        v.sort_by_key(|e| e.bin_data_id);
        v
    }

    /// `bin_data_id` → manifest id 조회 (Stage 4의 `<hc:img binaryItemIDRef="...">` 용).
    pub fn resolve_bin_id(&self, bin_data_id: u16) -> Option<&str> {
        self.bin_data_map
            .get(&bin_data_id)
            .map(|e| e.manifest_id.as_str())
    }

    /// 모든 참조가 해소되었는지 단언. 해소되지 않은 ID가 있으면 `SerializeError::XmlError` 반환.
    pub fn assert_all_refs_resolved(&self) -> Result<(), SerializeError> {
        let mut missing: Vec<String> = Vec::new();
        let cs = self.char_shape_ids.unresolved();
        if !cs.is_empty() {
            missing.push(format!("charPrIDRef: {:?}", cs));
        }
        let ps = self.para_shape_ids.unresolved();
        if !ps.is_empty() {
            missing.push(format!("paraPrIDRef: {:?}", ps));
        }
        let bf = self.border_fill_ids.unresolved();
        if !bf.is_empty() {
            missing.push(format!("borderFillIDRef: {:?}", bf));
        }
        let tp = self.tab_pr_ids.unresolved();
        if !tp.is_empty() {
            missing.push(format!("tabPrIDRef: {:?}", tp));
        }
        let nm = self.numbering_ids.unresolved();
        if !nm.is_empty() {
            missing.push(format!("numberingIDRef: {:?}", nm));
        }
        let st = self.style_ids.unresolved();
        if !st.is_empty() {
            missing.push(format!("styleIDRef: {:?}", st));
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(SerializeError::XmlError(format!(
                "미등록 ID 참조 발견: {}",
                missing.join("; ")
            )))
        }
    }

    /// 문서 전역 문단 ID를 하나 발급하고 카운터를 증가시킨다.
    pub fn next_para_id(&mut self) -> u32 {
        let id = self.para_id_counter;
        self.para_id_counter += 1;
        id
    }

    /// [Issue #1933] 스타일 목록 밖 styleIDRef 를 기본 스타일(0)로 강등한다.
    ///
    /// 일부 생성기 산출물(보도자료 계열)은 header 스타일 목록에 없는 style_id 를
    /// 문단이 참조한다(파일 자기모순). 종전에는 `assert_all_refs_resolved` 가
    /// 하드 실패해 "열리는데 저장 불가" 상태가 됐다. 한글은 이런 문서를 기본
    /// 스타일로 폴백해 열고 저장하므로, 미등록 참조는 0(항상 등록됨)으로 강등한다.
    /// 등록된 참조는 그대로 반환한다.
    pub fn effective_style_id(&self, raw: u8) -> u8 {
        if self.style_ids.is_registered(&(raw as u16)) {
            raw
        } else {
            0
        }
    }
}

/// 재귀 스캔 최대 깊이 — `table_extract::MAX_NEST_DEPTH` 와 같은 값.
/// 순환 참조가 없는 정상 IR에서는 닿지 않고, 적대적/손상 입력에서 무한 재귀를 막는다.
const MAX_BORDER_FILL_SCAN_DEPTH: usize = 8;

/// 표 하나의 `border_fill_id`(표/영역/셀)를 등록한다.
fn register_table_border_fills(ctx: &mut SerializeContext, table: &crate::model::table::Table) {
    ctx.border_fill_ids.register(table.border_fill_id);
    for zone in &table.zones {
        ctx.border_fill_ids.register(zone.border_fill_id);
    }
    for cell in &table.cells {
        ctx.border_fill_ids.register(cell.border_fill_id);
    }
}

/// 문단 목록을 재귀하며 표(및 중첩 표)의 `border_fill_id`를 사전 등록한다.
///
/// `table_extract::collect_from_paragraph`/`nested_tables`(#3719 계열 표 추출)와 같은
/// 위상의 재귀 — 표 셀 안의 표(중첩 표), 글상자, 머리말/꼬리말, 각주/미주까지 내려간다.
/// 종전에는 `doc.sections[].paragraphs[]`의 최상위 `Control::Table`만 훑어 셀 안에 중첩된
/// 표나 글상자·머리말/꼬리말·각주/미주 안의 표의 `border_fill_id`가 등록에서 빠졌다.
/// 그 표가 실제 직렬화(`table.rs`의 `reference()` 호출)에서 참조되면
/// `assert_all_refs_resolved`가 "미등록 ID 참조 발견"으로 하드 실패해 문서 전체의
/// `export-hwpx`가 산출물 없이 실패했다(실측: 정부 보고서 표 107개 중 중첩 표 1개를 가진
/// 문서에서 `borderFillIDRef: [0]` 미등록으로 재현).
///
/// [gestell 리뷰] 첫 수정은 6개 소유자(표 셀·글상자·머리말·꼬리말·각주·미주)만 돌았다.
/// OWPML 은 문단 리스트를 8개 자리에 둘 수 있다 — 나머지 2개(캡션, 필드 메모)와 함께
/// `Control::Picture`(자체 캡션을 갖는 별도 컨트롤)·`Control::HiddenComment` 도 이 재귀 밖에
/// 있었다. `#2736`(전수 조사: 순회 × 컨테이너 행렬)이 이미 지적한 "공유 방문자 부재로 반복되는
/// 재귀 누락"과 같은 계열 — 이번 세션의 `#4321`/PR #4365(`injection_scan.rs`)가 캡션·그림·필드
/// 메모 누락을 고친 것과 동형이다.
fn register_border_fills_in_paragraphs(
    ctx: &mut SerializeContext,
    paragraphs: &[crate::model::paragraph::Paragraph],
    depth: usize,
) {
    if depth >= MAX_BORDER_FILL_SCAN_DEPTH {
        return;
    }
    for para in paragraphs {
        for ctrl in &para.controls {
            register_border_fills_in_control(ctx, ctrl, depth);
        }
    }
}

fn register_border_fills_in_control(ctx: &mut SerializeContext, ctrl: &Control, depth: usize) {
    match ctrl {
        Control::Table(tbl) => {
            register_table_border_fills(ctx, tbl);
            for cell in &tbl.cells {
                register_border_fills_in_paragraphs(ctx, &cell.paragraphs, depth + 1);
            }
            if let Some(caption) = &tbl.caption {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
        }
        Control::Shape(shape) => {
            register_border_fills_in_shape(ctx, shape, depth);
        }
        // 독립 그림 컨트롤(글상자/묶음 밖) — 자체 `caption` 을 갖는다(`src/model/image.rs`).
        Control::Picture(pic) => {
            if let Some(caption) = &pic.caption {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
        }
        Control::Header(h) => {
            register_border_fills_in_paragraphs(ctx, &h.paragraphs, depth + 1);
        }
        Control::Footer(f) => {
            register_border_fills_in_paragraphs(ctx, &f.paragraphs, depth + 1);
        }
        Control::Footnote(f) => {
            register_border_fills_in_paragraphs(ctx, &f.paragraphs, depth + 1);
        }
        Control::Endnote(e) => {
            register_border_fills_in_paragraphs(ctx, &e.paragraphs, depth + 1);
        }
        // 숨은 설명(메모) — 화면에 안 보여도 파일에는 문단 리스트로 존재한다.
        Control::HiddenComment(hc) => {
            register_border_fills_in_paragraphs(ctx, &hc.paragraphs, depth + 1);
        }
        // 필드(누름틀 등) 메모 — `Field.memo_paragraphs`.
        Control::Field(f) => {
            register_border_fills_in_paragraphs(ctx, &f.memo_paragraphs, depth + 1);
        }
        _ => {}
    }
}

/// `ShapeObject`(그리기 개체) 하나의 글상자·캡션과, 묶음이면 자식 개체까지 재귀한다.
///
/// 캡션 자리는 변형마다 다르다(#4319 가 렌더 쪽에서 이미 지적한 비대칭과 같은 축):
/// - 기본 6종(Line/Rectangle/Ellipse/Arc/Polygon/Curve): `drawing.caption`
/// - `Group`/`Picture`(묶음 내 자식으로서의 그림): 자기 struct 의 `caption`
/// - `Chart`/`Ole`: 자기 struct 의 `caption` — 파서(`src/parser/control/shape.rs:213,222`)가
///   `drawing.caption` 을 `.take()` 로 옮기므로 `drawing.caption` 은 항상 `None` 이다. 그래도
///   방어적으로 두 자리를 모두 확인한다(렌더의 `shape_caption_for_layout` 폴백과 동형).
fn register_border_fills_in_shape(
    ctx: &mut SerializeContext,
    shape: &crate::model::shape::ShapeObject,
    depth: usize,
) {
    if depth >= MAX_BORDER_FILL_SCAN_DEPTH {
        return;
    }
    use crate::model::shape::ShapeObject;

    if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
        register_border_fills_in_paragraphs(ctx, &tb.paragraphs, depth + 1);
    }

    match shape {
        ShapeObject::Group(g) => {
            if let Some(caption) = &g.caption {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
            for child in &g.children {
                register_border_fills_in_shape(ctx, child, depth + 1);
            }
        }
        ShapeObject::Picture(p) => {
            if let Some(caption) = &p.caption {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
        }
        ShapeObject::Chart(c) => {
            if let Some(caption) = c.caption.as_ref().or(c.drawing.caption.as_ref()) {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
        }
        ShapeObject::Ole(o) => {
            if let Some(caption) = o.caption.as_ref().or(o.drawing.caption.as_ref()) {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
        }
        ShapeObject::Line(_)
        | ShapeObject::Rectangle(_)
        | ShapeObject::Ellipse(_)
        | ShapeObject::Arc(_)
        | ShapeObject::Polygon(_)
        | ShapeObject::Curve(_) => {
            if let Some(caption) = shape.drawing().and_then(|d| d.caption.as_ref()) {
                register_border_fills_in_paragraphs(ctx, &caption.paragraphs, depth + 1);
            }
        }
    }
}

fn mime_from_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_doc_has_no_registered_ids() {
        let doc = Document::default();
        let ctx = SerializeContext::collect_from_document(&doc);
        assert_eq!(ctx.char_shape_ids.registered_count(), 0);
        assert_eq!(ctx.para_shape_ids.registered_count(), 0);
        assert!(ctx.bin_data_map.is_empty());
    }

    #[test]
    fn empty_doc_passes_ref_resolution() {
        let doc = Document::default();
        let ctx = SerializeContext::collect_from_document(&doc);
        ctx.assert_all_refs_resolved().expect("empty doc must pass");
    }

    /// [Issue #4395] `doc_info.styles` 가 비어 있어도(=`<hh:styles>` 블록 없음) style id 0 은
    /// 항상 등록된 것으로 취급해야 한다 — `effective_style_id`(아래)의 기존 계약이자,
    /// `write_styles`(header.rs)가 빈 목록일 때 블록 자체를 생략하는 정상 동작과 짝을 이룬다.
    /// 수정 전에는 `doc.doc_info.styles` 순회로만 등록해서 목록이 비면 0 도 미등록으로 남아,
    /// styleIDRef=0 을 참조하는 문단이 있는 문서(예: `samples/task2156/width_ladder.hwpx`)의
    /// 저장 자체가 "미등록 ID 참조 발견: styleIDRef: [0]" 로 하드 실패했다.
    #[test]
    fn style_zero_always_registered_without_explicit_style_list() {
        let doc = Document::default();
        assert!(doc.doc_info.styles.is_empty());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        ctx.style_ids.reference(0);
        ctx.assert_all_refs_resolved()
            .expect("style 0 must always be registered, even with an empty style list (#4395)");
    }

    #[test]
    fn task1384_border_fill_registered_one_based() {
        // borderFill 은 1-based(방출 id=idx+1, borderFillIDRef 1-based)이므로
        // N 개 적재 시 마지막 참조 N 이 resolved 되어야 한다 (#1384 — 종전 0-based
        // 등록이라 N 이 미등록으로 SERIALIZE_FAIL 했다).
        use crate::model::style::BorderFill;
        let mut doc = Document::default();
        doc.doc_info.border_fills = vec![BorderFill::default(); 31];
        let mut ctx = SerializeContext::collect_from_document(&doc);
        // exam_social 패턴: charPr 가 borderFillIDRef=31(마지막) 참조.
        ctx.border_fill_ids.reference(31);
        ctx.assert_all_refs_resolved()
            .expect("1-based 등록이면 borderFillIDRef=31 resolved");
        // 0 은 1-based 축에 없음(미등록) — 회귀 가드 의미 명시.
        assert!(!ctx.border_fill_ids.is_registered(&0));
        assert!(ctx.border_fill_ids.is_registered(&31));
    }

    #[test]
    fn task1409_numbering_registered_one_based() {
        // numbering 도 1-based(방출 id=idx+1, 실물 1-based) — borderFill 동형(#1384).
        // N 개 적재 시 마지막 id=N 이 등록되고 0 은 미등록이어야 한다.
        use crate::model::style::Numbering;
        let mut doc = Document::default();
        doc.doc_info.numberings = vec![Numbering::default(); 8];
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.numbering_ids.is_registered(&8),
            "1-based 등록이면 마지막 numbering id=8 등록"
        );
        assert!(
            !ctx.numbering_ids.is_registered(&0),
            "0 은 1-based 축에 없음 (회귀 가드)"
        );
    }

    #[test]
    fn nested_table_border_fill_id_registered() {
        // 실측(정부 보고서, 표 107개 중 1개가 셀 안에 표를 담은 문서)에서 재현: 종전에는
        // `doc.sections[].paragraphs[]`의 최상위 `Control::Table`만 훑어 셀 안에 중첩된
        // 표의 `border_fill_id`가 등록에서 빠졌다. 그 표가 실제 직렬화에서 참조되면
        // `assert_all_refs_resolved`가 "미등록 ID 참조 발견"으로 하드 실패해 문서 전체의
        // `export-hwpx`가 산출물 없이 실패했다 — 이 테스트는 그 등록 누락의 회귀 가드다.
        use crate::model::paragraph::Paragraph;
        use crate::model::table::{Cell, Table};

        const OUTER_BORDER_FILL_ID: u16 = 1;
        const INNER_BORDER_FILL_ID: u16 = 2; // outer와 달라 별도 등록이 필요함을 보장.

        let mut inner_table = Table::default();
        inner_table.border_fill_id = INNER_BORDER_FILL_ID;
        inner_table.row_count = 1;
        inner_table.col_count = 1;
        inner_table.cells = vec![Cell::new_empty(0, 0, 1000, 1000, INNER_BORDER_FILL_ID)];

        let mut outer_cell = Cell::new_empty(0, 0, 5000, 5000, OUTER_BORDER_FILL_ID);
        outer_cell.paragraphs = vec![{
            let mut p = Paragraph::new_empty();
            p.controls.push(Control::Table(Box::new(inner_table)));
            p
        }];

        let mut outer_table = Table::default();
        outer_table.border_fill_id = OUTER_BORDER_FILL_ID;
        outer_table.row_count = 1;
        outer_table.col_count = 1;
        outer_table.cells = vec![outer_cell];

        let mut doc = Document::default();
        let mut para = Paragraph::new_empty();
        para.controls.push(Control::Table(Box::new(outer_table)));
        doc.sections = vec![crate::model::document::Section {
            paragraphs: vec![para],
            ..Default::default()
        }];

        let mut ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&INNER_BORDER_FILL_ID),
            "중첩 표(셀 안의 표)의 border_fill_id도 사전 등록되어야 한다"
        );

        // table.rs 의 실제 직렬화 경로와 동형: 중첩 표가 참조하는 시점을 재현해도
        // assert_all_refs_resolved 가 통과해야 한다(수정 전에는 여기서 실패했다).
        ctx.border_fill_ids.reference(OUTER_BORDER_FILL_ID);
        ctx.border_fill_ids.reference(INNER_BORDER_FILL_ID);
        ctx.assert_all_refs_resolved()
            .expect("중첩 표 border_fill_id 참조가 미등록으로 남으면 안 된다");
    }

    // ── [gestell 리뷰] 문단 리스트 8개 소유자 전수 — 각 자리에 표를 하나씩 심어
    // border_fill_id 사전 등록을 개별로 확인한다. 표 셀은 위 테스트가 이미 덮는다.

    /// 표 하나(1×1)를 만들어 스스로 border_fill_id 를 갖게 한다 — 소유자 테스트 공용 헬퍼.
    fn table_with_border_fill(id: u16) -> crate::model::table::Table {
        use crate::model::table::{Cell, Table};
        let mut t = Table::default();
        t.border_fill_id = id;
        t.row_count = 1;
        t.col_count = 1;
        t.cells = vec![Cell::new_empty(0, 0, 1000, 1000, id)];
        t
    }

    /// 문단 하나에 표 컨트롤 하나를 심은 문단 리스트 — 캡션/메모/숨은설명 등
    /// `Vec<Paragraph>` 필드에 그대로 대입해 쓴다.
    fn paragraphs_with_table(id: u16) -> Vec<crate::model::paragraph::Paragraph> {
        use crate::model::paragraph::Paragraph;
        let mut p = Paragraph::new_empty();
        p.controls
            .push(Control::Table(Box::new(table_with_border_fill(id))));
        vec![p]
    }

    /// 컨트롤 하나를 문서 최상위 문단에 심는다 — 소유자 테스트 공용 헬퍼.
    fn doc_with_top_control(ctrl: Control) -> Document {
        use crate::model::paragraph::Paragraph;
        use crate::model::style::{CharShape, ParaShape, Style};
        let mut doc = Document::default();
        // 실제 문서는 항상 기본 char_shape/para_shape/style 을 최소 1개씩 갖는다 —
        // 완전히 빈 doc_info 는 para_shape_id=0/style_id=0(필드 기본값) 참조조차
        // 미등록으로 만들어, border_fill_id 축과 무관한 잡음으로 end-to-end 직렬화가
        // 실패한다. 소유자별 회귀 테스트가 실제로 검사하려는 축(border_fill_id)만
        // 남기기 위해 기본 항목을 채운다.
        doc.doc_info.char_shapes = vec![CharShape::default()];
        doc.doc_info.para_shapes = vec![ParaShape::default()];
        doc.doc_info.styles = vec![Style::default()];
        let mut para = Paragraph::new_empty();
        para.controls.push(ctrl);
        doc.sections = vec![crate::model::document::Section {
            paragraphs: vec![para],
            ..Default::default()
        }];
        doc
    }

    #[test]
    fn table_caption_border_fill_id_registered() {
        // 표 캡션(Table.caption) 안의 표 — 캡션은 8개 문단 리스트 소유자 중 하나다.
        use crate::model::shape::Caption;
        const ID: u16 = 21;
        let mut outer = table_with_border_fill(1);
        outer.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Table(Box::new(outer)));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "표 캡션 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    /// [gestell 리뷰 요청] 실제 export-hwpx 진입점(`serialize_hwpx`)까지 물려서 확인한다 —
    /// `SerializeContext` 등록만 보는 위 테스트보다 실측(#4408 재현)에 가까운 형태. 표
    /// 캡션 안에 표를 담은 실제 코퍼스 문서를 찾지 못해(§보고 — 재현 문서 없음, 최소 IR로
    /// 구성) `serialize_hwpx` 를 직접 호출해 전체 파이프라인이 크래시 없이 끝나는지 본다.
    #[test]
    fn table_in_caption_serializes_end_to_end_without_crash() {
        use crate::model::shape::Caption;
        const ID: u16 = 121;
        let mut outer = table_with_border_fill(1);
        outer.row_count = 1;
        outer.col_count = 1;
        outer.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Table(Box::new(outer)));
        let result = crate::serializer::hwpx::serialize_hwpx(&doc);
        assert!(
            result.is_ok(),
            "표 캡션 안에 표가 있으면 export-hwpx 가 산출물 없이 실패해서는 안 된다: {:?}",
            result.err()
        );
    }

    #[test]
    fn shape_default_caption_border_fill_id_registered() {
        // 기본 6종 도형(Line 등)의 캡션은 `drawing.caption` 에 있다.
        use crate::model::shape::{Caption, LineShape, ShapeObject};
        const ID: u16 = 22;
        let mut line = LineShape::default();
        line.drawing.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Shape(Box::new(ShapeObject::Line(line))));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "기본 도형 drawing.caption 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn shape_group_caption_and_child_border_fill_id_registered() {
        // Group 은 자기 struct 의 `caption` 을 쓰고(drawing 이 없음), 자식 개체도 재귀해야 한다.
        use crate::model::shape::{Caption, GroupShape, LineShape, ShapeObject};
        const GROUP_CAPTION_ID: u16 = 23;
        const CHILD_CAPTION_ID: u16 = 24;

        let mut child_line = LineShape::default();
        child_line.drawing.caption = Some(Caption {
            paragraphs: paragraphs_with_table(CHILD_CAPTION_ID),
            ..Default::default()
        });

        let mut group = GroupShape::default();
        group.caption = Some(Caption {
            paragraphs: paragraphs_with_table(GROUP_CAPTION_ID),
            ..Default::default()
        });
        group.children = vec![ShapeObject::Line(child_line)];

        let doc = doc_with_top_control(Control::Shape(Box::new(ShapeObject::Group(group))));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&GROUP_CAPTION_ID),
            "Group.caption 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
        assert!(
            ctx.border_fill_ids.is_registered(&CHILD_CAPTION_ID),
            "Group 자식 개체(재귀)의 캡션 안 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn shape_picture_variant_caption_border_fill_id_registered() {
        // 묶음 내 자식으로서의 그림(ShapeObject::Picture) — 자기 struct 의 `caption`.
        use crate::model::image::Picture;
        use crate::model::shape::{Caption, ShapeObject};
        const ID: u16 = 25;
        let mut pic = Picture::default();
        pic.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Shape(Box::new(ShapeObject::Picture(Box::new(
            pic,
        )))));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "ShapeObject::Picture.caption 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn shape_chart_caption_border_fill_id_registered() {
        // Chart — 파서가 drawing.caption 을 자기 struct 의 caption 으로 옮긴다(#4319 계열).
        // 두 자리 다 방어적으로 확인하므로 own-field 경로를 실측한다.
        use crate::model::shape::{Caption, ChartShape, ShapeObject};
        const ID: u16 = 26;
        let mut chart = ChartShape::default();
        chart.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Shape(Box::new(ShapeObject::Chart(Box::new(
            chart,
        )))));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "Chart.caption 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn shape_ole_caption_border_fill_id_registered() {
        // Ole — Chart 와 같은 축(own-field caption).
        use crate::model::shape::{Caption, OleShape, ShapeObject};
        const ID: u16 = 27;
        let mut ole = OleShape::default();
        ole.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole)))));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "Ole.caption 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn control_picture_caption_border_fill_id_registered() {
        // 독립 그림 컨트롤(Control::Picture) — ShapeObject::Picture 와 별개 축.
        // 종전 코드는 Control::Picture 자체를 재귀 match 에서 다루지 않았다.
        use crate::model::image::Picture;
        use crate::model::shape::Caption;
        const ID: u16 = 28;
        let mut pic = Picture::default();
        pic.caption = Some(Caption {
            paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        });
        let doc = doc_with_top_control(Control::Picture(Box::new(pic)));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "Control::Picture.caption 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn hidden_comment_border_fill_id_registered() {
        // 숨은 설명(HiddenComment) — 화면에 안 보여도 파일에는 문단 리스트로 존재한다.
        use crate::model::control::HiddenComment;
        const ID: u16 = 29;
        let hc = HiddenComment {
            paragraphs: paragraphs_with_table(ID),
        };
        let doc = doc_with_top_control(Control::HiddenComment(Box::new(hc)));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "HiddenComment 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn field_memo_paragraphs_border_fill_id_registered() {
        // 필드(누름틀 등) 메모 — `Field.memo_paragraphs`.
        use crate::model::control::Field;
        const ID: u16 = 30;
        let field = Field {
            memo_paragraphs: paragraphs_with_table(ID),
            ..Default::default()
        };
        let doc = doc_with_top_control(Control::Field(field));
        let ctx = SerializeContext::collect_from_document(&doc);
        assert!(
            ctx.border_fill_ids.is_registered(&ID),
            "Field.memo_paragraphs 안의 표 border_fill_id도 사전 등록되어야 한다"
        );
    }

    #[test]
    fn issue1981_empty_extension_bindata_keeps_no_ext() {
        // 빈 확장자 BinData 는 `.bin` 을 붙이지 않고 원본 형태(`image{id}.`)로
        // 재직렬화해야 한다 — 라운드트립 확장자 멀티셋 보존(#1981).
        use crate::model::bin_data::BinDataContent;
        let mut doc = Document::default();
        doc.bin_data_content.push(BinDataContent {
            id: 6,
            data: vec![0, 1, 2].into(),
            extension: String::new(),
        });
        doc.bin_data_content.push(BinDataContent {
            id: 7,
            data: vec![3, 4, 5].into(),
            extension: "bmp".to_string(),
        });
        let ctx = SerializeContext::collect_from_document(&doc);
        let e6 = &ctx.bin_data_map[&6];
        assert_eq!(e6.href, "BinData/image6.", "빈 확장자는 .bin 금지");
        assert_eq!(e6.media_type, "application/octet-stream");
        let e7 = &ctx.bin_data_map[&7];
        assert_eq!(e7.href, "BinData/image7.bmp");
    }

    #[test]
    fn issue3526_contentless_embedding_bindata_is_registered() {
        // [#3526] 스트림이 없어 `bin_data_content` 가 비는 Embedding/Storage 항목도
        // manifest 에 등록돼야 한다. 미등록이면 picture.rs `write_img` 가 Err 를
        // 반환해 <hp:pic> 이 통째로 드롭되고 앵커·레이아웃까지 사라진다
        // (hwpspec.hwp bin_data_id=37). [#1891] 은 Link 만 막아서 Embedding/Storage
        // 는 두 등록 루프를 모두 빠져나갔다.
        use crate::model::bin_data::{BinData, BinDataContent, BinDataType};
        let mut doc = Document::default();
        // 정상 항목(콘텐츠 보유) — 넓힌 루프가 이걸 덮어쓰면 안 된다.
        doc.bin_data_content.push(BinDataContent {
            id: 1,
            data: vec![0, 1, 2].into(),
            extension: "png".to_string(),
        });
        doc.doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Embedding,
            storage_id: 1,
            extension: Some("png".to_string()),
            ..Default::default()
        });
        // 스트림 부재 재현: 목록에는 있으나 콘텐츠가 없는 Embedding / Storage.
        doc.doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Embedding,
            storage_id: 37,
            extension: Some("jpg".to_string()),
            ..Default::default()
        });
        doc.doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Storage,
            storage_id: 38,
            extension: Some("OLE".to_string()),
            ..Default::default()
        });

        let ctx = SerializeContext::collect_from_document(&doc);

        // `write_img`(picture.rs) 의 분기 조건 그 자체 — Some 이어야 pic 이 산다.
        assert_eq!(
            ctx.resolve_bin_id(37),
            Some("image37"),
            "콘텐츠 없는 Embedding 도 등록돼야 <hp:pic> 이 드롭되지 않는다"
        );
        assert_eq!(
            ctx.resolve_bin_id(38),
            Some("image38"),
            "콘텐츠 없는 Storage 도 동일하게 등록"
        );

        let e37 = &ctx.bin_data_map[&37];
        assert!(
            !e37.is_embedded,
            "ZIP 엔트리가 없으므로 isEmbeded=0 (mod.rs 3-way 단언 제외 대상)"
        );
        assert_eq!(e37.media_type, "image/jpeg");
        // abs_path 없는 Embedding 은 빈 href — populate_link_image_paths 가 빈
        // 경로를 걸러내므로(parser/mod.rs) 허위 external_path 가 생기지 않는다.
        assert_eq!(e37.href, "", "존재하지 않는 ZIP 경로를 가리키면 안 된다");

        // 콘텐츠 보유 항목은 그대로 임베드로 남아야 한다(덮어쓰기 회귀 가드).
        let e1 = &ctx.bin_data_map[&1];
        assert!(e1.is_embedded, "콘텐츠 보유 항목은 isEmbeded=1 유지");
        assert_eq!(e1.href, "BinData/image1.png");
    }

    #[test]
    fn unresolved_char_pr_fails() {
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        ctx.char_shape_ids.reference(42); // 등록되지 않은 ID 참조
        let err = ctx.assert_all_refs_resolved().unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("charPrIDRef"),
            "error message should name charPrIDRef: {}",
            msg
        );
        assert!(
            msg.contains("42"),
            "error message should include id 42: {}",
            msg
        );
    }

    #[test]
    fn id_pool_register_reference_roundtrip() {
        let mut pool: IdPool<u32> = IdPool::new();
        pool.register(1);
        pool.register(2);
        pool.reference(1);
        pool.reference(3); // 미등록
        assert!(pool.is_registered(&1));
        assert!(!pool.is_registered(&3));
        assert_eq!(pool.unresolved(), vec![3]);
    }

    #[test]
    fn mime_from_ext_covers_common_formats() {
        assert_eq!(mime_from_ext("png"), "image/png");
        assert_eq!(mime_from_ext("PNG"), "image/png");
        assert_eq!(mime_from_ext("jpg"), "image/jpeg");
        assert_eq!(mime_from_ext("unknown"), "application/octet-stream");
    }
}
