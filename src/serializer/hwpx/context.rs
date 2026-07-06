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
    /// content.hpf `isEmbeded` — false 면 외부 파일 참조(ZIP 엔트리 없음, #1891).
    pub is_embedded: bool,
}

/// 1-pass 스캔으로 구축되는 직렬화 컨텍스트.
#[derive(Debug, Default)]
pub struct SerializeContext {
    pub char_shape_ids: IdPool<u32>,
    pub para_shape_ids: IdPool<u16>,
    pub border_fill_ids: IdPool<u16>,
    pub tab_pr_ids: IdPool<u16>,
    pub numbering_ids: IdPool<u16>,
    pub style_ids: IdPool<u16>,
    /// `bin_data_id` (IR) → manifest 엔트리 매핑
    pub bin_data_map: HashMap<u16, BinDataEntry>,
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

        // 인라인 컨트롤(표/그림 등)의 borderFillIDRef를 사전 등록하여
        // assert_all_refs_resolved 검증 시 누락 방지.
        for sec in &doc.sections {
            for para in &sec.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Table(tbl) = ctrl {
                        ctx.border_fill_ids.register(tbl.border_fill_id);
                        for zone in &tbl.zones {
                            ctx.border_fill_ids.register(zone.border_fill_id);
                        }
                        for cell in &tbl.cells {
                            ctx.border_fill_ids.register(cell.border_fill_id);
                        }
                    }
                }
            }
        }

        // BinData: bin_data_content의 storage_id → manifest 엔트리 생성.
        // manifest id 는 반드시 `image{bin_data_id}` — HWPX 파서(section.rs)가
        // binaryItemIDRef 의 숫자를 그대로 bin_data_id 로 파싱하므로(숫자 불변식),
        // 순번(i+1) 명명은 링크 항목으로 id 에 구멍이 있는 문서(#1891 73504)에서
        // 이름과 id 가 어긋나 재파스 그림 참조가 엉킨다.
        for bd in doc.bin_data_content.iter() {
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

        // 외부 참조(Link) BinData: 콘텐츠가 없어도 manifest 항목과 참조는 보존해야
        // 한다 (#1891 — 미등록이면 해당 <hp:pic> 직렬화가 실패해 그림 컨트롤이
        // 통째로 드롭되고 레이아웃 앵커가 사라져 렌더가 갈라진다). ZIP 엔트리는
        // 만들지 않고 content.hpf 에 isEmbeded="0" + 원본 href 로만 방출한다.
        // 명명은 위와 같은 숫자 불변식(`image{storage_id}`)을 따른다.
        for bd in &doc.doc_info.bin_data_list {
            if !matches!(bd.data_type, crate::model::bin_data::BinDataType::Link) {
                continue;
            }
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
    fn issue1981_empty_extension_bindata_keeps_no_ext() {
        // 빈 확장자 BinData 는 `.bin` 을 붙이지 않고 원본 형태(`image{id}.`)로
        // 재직렬화해야 한다 — 라운드트립 확장자 멀티셋 보존(#1981).
        use crate::model::bin_data::BinDataContent;
        let mut doc = Document::default();
        doc.bin_data_content.push(BinDataContent {
            id: 6,
            data: vec![0, 1, 2],
            extension: String::new(),
        });
        doc.bin_data_content.push(BinDataContent {
            id: 7,
            data: vec![3, 4, 5],
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
