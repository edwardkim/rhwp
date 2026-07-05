//! 바탕쪽(MasterPage) HWPX 직렬화.
//!
//! 바탕쪽은 패키지의 별도 파일 `Contents/masterpage{N}.xml` (`<masterPage>` 루트)로 저장되고,
//! content.hpf manifest 의 `<opf:item id="masterpage{N}">` 로 등록되며, section XML 의 secPr
//! 내부 `<hp:masterPage idRef="masterpage{N}"/>` 로 참조된다. 파서(parse_hwpx_master_page)의 역.
//!
//! 종전 HWPX 직렬화는 바탕쪽을 전혀 쓰지 않아 라운드트립에서 소실되었다(렌더 노드 손실 →
//! 시각 회귀). 본 모듈이 그 직렬화 축을 채운다.

use crate::model::header_footer::{HeaderFooterApply, MasterPage};

use super::context::SerializeContext;
use super::section::{render_hp_p_open, render_paragraph_parts};

/// `<masterPage>`/section XML 공용 네임스페이스 블록 (empty_section0.xml 와 동일).
const XMLNS: &str = r#"xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hp10="http://www.hancom.co.kr/hwpml/2016/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core" xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head" xmlns:hhs="http://www.hancom.co.kr/hwpml/2011/history" xmlns:hm="http://www.hancom.co.kr/hwpml/2011/master-page" xmlns:hpf="http://www.hancom.co.kr/schema/2011/hpf" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf/" xmlns:ooxmlchart="http://www.hancom.co.kr/hwpml/2016/ooxmlchart" xmlns:hwpunitchar="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar" xmlns:epub="http://www.idpf.org/2007/ops" xmlns:config="urn:oasis:names:tc:opendocument:xmlns:config:1.0""#;

/// 바탕쪽 적용 유형 문자열 (parser `parse_hwpx_master_page_type` 의 역).
/// 확장 바탕쪽(is_extension)은 ext_flags 의 optional 비트(0x04)로 LAST_PAGE/OPTIONAL_PAGE 구분.
fn master_page_type_str(mp: &MasterPage) -> &'static str {
    if mp.is_extension {
        if mp.ext_flags & 0x04 != 0 {
            "OPTIONAL_PAGE"
        } else {
            "LAST_PAGE"
        }
    } else {
        match mp.apply_to {
            HeaderFooterApply::Even => "EVEN",
            HeaderFooterApply::Odd => "ODD",
            HeaderFooterApply::Both => "BOTH",
        }
    }
}

/// pageDuplicate 문자열 (parser 의 역). LAST_PAGE 는 replace_base 면 "0", 그 외 overlap 기준.
fn page_duplicate_str(mp: &MasterPage) -> &'static str {
    if mp.is_extension && mp.ext_flags & 0x04 == 0 {
        // LAST_PAGE
        if mp.replace_base {
            "0"
        } else {
            "1"
        }
    } else if mp.overlap {
        "1"
    } else {
        "0"
    }
}

/// 바탕쪽 1개를 `Contents/masterpage{N}.xml` 내용으로 직렬화한다.
/// `id` 는 manifest/idRef 와 일치해야 하는 식별자(예: `masterpage0`).
pub fn render_master_page_xml(mp: &MasterPage, id: &str, ctx: &mut SerializeContext) -> String {
    let mut body = String::new();
    let mut vert_cursor: u32 = 0;
    for p in &mp.paragraphs {
        let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        let pid = ctx.next_para_id();
        let sid = ctx.effective_style_id(p.style_id);
        body.push_str(&render_hp_p_open(p, pid, sid));
        body.push_str(&runs);
        body.push_str(&linesegs);
        body.push_str("</hp:p>");
    }

    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#,
            r#"<masterPage {xmlns} id="{id}" type="{ty}" pageNumber="{pn}" pageDuplicate="{pd}" pageFront="0">"#,
            r#"<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="{tw}" textHeight="{th}" hasTextRef="{tr}" hasNumRef="{nr}">"#,
            "{body}",
            r#"</hp:subList></masterPage>"#,
        ),
        xmlns = XMLNS,
        id = id,
        ty = master_page_type_str(mp),
        pn = mp.hwpx_page_number.unwrap_or(0),
        pd = page_duplicate_str(mp),
        tw = mp.text_width,
        th = mp.text_height,
        tr = mp.text_ref,
        nr = mp.num_ref,
        body = body,
    )
}

/// secPr 내부에 삽입할 `<hp:masterPage idRef="..."/>` 참조 묶음.
/// `ids` 는 이 섹션 바탕쪽들의 식별자(전역 인덱스 기반).
pub fn render_master_page_refs(ids: &[String]) -> String {
    ids.iter()
        .map(|id| format!(r#"<hp:masterPage idRef="{id}"/>"#))
        .collect()
}
