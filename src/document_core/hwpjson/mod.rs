//! 한글 클립보드 문서모델(hwpjson) → HWPX XML 변환.
//!
//! 한글은 Ctrl+C 시 클립보드 HTML 끝에 `<!--[data-hwpjson]{…}-->` 주석으로 **문서 모델 전체**를
//! 실어 보낸다(실측 1.08MB). HTML 에는 없는 글꼴 등록·문단모양·쪽 설정·표 셀 속성·그림 원본이
//! 여기 다 들어 있어, 이것을 읽어야 원본과 같은 조판이 나온다.
//!
//! 이 모듈은 그 모델을 **HWPX 의 header.xml / section0.xml 문자열**로 옮긴다. 매핑이 HWPX 와
//! 1:1 이라 새 IR 매퍼를 쓰지 않고 검증된 HWPX 파서를 그대로 태울 수 있다.
//!
//! 사양은 원본 HWPX 를 정답지로 두고 실측 확정했다(문단 259/259·표 15/15·셀 63/63·
//! 요소 속성 3,824/3,824 일치, 원시 문자열 153,523자 동일).
//!
//! 구현은 검증이 끝난 파이썬 변환기(`hj/conv/{ctx,fonts_charpr,parapr,body,pack}.py`)의 1:1 이식이다.
//! 파일 분할도 그 모듈 경계를 그대로 따랐다 — 어느 규칙이 어디서 왔는지 대조할 수 있게 하려는 것이다.

use crate::error::HwpError;

mod body;
mod ctx;
mod fonts_charpr;
mod pack;
mod parapr;

/// 변환 산출물 — HWPX 패키지를 만들지 않고 파서에 바로 넣을 조각들.
#[derive(Debug, Default, Clone)]
pub struct HwpxParts {
    /// `Contents/header.xml` 전문
    pub header_xml: String,
    /// `Contents/section0.xml` 전문
    pub section_xml: String,
    /// 그림 바이너리 — (1-base 항목 번호, MIME, 원본 바이트)
    pub bins: Vec<(u16, String, Vec<u8>)>,
}

/// 클립보드 문서모델(JSON 문자열) → HWPX 조각.
///
/// 🔴 호출 순서에 의미가 있다. borderFill 이 먼저 1-base id 를 발급해야 글자모양·문단모양·본문의
/// `borderFillIDRef` 가 같은 번호를 가리키고, 글꼴표가 먼저 서야 `charPr/fontRef` 가 유효해진다.
/// 파이썬 정본 `pack.convert()` 의 순서를 그대로 지킨다.
pub fn hwpjson_to_hwpx_parts(json: &str) -> Result<HwpxParts, HwpError> {
    let value: serde_json::Value = serde_json::from_str(json.trim())
        .map_err(|e| HwpError::InvalidFile(format!("hwpjson 파싱 실패: {e}")))?;
    let model = ctx::Model::new(value)
        .ok_or_else(|| HwpError::InvalidFile("hwpjson 최상위가 객체가 아니다".to_string()))?;

    let mut ids = ctx::Ids::default();
    // ① 그림 먼저 — 본문의 binaryItemIDRef 가 이 번호를 그대로 쓴다.
    let bins = pack::bin_entries(&model, &mut ids);
    // ② 글꼴 수집 → 글꼴표
    fonts_charpr::collect_fonts(&model, &mut ids);
    let fontfaces = fonts_charpr::emit_fontfaces(&ids);
    // ③ 테두리/배경(1-base) → 글자모양 → 문단모양 (뒤 둘이 borderFill id 를 참조한다)
    let borderfills = parapr::emit_borderfills(&model, &mut ids);
    let charprops = fonts_charpr::emit_charpr(&model, &mut ids);
    let paraprops = parapr::emit_parapr(&model, &mut ids);
    // ④ header 조립(스타일 id 도 여기서 발급된다) → 본문
    let header_xml = pack::build_header(
        &model,
        &mut ids,
        &fontfaces,
        &borderfills,
        &charprops,
        &paraprops,
    );
    let section_xml = body::emit_section_file(&model, &mut ids);

    Ok(HwpxParts {
        header_xml,
        section_xml,
        bins,
    })
}
