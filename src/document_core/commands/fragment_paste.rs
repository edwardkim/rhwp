//! 외부 HWPX fragment paste — Stage 1: ID remap (charPr/paraPr/style/borderFill).
//!
//! 본 모듈은 byte-preserving HWPX 편집 원칙에 따라
//! lxml/quick-xml 직렬화 일체 사용 금지. 모든 변경은 raw `String`/byte slice에
//! 대한 `find` + `String::insert_str` / 직접 build로만 수행한다.
//!
//! 단위 테스트는 ID remap 로직을 격리 검증한다. paste 본체는 Stage 2.

use std::collections::HashMap;

/// 양식.hwpx 등 외부 HWPX의 raw header 정의 묶음. 각 필드는
/// `<hh:charPr id="N" ...>...</hh:charPr>` 형태의 1개 이상 정의가
/// 줄바꿈으로 이어붙인 raw XML 문자열이다.
#[derive(Debug, Clone, Default)]
pub struct SourceDefinitions {
    pub char_prs: String,
    pub para_prs: String,
    pub styles: String,
    pub border_fills: String,
}

/// source ID(외부 양식 기준) → target ID(현재 문서 기준) 매핑.
#[derive(Debug, Default)]
pub struct IdRemap {
    pub char_pr: HashMap<u32, u32>,
    pub para_pr: HashMap<u32, u32>,
    pub style: HashMap<u32, u32>,
    pub border_fill: HashMap<u32, u32>,
}

/// Stage 1/2 격리 에러 enum. Stage 4 wasm 통합 시 HwpError로 매핑한다.
#[derive(Debug, PartialEq, Eq)]
pub enum FragmentPasteError {
    /// source_definitions 안 정의의 id 속성이 없거나 파싱 불가.
    MalformedSourceDefinitions(String),
    /// header_xml에 동종 refList 닫는 태그가 없어 새 정의를 삽입할 위치가 없음.
    HeaderInsertionPointMissing(&'static str),
    /// fragment_xml 또는 paste 결과 section_xml이 well-formed가 아님.
    MalformedFragment(String),
}

impl std::fmt::Display for FragmentPasteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FragmentPasteError::MalformedSourceDefinitions(d) => {
                write!(f, "malformed source definitions: {d}")
            }
            FragmentPasteError::HeaderInsertionPointMissing(tag) => {
                write!(f, "header insertion point missing for {tag}")
            }
            FragmentPasteError::MalformedFragment(d) => {
                write!(f, "malformed fragment: {d}")
            }
        }
    }
}
impl std::error::Error for FragmentPasteError {}

// ───────────────────────────── Internal helpers ─────────────────────────────

/// `<hh:{tag} id="N" ...>...</hh:{tag}>` 또는 self-closing `<hh:{tag} id="N" .../>`
/// 형태의 정의 1건씩을 raw bytes로 떼어낸다. (id, raw_definition) 시퀀스 반환.
/// 정규식/quick-xml 사용 금지 — 직접 byte scan.
fn split_definitions<'a>(
    src: &'a str,
    tag: &str,
) -> Result<Vec<(u32, &'a str)>, FragmentPasteError> {
    let open_prefix = format!("<hh:{tag}");
    let close_tag = format!("</hh:{tag}>");
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let Some(rel) = src[pos..].find(&open_prefix) else {
            break;
        };
        let start = pos + rel;
        // open prefix 다음 문자가 ' ', '/', '>' 중 하나여야 정확히 매칭
        // (예: <hh:charPr 와 <hh:charProperty 구분)
        let after = bytes.get(start + open_prefix.len()).copied();
        if !matches!(
            after,
            Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n')
        ) {
            pos = start + open_prefix.len();
            continue;
        }
        // 여는 태그의 닫힘 위치
        let Some(open_close_rel) = src[start..].find('>') else {
            return Err(FragmentPasteError::MalformedSourceDefinitions(format!(
                "unterminated <hh:{tag} ...> at offset {start}"
            )));
        };
        let open_close = start + open_close_rel; // '>' 위치
        let self_closed = bytes.get(open_close.saturating_sub(1)).copied() == Some(b'/');

        // id 속성 추출
        let open_attrs = &src[start..=open_close];
        let id = parse_id_attr(open_attrs).ok_or_else(|| {
            FragmentPasteError::MalformedSourceDefinitions(format!(
                "missing id attr in <hh:{tag}> near offset {start}"
            ))
        })?;

        let end_excl = if self_closed {
            open_close + 1
        } else {
            // 닫는 태그까지 포함
            let Some(close_rel) = src[open_close..].find(&close_tag) else {
                return Err(FragmentPasteError::MalformedSourceDefinitions(format!(
                    "unterminated <hh:{tag} id=\"{id}\"> definition"
                )));
            };
            open_close + close_rel + close_tag.len()
        };

        out.push((id, &src[start..end_excl]));
        pos = end_excl;
    }
    Ok(out)
}

/// `<hh:tag id="N" ...>` 또는 `<hh:tag id='N' .../>` 의 N을 추출.
fn parse_id_attr(open_tag: &str) -> Option<u32> {
    for needle in [" id=\"", " id='"] {
        if let Some(idx) = open_tag.find(needle) {
            let after = &open_tag[idx + needle.len()..];
            let quote = needle.as_bytes().last().copied().unwrap();
            let end = after.find(quote as char)?;
            return after[..end].parse::<u32>().ok();
        }
    }
    None
}

/// 동일성 비교: id 속성을 제거한 나머지 raw bytes 문자열 반환.
fn def_body_normalized(def_xml: &str) -> String {
    let mut out = String::with_capacity(def_xml.len());
    for needle in [" id=\"", " id='"] {
        if let Some(idx) = def_xml.find(needle) {
            let quote = needle.as_bytes().last().copied().unwrap() as char;
            let after_start = idx + needle.len();
            if let Some(end_rel) = def_xml[after_start..].find(quote) {
                out.push_str(&def_xml[..idx]);
                out.push_str(&def_xml[after_start + end_rel + 1..]);
                return out;
            }
        }
    }
    def_xml.to_string()
}

/// 현재 header에서 사용 중인 최대 id를 찾는다.
fn max_existing_id(header_xml: &str, tag: &str) -> u32 {
    let open_prefix = format!("<hh:{tag}");
    let mut max = 0u32;
    let mut pos = 0usize;
    while let Some(rel) = header_xml[pos..].find(&open_prefix) {
        let start = pos + rel;
        let after = header_xml
            .as_bytes()
            .get(start + open_prefix.len())
            .copied();
        if !matches!(
            after,
            Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n')
        ) {
            pos = start + open_prefix.len();
            continue;
        }
        let Some(close_rel) = header_xml[start..].find('>') else {
            break;
        };
        let close = start + close_rel;
        if let Some(id) = parse_id_attr(&header_xml[start..=close]) {
            if id > max {
                max = id;
            }
        }
        pos = close + 1;
    }
    max
}

/// HWPX header.xml 의 ref list 닫는 태그 후보. 두 변형(`*List`, `*Properties`)을 모두 시도.
/// - 일부 양식(예: 양식.hwpx)은 `<hh:charProperties>` 컨테이너 사용
/// - 표준 샘플은 `<hh:charPropertyList>` 사용
fn ref_list_close_tag_candidates(item_tag: &str) -> &'static [&'static str] {
    match item_tag {
        "charPr" => &["</hh:charPropertyList>", "</hh:charProperties>"],
        "paraPr" => &["</hh:paraPropertyList>", "</hh:paraProperties>"],
        "style" => &["</hh:styleList>", "</hh:styles>"],
        "borderFill" => &["</hh:borderFillList>", "</hh:borderFills>"],
        _ => &["</hh:refList>"],
    }
}

/// 첫 번째 후보 닫는 태그 (대표 — 에러 메시지에 사용).
fn ref_list_close_tag(item_tag: &str) -> &'static str {
    ref_list_close_tag_candidates(item_tag)[0]
}

/// header_xml의 ref list 닫는 태그 직전에 `to_insert`를 삽입.
/// 후보 닫는 태그 중 가장 먼저 발견되는 것을 사용.
fn insert_into_ref_list(
    header_xml: &mut String,
    item_tag: &str,
    to_insert: &str,
) -> Result<(), FragmentPasteError> {
    if to_insert.is_empty() {
        return Ok(());
    }
    let candidates = ref_list_close_tag_candidates(item_tag);
    let mut found: Option<usize> = None;
    for cand in candidates {
        if let Some(p) = header_xml.rfind(cand) {
            found = Some(p);
            break;
        }
    }
    let Some(pos) = found else {
        return Err(FragmentPasteError::HeaderInsertionPointMissing(
            candidates[0],
        ));
    };
    header_xml.insert_str(pos, to_insert);
    Ok(())
}

/// `<hh:tag id="N"` 의 N을 새 ID로 갈아끼운 raw definition 반환.
fn rewrite_id_in_def(def_xml: &str, new_id: u32) -> String {
    for needle in [" id=\"", " id='"] {
        if let Some(idx) = def_xml.find(needle) {
            let quote = needle.as_bytes().last().copied().unwrap() as char;
            let after_start = idx + needle.len();
            if let Some(end_rel) = def_xml[after_start..].find(quote) {
                let mut out = String::with_capacity(def_xml.len() + 4);
                out.push_str(&def_xml[..after_start]);
                out.push_str(&new_id.to_string());
                out.push_str(&def_xml[after_start + end_rel..]);
                return out;
            }
        }
    }
    def_xml.to_string()
}

// ───────────────────────────── Public API ─────────────────────────────

/// header_xml에 source 정의들을 add-or-reuse 한 뒤 source_id → target_id 매핑을 반환.
/// header_xml은 in-place로 갱신된다(byte-preserving: 기존 정의 변경 없음, 닫는 태그
/// 직전에 새 정의 raw 삽입만).
pub fn build_id_remap(
    header_xml: &mut String,
    source: &SourceDefinitions,
) -> Result<IdRemap, FragmentPasteError> {
    let mut out = IdRemap::default();
    process_one_kind(header_xml, "charPr", &source.char_prs, &mut out.char_pr)?;
    process_one_kind(header_xml, "paraPr", &source.para_prs, &mut out.para_pr)?;
    process_one_kind(header_xml, "style", &source.styles, &mut out.style)?;
    process_one_kind(
        header_xml,
        "borderFill",
        &source.border_fills,
        &mut out.border_fill,
    )?;
    Ok(out)
}

fn process_one_kind(
    header_xml: &mut String,
    item_tag: &str,
    source_xml: &str,
    remap: &mut HashMap<u32, u32>,
) -> Result<(), FragmentPasteError> {
    if source_xml.trim().is_empty() {
        return Ok(());
    }
    let source_defs = split_definitions(source_xml, item_tag)?;
    let existing_defs = split_definitions(header_xml, item_tag)?;

    // 기존 본문(id 제거 후)을 키로 하는 lookup — 동일 정의면 기존 ID 재사용.
    let mut body_to_id: HashMap<String, u32> = HashMap::with_capacity(existing_defs.len());
    for (eid, raw) in &existing_defs {
        body_to_id.entry(def_body_normalized(raw)).or_insert(*eid);
    }

    let mut next_id = max_existing_id(header_xml, item_tag) + 1;
    let mut to_append = String::new();

    for (sid, raw) in &source_defs {
        let body = def_body_normalized(raw);
        if let Some(&target) = body_to_id.get(&body) {
            remap.insert(*sid, target);
        } else {
            let target = next_id;
            next_id += 1;
            let rewritten = rewrite_id_in_def(raw, target);
            to_append.push('\n');
            to_append.push_str(&rewritten);
            body_to_id.insert(body, target);
            remap.insert(*sid, target);
        }
    }

    insert_into_ref_list(header_xml, item_tag, &to_append)?;
    Ok(())
}

/// fragment_xml의 *IDRef 속성값을 remap 결과로 갈아끼운다.
/// 매칭은 `charPrIDRef="N"` / `charPrIDRef='N'` 양쪽 모두 지원.
pub fn rewrite_id_refs(fragment_xml: &str, remap: &IdRemap) -> String {
    let mut out = fragment_xml.to_string();
    out = rewrite_one_attr(out, "charPrIDRef", &remap.char_pr);
    out = rewrite_one_attr(out, "paraPrIDRef", &remap.para_pr);
    out = rewrite_one_attr(out, "styleIDRef", &remap.style);
    out = rewrite_one_attr(out, "borderFillIDRef", &remap.border_fill);
    out
}

fn rewrite_one_attr(input: String, attr: &str, remap: &HashMap<u32, u32>) -> String {
    if remap.is_empty() {
        return input;
    }
    let dq_needle = format!("{attr}=\"");
    let sq_needle = format!("{attr}='");
    let mut out = String::with_capacity(input.len());
    let mut pos = 0usize;
    while pos < input.len() {
        let dq = input[pos..].find(&dq_needle);
        let sq = input[pos..].find(&sq_needle);
        let next = match (dq, sq) {
            (None, None) => None,
            (Some(a), None) => Some((a, '"')),
            (None, Some(a)) => Some((a, '\'')),
            (Some(a), Some(b)) => Some(if a <= b { (a, '"') } else { (b, '\'') }),
        };
        let Some((rel, quote)) = next else {
            out.push_str(&input[pos..]);
            break;
        };
        let attr_start = pos + rel;
        // attr 시작 직전 문자가 attribute boundary인지 확인 (다른 속성 prefix 충돌 방지)
        let prev = if attr_start == 0 {
            b'\0'
        } else {
            input.as_bytes()[attr_start - 1]
        };
        if !matches!(prev, b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'/') {
            // boundary 아님 — 이 위치까지 그대로 복사하고 attr_start+1부터 다시 시작
            out.push_str(&input[pos..=attr_start]);
            pos = attr_start + 1;
            continue;
        }
        let value_start = attr_start + attr.len() + 2; // attr + '=' + quote
        let Some(value_end_rel) = input[value_start..].find(quote) else {
            out.push_str(&input[pos..]);
            break;
        };
        let value_end = value_start + value_end_rel;
        let raw_id = &input[value_start..value_end];
        out.push_str(&input[pos..value_start]);
        match raw_id
            .parse::<u32>()
            .ok()
            .and_then(|sid| remap.get(&sid).copied())
        {
            Some(target) => out.push_str(&target.to_string()),
            None => out.push_str(raw_id),
        }
        out.push(quote);
        pos = value_end + 1;
    }
    out
}

// ───────────────────────────── Stage 2: paragraphs paste ─────────────────────────────

/// Stage 2 paste 결과. Document 모델 갱신은 Stage 4에서.
#[derive(Debug, Default)]
pub struct ParagraphPasteResult {
    pub id_remap: IdRemap,
    pub inserted_para_count: usize,
    pub new_section_xml: String,
}

/// section_xml의 top-level `<hp:p>` 시퀀스 중 `after_para_idx` 직후에 fragment를 byte-preserving 삽입.
/// header_xml은 in-place로 갱신된다(ID remap에 따른 새 정의 추가).
///
/// fragment_xml은 1개 이상의 `<hp:p ...>...</hp:p>` 또는 self-closing `<hp:p/>` 시퀀스여야 한다.
/// 표(`<hp:tbl>`)가 포함된 fragment는 Stage 3에서 처리.
pub fn paste_paragraphs_into_section(
    section_xml: &str,
    header_xml: &mut String,
    after_para_idx: usize,
    fragment_xml: &str,
    source: &SourceDefinitions,
) -> Result<ParagraphPasteResult, FragmentPasteError> {
    // 1. ID remap 사전 단계 (Stage 1 함수 재사용)
    let remap = build_id_remap(header_xml, source)?;
    let fragment_remapped = rewrite_id_refs(fragment_xml, &remap);

    // 2. 입력 fragment well-formedness 사전 검증 (read-only, 직렬화 금지)
    validate_paragraphs_wellformed(&fragment_remapped)?;

    // 3. fragment의 top-level <hp:p> 개수 — 단위 테스트 검증용
    let fragment_p_count = count_top_level_p(&fragment_remapped);
    if fragment_p_count == 0 {
        return Err(FragmentPasteError::MalformedFragment(
            "no top-level <hp:p> found in fragment".into(),
        ));
    }

    // 4. anchor: section_xml의 top-level <hp:p> 시퀀스에서 after_para_idx 위치를 찾음
    let p_spans = find_top_level_p_spans(section_xml);
    if after_para_idx >= p_spans.len() {
        return Err(FragmentPasteError::MalformedFragment(format!(
            "after_para_idx {after_para_idx} out of range (section has {} top-level paragraphs)",
            p_spans.len()
        )));
    }
    let (_a_start, a_end) = p_spans[after_para_idx];

    // 5. byte-preserving 삽입: anchor paragraph 닫는 태그 직후에 fragment 삽입
    let mut new_section = String::with_capacity(section_xml.len() + fragment_remapped.len());
    new_section.push_str(&section_xml[..a_end]);
    new_section.push_str(&fragment_remapped);
    new_section.push_str(&section_xml[a_end..]);

    // 6. 출력 well-formedness 검증 (read-only)
    validate_section_wellformed(&new_section)?;

    Ok(ParagraphPasteResult {
        id_remap: remap,
        inserted_para_count: fragment_p_count,
        new_section_xml: new_section,
    })
}

/// section_xml의 top-level `<hp:p>` 시퀀스를 (start, end) byte offset으로 반환.
/// `<hp:tbl>`/`<hp:subList>` 안의 nested paragraph는 제외.
fn find_top_level_p_spans(section_xml: &str) -> Vec<(usize, usize)> {
    let bytes = section_xml.as_bytes();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut depth: i32 = 0;
    let mut cur_start: Option<usize> = None;
    let mut pos = 0usize;
    while pos < bytes.len() {
        // CDATA / comment 안전 스킵.
        if section_xml[pos..].starts_with("<![CDATA[") {
            pos = section_xml[pos..]
                .find("]]>")
                .map(|r| pos + r + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if section_xml[pos..].starts_with("<!--") {
            pos = section_xml[pos..]
                .find("-->")
                .map(|r| pos + r + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if section_xml[pos..].starts_with("<hp:p/>") {
            if depth == 0 {
                spans.push((pos, pos + 7));
            }
            pos += 7;
            continue;
        }
        if section_xml[pos..].starts_with("<hp:p ") {
            if depth == 0 && cur_start.is_none() {
                cur_start = Some(pos);
            }
            depth += 1;
            pos += 6;
            continue;
        }
        if section_xml[pos..].starts_with("</hp:p>") {
            depth -= 1;
            pos += 7;
            if depth == 0 {
                if let Some(s) = cur_start.take() {
                    spans.push((s, pos));
                }
            }
            continue;
        }
        if section_xml[pos..].starts_with("<hp:subList") {
            depth += 1;
            pos += "<hp:subList".len();
            continue;
        }
        if section_xml[pos..].starts_with("</hp:subList>") {
            depth -= 1;
            pos += "</hp:subList>".len();
            continue;
        }
        // 다음 '<' 까지 빠르게 점프
        let nxt = section_xml[pos + 1..].find('<').map(|r| pos + 1 + r);
        pos = nxt.unwrap_or(bytes.len());
    }
    spans
}

fn count_top_level_p(fragment_xml: &str) -> usize {
    find_top_level_p_spans(fragment_xml).len()
}

/// fragment에 1개 이상의 top-level `<hp:p>` 가 있고 모두 닫혀 있으면 OK.
/// quick-xml 등 직렬화 라이브러리 사용 금지. 직접 byte scan.
fn validate_paragraphs_wellformed(fragment_xml: &str) -> Result<(), FragmentPasteError> {
    let spans = find_top_level_p_spans(fragment_xml);
    if spans.is_empty() {
        return Err(FragmentPasteError::MalformedFragment(
            "no top-level <hp:p> in fragment".into(),
        ));
    }
    // 마지막 span end 이후에 닫히지 않은 <hp:p 가 있으면 안 됨
    let last_end = spans.last().unwrap().1;
    let tail = &fragment_xml[last_end..];
    if tail.contains("<hp:p ") || tail.contains("<hp:p>") {
        return Err(FragmentPasteError::MalformedFragment(
            "unclosed <hp:p> in fragment tail".into(),
        ));
    }
    Ok(())
}

/// section 결과에 대해 lightweight well-formedness 검증.
/// top-level `<hp:p>` depth가 paste 전후로 양수→0 으로 일관되는지 확인.
fn validate_section_wellformed(section_xml: &str) -> Result<(), FragmentPasteError> {
    // find_top_level_p_spans는 depth 0으로 끝나면 정상 종료.
    // 끝까지 못 닫힌 경우 cur_start.take() 호출이 안 되어 spans에 안 들어감 — 그 경우를 별도 검출.
    let bytes = section_xml.as_bytes();
    let mut depth: i32 = 0;
    let mut pos = 0usize;
    while pos < bytes.len() {
        if section_xml[pos..].starts_with("<![CDATA[") {
            pos = section_xml[pos..]
                .find("]]>")
                .map(|r| pos + r + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if section_xml[pos..].starts_with("<!--") {
            pos = section_xml[pos..]
                .find("-->")
                .map(|r| pos + r + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if section_xml[pos..].starts_with("<hp:p/>") {
            pos += 7;
            continue;
        }
        if section_xml[pos..].starts_with("<hp:p ") {
            depth += 1;
            pos += 6;
            continue;
        }
        if section_xml[pos..].starts_with("</hp:p>") {
            depth -= 1;
            if depth < 0 {
                return Err(FragmentPasteError::MalformedFragment(
                    "unbalanced </hp:p> in section".into(),
                ));
            }
            pos += 7;
            continue;
        }
        if section_xml[pos..].starts_with("<hp:subList") {
            depth += 1;
            pos += "<hp:subList".len();
            continue;
        }
        if section_xml[pos..].starts_with("</hp:subList>") {
            depth -= 1;
            if depth < 0 {
                return Err(FragmentPasteError::MalformedFragment(
                    "unbalanced </hp:subList> in section".into(),
                ));
            }
            pos += "</hp:subList>".len();
            continue;
        }
        let nxt = section_xml[pos + 1..].find('<').map(|r| pos + 1 + r);
        pos = nxt.unwrap_or(bytes.len());
    }
    if depth != 0 {
        return Err(FragmentPasteError::MalformedFragment(format!(
            "section ended with unbalanced <hp:p> depth={depth}"
        )));
    }
    Ok(())
}

// ───────────────────────────── Stage 3: table addrs + table-aware paste ─────────────────────────────

/// `<hp:tbl>` 안 모든 행/셀의 `rowCnt`/`rowAddr`/`colAddr` 를 colSpan + rowSpan 그리드 기준으로
/// 재계산해 byte-preserving으로 갱신한 fragment_xml 반환.
/// HWPX table coordinate invariants를 직접 보정한다.
pub fn recompute_table_addrs(fragment_xml: &str) -> Result<String, FragmentPasteError> {
    let mut out = String::with_capacity(fragment_xml.len());
    let mut pos = 0usize;
    while let Some(rel) = fragment_xml[pos..].find("<hp:tbl") {
        let tbl_start = pos + rel;
        // 다음 문자 boundary (공백/'/'/'>')
        let after = fragment_xml.as_bytes().get(tbl_start + 7).copied();
        if !matches!(
            after,
            Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n')
        ) {
            out.push_str(&fragment_xml[pos..tbl_start + 7]);
            pos = tbl_start + 7;
            continue;
        }
        // 닫는 </hp:tbl> 까지 (depth tracking — nested tables)
        let tbl_end = find_balanced_close(fragment_xml, tbl_start, "<hp:tbl", "</hp:tbl>")
            .ok_or_else(|| {
                FragmentPasteError::MalformedFragment(format!(
                    "unbalanced <hp:tbl> at offset {tbl_start}"
                ))
            })?;
        out.push_str(&fragment_xml[pos..tbl_start]);
        let tbl_slice = &fragment_xml[tbl_start..tbl_end];
        let recomputed = recompute_one_table(tbl_slice)?;
        out.push_str(&recomputed);
        pos = tbl_end;
    }
    out.push_str(&fragment_xml[pos..]);
    Ok(out)
}

/// 단일 `<hp:tbl ...>...</hp:tbl>` slice의 메타데이터 재계산.
fn recompute_one_table(tbl_slice: &str) -> Result<String, FragmentPasteError> {
    // 1. row spans 추출 — 각 row에 (col_span, row_span)의 셀들
    let rows = parse_rows(tbl_slice)?;

    // 2. row-by-row pass — 같은 row 안 cells는 logical_col 변수로 진행,
    //    rowSpan>1로 다른 row를 점유하는 경우만 occupied에 마킹.
    let mut occupied: Vec<std::collections::HashSet<u32>> =
        vec![std::collections::HashSet::new(); rows.len()];
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    for (r, row) in rows.iter().enumerate() {
        let mut logical_col: u32 = 0;
        for cell in &row.cells {
            while occupied[r].contains(&logical_col) {
                logical_col += 1;
            }
            // colAddr / rowAddr 속성 갱신
            for &(attr_name, new_value) in &[("rowAddr", r as u32), ("colAddr", logical_col)] {
                if let Some((vs, ve)) = find_attr_value_span(
                    tbl_slice,
                    cell.open_tag_start,
                    cell.open_tag_end,
                    attr_name,
                ) {
                    replacements.push((vs, ve, new_value.to_string()));
                }
            }
            // rowSpan>1 만 occupied에 마킹 (자기 row는 logical_col 변수로 처리)
            for span_r in 1..cell.row_span {
                let target_row = r + span_r as usize;
                if target_row >= rows.len() {
                    break;
                }
                for span_c in 0..cell.col_span {
                    occupied[target_row].insert(logical_col + span_c);
                }
            }
            logical_col += cell.col_span;
        }
    }

    // 3b. 표의 rowCnt — opening <hp:tbl ...> 태그 안에서 갱신
    let tbl_open_end = tbl_slice.find('>').ok_or_else(|| {
        FragmentPasteError::MalformedFragment("table open tag missing '>'".into())
    })?;
    if let Some((vs, ve)) = find_attr_value_span(tbl_slice, 0, tbl_open_end + 1, "rowCnt") {
        replacements.push((vs, ve, rows.len().to_string()));
    }

    // 4. 역순 적용 (offset 보존)
    replacements.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = tbl_slice.to_string();
    for (start, end, new_val) in replacements {
        out.replace_range(start..end, &new_val);
    }

    // 5. nested table 재귀 처리: outer open tag와 close tag 사이 본문에 대해 recompute_table_addrs 재귀
    if let (Some(open_end_rel), Some(close_start_rel)) = (out.find('>'), out.rfind("</hp:tbl>")) {
        let open_end = open_end_rel + 1;
        if open_end < close_start_rel {
            let inner = &out[open_end..close_start_rel];
            let inner_recomputed = recompute_table_addrs(inner)?;
            let mut final_out = String::with_capacity(out.len());
            final_out.push_str(&out[..open_end]);
            final_out.push_str(&inner_recomputed);
            final_out.push_str(&out[close_start_rel..]);
            return Ok(final_out);
        }
    }
    Ok(out)
}

/// 단순 row/cell 추상화 (메타데이터만)
#[derive(Debug)]
struct CellInfo {
    open_tag_start: usize, // <hp:tc ...> 의 '<' 위치 (tbl_slice 내부 offset)
    open_tag_end: usize,   // open tag 의 '>' 직후 위치
    col_span: u32,
    row_span: u32,
}

#[derive(Debug)]
struct RowInfo {
    cells: Vec<CellInfo>,
}

/// `<hp:tbl ...>` slice 안의 직접 자식 row들과 각 row의 직접 자식 cell들을 추출.
/// nested 표는 무시 (마지막 `</hp:subList>` 뒤 메타데이터만 검색).
fn parse_rows(tbl_slice: &str) -> Result<Vec<RowInfo>, FragmentPasteError> {
    let mut rows = Vec::new();
    // 직접 자식 <hp:tr> 만 찾는다 — depth=1 (table 직속)
    let row_spans = find_direct_children(tbl_slice, "<hp:tbl", "</hp:tbl>", "<hp:tr", "</hp:tr>")?;
    for (rs, re) in row_spans {
        let row_slice = &tbl_slice[rs..re];
        let cell_spans =
            find_direct_children(row_slice, "<hp:tr", "</hp:tr>", "<hp:tc", "</hp:tc>")?;
        let mut cells = Vec::new();
        for (cs, ce) in cell_spans {
            let cell_slice = &row_slice[cs..ce];
            let open_end_rel = cell_slice.find('>').ok_or_else(|| {
                FragmentPasteError::MalformedFragment("cell open tag missing '>'".into())
            })?;
            let col_span = extract_span_attr(cell_slice, "colSpan").unwrap_or(1);
            let row_span = extract_span_attr(cell_slice, "rowSpan").unwrap_or(1);
            cells.push(CellInfo {
                open_tag_start: rs + cs,
                open_tag_end: rs + cs + open_end_rel + 1,
                col_span,
                row_span,
            });
        }
        rows.push(RowInfo { cells });
    }
    Ok(rows)
}

/// `outer_open` ... `outer_close` 안에서 `child_open`/`child_close` 인 직접 자식의 (start, end) 시퀀스 반환.
/// `outer_open`은 입력 slice의 시작이 `<outer_open` 으로 시작한다고 가정.
/// nested 표 안전을 위해 depth 추적.
fn find_direct_children(
    slice: &str,
    outer_open: &str,
    outer_close: &str,
    child_open: &str,
    child_close: &str,
) -> Result<Vec<(usize, usize)>, FragmentPasteError> {
    let mut spans = Vec::new();
    let mut pos = 0usize;
    let mut depth: i32 = 0;
    let mut current_child_start: Option<usize> = None;
    while pos < slice.len() {
        if slice[pos..].starts_with(outer_open) {
            // outer 자체는 depth 변화 없이 그냥 지나감 (slice 시작에서만 등장 가정)
            let after = slice.as_bytes().get(pos + outer_open.len()).copied();
            if matches!(
                after,
                Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n')
            ) {
                pos += outer_open.len();
                continue;
            }
        }
        if slice[pos..].starts_with(outer_close) {
            pos += outer_close.len();
            continue;
        }
        if slice[pos..].starts_with(child_open) {
            let after = slice.as_bytes().get(pos + child_open.len()).copied();
            if matches!(
                after,
                Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n')
            ) {
                if depth == 0 && current_child_start.is_none() {
                    current_child_start = Some(pos);
                }
                depth += 1;
                pos += child_open.len();
                continue;
            }
        }
        if slice[pos..].starts_with(child_close) {
            depth -= 1;
            pos += child_close.len();
            if depth == 0 {
                if let Some(s) = current_child_start.take() {
                    spans.push((s, pos));
                }
            } else if depth < 0 {
                return Err(FragmentPasteError::MalformedFragment(format!(
                    "unbalanced {child_close}"
                )));
            }
            continue;
        }
        let nxt = slice[pos + 1..].find('<').map(|r| pos + 1 + r);
        pos = nxt.unwrap_or(slice.len());
    }
    Ok(spans)
}

/// cell slice의 open tag 안에서 colSpan/rowSpan 속성 값을 추출. 없으면 None.
/// 마지막 `</hp:subList>` 뒤 부분만 검색해 nested table 안 cellSpan과 혼동 방지.
fn extract_span_attr(cell_slice: &str, attr: &str) -> Option<u32> {
    // open tag만 본다: '<' 부터 첫 '>' 까지
    let open_end = cell_slice.find('>')?;
    let open_tag = &cell_slice[..=open_end];
    for q in [&format!(" {attr}=\""), &format!(" {attr}='")] {
        if let Some(idx) = open_tag.find(q.as_str()) {
            let quote = q.as_bytes().last().copied().unwrap() as char;
            let val_start = idx + q.len();
            if let Some(end_rel) = open_tag[val_start..].find(quote) {
                return open_tag[val_start..val_start + end_rel].parse().ok();
            }
        }
    }
    None
}

/// open tag 내부에서 attribute 값의 byte span (인용부호 안쪽) 반환.
/// `tag_start` ~ `tag_end` 는 open tag '<' ... '>' 의 byte range (tag_end는 '>' 다음 위치).
fn find_attr_value_span(
    src: &str,
    tag_start: usize,
    tag_end: usize,
    attr: &str,
) -> Option<(usize, usize)> {
    let region = &src[tag_start..tag_end];
    for q in [&format!(" {attr}=\""), &format!(" {attr}='")] {
        if let Some(idx) = region.find(q.as_str()) {
            let quote = q.as_bytes().last().copied().unwrap() as char;
            let val_start = tag_start + idx + q.len();
            let after = &src[val_start..tag_end];
            if let Some(end_rel) = after.find(quote) {
                return Some((val_start, val_start + end_rel));
            }
        }
    }
    None
}

/// `outer_open`/`outer_close`로 둘러싸인 nested 구조에서 outer의 닫는 위치를 찾음.
fn find_balanced_close(
    src: &str,
    start: usize,
    open_prefix: &str,
    close_tag: &str,
) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut pos = start;
    while pos < src.len() {
        if src[pos..].starts_with(open_prefix) {
            let after = src.as_bytes().get(pos + open_prefix.len()).copied();
            if matches!(
                after,
                Some(b' ') | Some(b'/') | Some(b'>') | Some(b'\t') | Some(b'\n')
            ) {
                depth += 1;
                pos += open_prefix.len();
                continue;
            }
        }
        if src[pos..].starts_with(close_tag) {
            depth -= 1;
            pos += close_tag.len();
            if depth == 0 {
                return Some(pos);
            }
            continue;
        }
        let nxt = src[pos + 1..].find('<').map(|r| pos + 1 + r);
        pos = nxt.unwrap_or(src.len());
    }
    None
}

/// table-aware fragment paste — fragment에 `<hp:tbl>` 포함 시 자동으로 rowCnt/rowAddr/colAddr 보정.
/// paragraphs-only fragment는 paste_paragraphs_into_section과 동일 동작.
pub fn paste_fragment_into_section(
    section_xml: &str,
    header_xml: &mut String,
    after_para_idx: usize,
    fragment_xml: &str,
    source: &SourceDefinitions,
) -> Result<ParagraphPasteResult, FragmentPasteError> {
    let has_table = fragment_xml.contains("<hp:tbl");
    if !has_table {
        return paste_paragraphs_into_section(
            section_xml,
            header_xml,
            after_para_idx,
            fragment_xml,
            source,
        );
    }
    // table 포함 — recompute_table_addrs를 거친 뒤 paste
    let recomputed = recompute_table_addrs(fragment_xml)?;
    paste_paragraphs_into_section(section_xml, header_xml, after_para_idx, &recomputed, source)
}

// ───────────────────────────── Stage 3: hwp_open_test (회귀 검증용) ─────────────────────────────

/// 한컴 hwp 바이너리로 hwpx 파일을 열어 정상 로드 여부 판정.
/// timeout 안에 GUI loop에 들어가면 SIGTERM(143) 또는 timeout(124) → Accepted.
/// 즉시 자발 종료(0 또는 다른 코드)면 Rejected.
/// hwp 바이너리/디스플레이 가용 안 하면 Skipped.
#[derive(Debug, PartialEq, Eq)]
pub enum HwpOpenResult {
    Accepted,
    Rejected(i32),
    Skipped(String),
}

#[cfg(all(test, not(target_arch = "wasm32")))]
fn hwp_open_test(path: &std::path::Path, timeout_sec: u32) -> HwpOpenResult {
    use std::process::Command;
    let bin = std::path::Path::new("/opt/hnc/hoffice11/Bin/hwp");
    if !bin.is_file() {
        return HwpOpenResult::Skipped(format!("hwp binary not found at {}", bin.display()));
    }
    if !path.is_file() {
        return HwpOpenResult::Skipped(format!("input not found: {}", path.display()));
    }
    let out = Command::new("timeout")
        .arg(timeout_sec.to_string())
        .arg("env")
        .arg("DISPLAY=:0")
        .arg(bin)
        .arg(path)
        .output();
    let Ok(out) = out else {
        return HwpOpenResult::Skipped("failed to spawn timeout/hwp".into());
    };
    match out.status.code() {
        Some(143) | Some(124) => HwpOpenResult::Accepted,
        Some(c) => HwpOpenResult::Rejected(c),
        None => HwpOpenResult::Skipped("no exit code".into()),
    }
}

// ───────────────────────────── Tests ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with(char_prs: &str, para_prs: &str, styles: &str, border_fills: &str) -> String {
        format!(
            "<hh:head>\
<hh:charPropertyList>{char_prs}</hh:charPropertyList>\
<hh:paraPropertyList>{para_prs}</hh:paraPropertyList>\
<hh:styleList>{styles}</hh:styleList>\
<hh:borderFillList>{border_fills}</hh:borderFillList>\
</hh:head>"
        )
    }

    #[test]
    fn id_remap_existing_definition_reuses_id() {
        let mut header = header_with(
            "<hh:charPr id=\"5\" height=\"1000\"><hh:fontRef hangul=\"0\"/></hh:charPr>",
            "",
            "",
            "",
        );
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"9\" height=\"1000\"><hh:fontRef hangul=\"0\"/></hh:charPr>"
                .into(),
            ..Default::default()
        };
        let remap = build_id_remap(&mut header, &source).unwrap();
        // source id=9 가 기존 동일 정의(id=5)를 재사용해야 함
        assert_eq!(remap.char_pr.get(&9), Some(&5));
        // header에 새 정의 append되지 않았어야 함 (기존 1건만)
        assert_eq!(header.matches("<hh:charPr id=").count(), 1);
    }

    #[test]
    fn id_remap_new_definition_appends() {
        let mut header = header_with("<hh:charPr id=\"5\" height=\"1000\"/>", "", "", "");
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"9\" height=\"2000\"/>".into(),
            ..Default::default()
        };
        let original_len = header.len();
        let remap = build_id_remap(&mut header, &source).unwrap();
        // 다른 정의이므로 새 ID = max(5)+1 = 6
        assert_eq!(remap.char_pr.get(&9), Some(&6));
        // header에 새 정의가 추가됐고, 닫는 태그 위치가 보존됨
        assert!(header.len() > original_len);
        assert!(header.contains("<hh:charPr id=\"6\" height=\"2000\"/>"));
        assert!(header.ends_with("</hh:head>"));
        assert_eq!(header.matches("<hh:charPr id=").count(), 2);
    }

    #[test]
    fn id_remap_byte_exact_insertion() {
        let mut header = header_with("<hh:charPr id=\"5\" height=\"1000\"/>", "", "", "");
        let baseline = header.clone();
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"9\" height=\"2000\"/>".into(),
            ..Default::default()
        };
        build_id_remap(&mut header, &source).unwrap();
        // 기존 정의는 그대로 byte-exact 유지되어야 함
        assert!(header.contains("<hh:charPr id=\"5\" height=\"1000\"/>"));
        // baseline의 charPropertyList 닫는 태그 직전까지가 byte-exact 보존
        let baseline_close_pos = baseline.find("</hh:charPropertyList>").unwrap();
        assert_eq!(
            &header[..baseline_close_pos],
            &baseline[..baseline_close_pos]
        );
    }

    #[test]
    fn rewrite_id_refs_preserves_unrelated_attrs() {
        let mut remap = IdRemap::default();
        remap.char_pr.insert(9, 27);
        let frag = r#"<hp:run charPrIDRef="9" otherAttr="keep"><hp:t>x</hp:t></hp:run>"#;
        let out = rewrite_id_refs(frag, &remap);
        assert_eq!(
            out,
            r#"<hp:run charPrIDRef="27" otherAttr="keep"><hp:t>x</hp:t></hp:run>"#
        );
    }

    #[test]
    fn rewrite_id_refs_handles_quote_styles() {
        let mut remap = IdRemap::default();
        remap.char_pr.insert(9, 27);
        remap.para_pr.insert(3, 11);
        let frag = r#"<hp:p paraPrIDRef='3'><hp:run charPrIDRef="9"/></hp:p>"#;
        let out = rewrite_id_refs(frag, &remap);
        assert!(out.contains("paraPrIDRef='11'"));
        assert!(out.contains("charPrIDRef=\"27\""));
    }

    #[test]
    fn rewrite_id_refs_unknown_ids_left_intact() {
        let mut remap = IdRemap::default();
        remap.char_pr.insert(9, 27);
        // remap에 없는 12는 그대로 유지
        let frag = r#"<hp:run charPrIDRef="12"/>"#;
        let out = rewrite_id_refs(frag, &remap);
        assert_eq!(out, r#"<hp:run charPrIDRef="12"/>"#);
    }

    #[test]
    fn rewrite_id_refs_no_match_for_attr_prefix_collision() {
        // "extraCharPrIDRef"는 본 attr가 아니므로 변경되면 안 됨
        let mut remap = IdRemap::default();
        remap.char_pr.insert(9, 27);
        let frag = r#"<hp:run extraCharPrIDRef="9"/>"#;
        let out = rewrite_id_refs(frag, &remap);
        assert_eq!(out, frag);
    }

    // ─── Stage 2: paragraphs paste ───

    fn empty_section() -> String {
        // top-level <hp:p> 1개 (anchor용). secPr 같은 부수 요소는 단순화.
        "<hs:sec><hp:p paraPrIDRef=\"0\" styleIDRef=\"0\"><hp:run charPrIDRef=\"0\"><hp:t>x</hp:t></hp:run></hp:p></hs:sec>".to_string()
    }

    fn empty_header() -> String {
        header_with(
            "<hh:charPr id=\"0\" height=\"1000\"/>",
            "<hh:paraPr id=\"0\"/>",
            "<hh:style id=\"0\" name=\"def\"/>",
            "",
        )
    }

    #[test]
    fn paste_paragraphs_simple() {
        let section = empty_section();
        let mut header = empty_header();
        let fragment = r#"<hp:p paraPrIDRef="9" styleIDRef="0"><hp:run charPrIDRef="9"><hp:t>hello</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"9\" height=\"2000\"/>".into(),
            para_prs: "<hh:paraPr id=\"9\" alignTag=\"new\"/>".into(),
            ..Default::default()
        };
        let result =
            paste_paragraphs_into_section(&section, &mut header, 0, fragment, &source).unwrap();
        assert_eq!(result.inserted_para_count, 1);
        // 결과 section의 top-level <hp:p>가 paste 전(1)에서 +1 = 2
        let final_p_count = find_top_level_p_spans(&result.new_section_xml).len();
        assert_eq!(final_p_count, 2);
        // remap이 적용되어 fragment의 charPrIDRef="9"가 새 ID로 갈아끼워짐
        assert!(!result.new_section_xml.contains("charPrIDRef=\"9\""));
        assert!(result.new_section_xml.contains("hello"));
    }

    #[test]
    fn paste_paragraphs_multi() {
        let section = empty_section();
        let mut header = empty_header();
        let fragment = r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>1</hp:t></hp:run></hp:p><hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>2</hp:t></hp:run></hp:p><hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>3</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();
        let result =
            paste_paragraphs_into_section(&section, &mut header, 0, fragment, &source).unwrap();
        assert_eq!(result.inserted_para_count, 3);
        let final_p_count = find_top_level_p_spans(&result.new_section_xml).len();
        assert_eq!(final_p_count, 4);
    }

    #[test]
    fn paste_paragraphs_byte_preserving() {
        let section = empty_section();
        let baseline = section.clone();
        let mut header = empty_header();
        let fragment =
            r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();
        let result =
            paste_paragraphs_into_section(&section, &mut header, 0, fragment, &source).unwrap();
        // 기존 section의 첫 paragraph가 byte-exact 보존: anchor end까지가 baseline의 anchor end까지와 동일
        let baseline_first_p_end = find_top_level_p_spans(&baseline)[0].1;
        assert_eq!(
            &result.new_section_xml[..baseline_first_p_end],
            &baseline[..baseline_first_p_end]
        );
        // 닫는 </hs:sec>도 보존
        assert!(result.new_section_xml.ends_with("</hs:sec>"));
    }

    #[test]
    fn paste_paragraphs_id_remapped_into_header() {
        let section = empty_section();
        let mut header = empty_header();
        // source의 charPr id=99는 본문이 다른 신규 정의 → 새 ID 부여 + header에 추가됨
        let fragment =
            r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="99"><hp:t>x</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"99\" height=\"5555\"/>".into(),
            ..Default::default()
        };
        let result =
            paste_paragraphs_into_section(&section, &mut header, 0, fragment, &source).unwrap();
        // header에 height=5555 새 정의가 추가됨
        assert!(header.contains("height=\"5555\""));
        // remap은 99 → 1 (max 0 + 1) 또는 그 이상의 새 ID
        let new_id = result.id_remap.char_pr.get(&99).copied().unwrap();
        assert!(new_id > 0);
        // fragment 안 charPrIDRef="99"가 새 ID로 갈아끼워졌어야 함
        assert!(!result.new_section_xml.contains("charPrIDRef=\"99\""));
        assert!(result
            .new_section_xml
            .contains(&format!("charPrIDRef=\"{new_id}\"")));
    }

    #[test]
    fn paste_paragraphs_validates_input_unclosed_p() {
        let section = empty_section();
        let mut header = empty_header();
        // 닫히지 않은 <hp:p>
        let fragment = r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run>"#;
        let source = SourceDefinitions::default();
        let err =
            paste_paragraphs_into_section(&section, &mut header, 0, fragment, &source).unwrap_err();
        assert!(matches!(err, FragmentPasteError::MalformedFragment(_)));
    }

    #[test]
    fn paste_paragraphs_validates_input_no_p() {
        let section = empty_section();
        let mut header = empty_header();
        // top-level <hp:p>가 전혀 없는 fragment
        let fragment = r#"<hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run>"#;
        let source = SourceDefinitions::default();
        let err =
            paste_paragraphs_into_section(&section, &mut header, 0, fragment, &source).unwrap_err();
        assert!(matches!(err, FragmentPasteError::MalformedFragment(_)));
    }

    #[test]
    fn paste_paragraphs_after_para_idx_out_of_range() {
        let section = empty_section();
        let mut header = empty_header();
        let fragment =
            r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();
        let err =
            paste_paragraphs_into_section(&section, &mut header, 5, fragment, &source).unwrap_err();
        assert!(matches!(err, FragmentPasteError::MalformedFragment(_)));
    }

    // ─── Stage 3: table addrs ───

    fn make_simple_table(rows: u32, cols: u32) -> String {
        let mut out = format!("<hp:tbl rowCnt=\"0\" colCnt=\"{cols}\"><hp:cellzoneList/>");
        for r in 0..rows {
            out.push_str("<hp:tr>");
            for c in 0..cols {
                let _ = c; // placeholder
                out.push_str(&format!(
                    "<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p><hp:run charPrIDRef=\"0\"><hp:t>x</hp:t></hp:run></hp:p></hp:subList></hp:tc>"
                ));
                let _ = r;
            }
            out.push_str("</hp:tr>");
        }
        out.push_str("</hp:tbl>");
        out
    }

    #[test]
    fn recompute_simple_2x2() {
        let tbl = make_simple_table(2, 2);
        let out = recompute_table_addrs(&tbl).unwrap();
        // rowCnt=2 (opening tag)
        assert!(out.contains("rowCnt=\"2\""));
        // 4개 셀 모두 99 -> 정확한 값으로
        assert!(!out.contains("colAddr=\"99\""));
        assert!(!out.contains("rowAddr=\"99\""));
        // 첫 행: rowAddr=0, colAddr=[0, 1]
        assert!(out.contains("rowAddr=\"0\" colAddr=\"0\""));
        assert!(out.contains("rowAddr=\"0\" colAddr=\"1\""));
        // 둘째 행: rowAddr=1, colAddr=[0, 1]
        assert!(out.contains("rowAddr=\"1\" colAddr=\"0\""));
        assert!(out.contains("rowAddr=\"1\" colAddr=\"1\""));
    }

    #[test]
    fn recompute_with_colspan() {
        // colCnt=3, Row0 [colSpan=2, colSpan=1] → colAddr=[0, 2]
        let tbl = "<hp:tbl rowCnt=\"0\" colCnt=\"3\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"2\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr></hp:tbl>";
        let out = recompute_table_addrs(tbl).unwrap();
        // 첫 셀 colAddr=0, 둘째 셀 colAddr=2
        assert!(out.contains("rowAddr=\"0\" colAddr=\"0\" rowSpan=\"1\" colSpan=\"2\""));
        assert!(out.contains("rowAddr=\"0\" colAddr=\"2\" rowSpan=\"1\" colSpan=\"1\""));
        assert!(out.contains("rowCnt=\"1\""));
    }

    #[test]
    fn recompute_with_rowspan() {
        // colCnt=3, Row0 [rowSpan=2 colSpan=1, _, _] → Row1 cells start from col=1
        let tbl = "<hp:tbl rowCnt=\"0\" colCnt=\"3\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"2\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr>\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr>\
</hp:tbl>";
        let out = recompute_table_addrs(tbl).unwrap();
        // Row0: cells at colAddr 0, 1, 2
        assert!(out.contains("rowAddr=\"0\" colAddr=\"0\" rowSpan=\"2\""));
        assert!(out.contains("rowAddr=\"0\" colAddr=\"1\""));
        assert!(out.contains("rowAddr=\"0\" colAddr=\"2\""));
        // Row1: col=0 점유 → cells at colAddr 1, 2
        assert!(out.contains("rowAddr=\"1\" colAddr=\"1\""));
        assert!(out.contains("rowAddr=\"1\" colAddr=\"2\""));
        assert!(out.contains("rowCnt=\"2\""));
    }

    #[test]
    fn recompute_with_both_spans() {
        // colCnt=3, Row0 [colSpan=1 rowSpan=2, colSpan=2] → Row1은 col=0 점유 + col=1,2도 colspan으로 점유 안 됨
        // Row0 cells: c0(rs=2,cs=1), c1(rs=1,cs=2)  → colAddr [0, 1]
        // Row1 cells: 1개만, col=0 점유 → colAddr=1 (cs=1) but col 1과 2는 점유 안 됨, c1만 들어감 colAddr=1, 다음 c2 colAddr=2
        let tbl = "<hp:tbl rowCnt=\"0\" colCnt=\"3\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"2\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"2\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr>\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr>\
</hp:tbl>";
        let out = recompute_table_addrs(tbl).unwrap();
        // Row0: colAddr [0, 1]
        assert!(out.contains("rowAddr=\"0\" colAddr=\"0\" rowSpan=\"2\" colSpan=\"1\""));
        assert!(out.contains("rowAddr=\"0\" colAddr=\"1\" rowSpan=\"1\" colSpan=\"2\""));
        // Row1: col=0 점유, 다음 cells at colAddr [1, 2]
        assert!(out.contains("rowAddr=\"1\" colAddr=\"1\""));
        assert!(out.contains("rowAddr=\"1\" colAddr=\"2\""));
    }

    #[test]
    fn recompute_byte_exact_outside_attrs() {
        // baseline의 셀 안 본문 텍스트 / 다른 속성 / 공백이 보존되는지 검증
        let tbl = "<hp:tbl rowCnt=\"0\" colCnt=\"2\" hint=\"keep\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\" extra=\"X\"><hp:subList><hp:p><hp:run><hp:t>본문</hp:t></hp:run></hp:p></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p><hp:run><hp:t>본문2</hp:t></hp:run></hp:p></hp:subList></hp:tc>\
</hp:tr></hp:tbl>";
        let out = recompute_table_addrs(tbl).unwrap();
        assert!(out.contains("hint=\"keep\""));
        assert!(out.contains("extra=\"X\""));
        assert!(out.contains("<hp:t>본문</hp:t>"));
        assert!(out.contains("<hp:t>본문2</hp:t>"));
    }

    #[test]
    fn recompute_nested_table_safety() {
        // 외부 셀의 colSpan/rowSpan만 잡고, 내부 nested cell의 cellSpan은 무시되어야 함
        let tbl = "<hp:tbl rowCnt=\"0\" colCnt=\"1\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\">\
<hp:subList><hp:p>\
<hp:tbl rowCnt=\"99\" colCnt=\"2\"><hp:tr>\
<hp:tc rowAddr=\"55\" colAddr=\"55\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"55\" colAddr=\"55\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr></hp:tbl>\
</hp:p></hp:subList>\
</hp:tc>\
</hp:tr></hp:tbl>";
        let out = recompute_table_addrs(tbl).unwrap();
        // 외부 셀 0,0 — 갱신됨
        assert!(out.contains("rowAddr=\"0\" colAddr=\"0\""));
        // nested table도 자체적으로 recompute됨 (재귀적 처리) → "rowAddr=\"55\"" 사라져야 함
        assert!(!out.contains("rowAddr=\"55\""));
        assert!(!out.contains("colAddr=\"55\""));
        // outermost rowCnt=1
        assert!(out.starts_with("<hp:tbl rowCnt=\"1\""));
    }

    // ─── Stage 3: table-aware paste 통합 ───

    #[test]
    fn paste_fragment_table_auto_recomputes() {
        let section = empty_section();
        let mut header = empty_header();
        // 표 fragment는 paragraph로 래핑되어야 한다 — <hp:p> 안에 <hp:tbl>
        let fragment = "<hp:p paraPrIDRef=\"0\"><hp:tbl rowCnt=\"99\" colCnt=\"2\">\
<hp:tr>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
<hp:tc rowAddr=\"99\" colAddr=\"99\" rowSpan=\"1\" colSpan=\"1\"><hp:subList><hp:p/></hp:subList></hp:tc>\
</hp:tr></hp:tbl></hp:p>";
        let source = SourceDefinitions::default();
        let result =
            paste_fragment_into_section(&section, &mut header, 0, fragment, &source).unwrap();
        // rowCnt 99 → 1로 보정
        assert!(result.new_section_xml.contains("rowCnt=\"1\""));
        assert!(!result.new_section_xml.contains("rowAddr=\"99\""));
    }

    #[test]
    fn paste_fragment_paragraphs_only_unchanged() {
        // 표가 없는 fragment는 paste_paragraphs_into_section 동작과 동일
        let section = empty_section();
        let mut header = empty_header();
        let fragment =
            r#"<hp:p paraPrIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();
        let result =
            paste_fragment_into_section(&section, &mut header, 0, fragment, &source).unwrap();
        assert_eq!(result.inserted_para_count, 1);
    }

    #[test]
    fn header_insertion_missing_returns_error() {
        // 닫는 charPropertyList 태그가 없는 손상된 header
        let mut header = "<hh:head><hh:charPr id=\"5\"/></hh:head>".to_string();
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"9\" height=\"2000\"/>".into(),
            ..Default::default()
        };
        let err = build_id_remap(&mut header, &source).unwrap_err();
        assert!(matches!(
            err,
            FragmentPasteError::HeaderInsertionPointMissing(_)
        ));
    }
}
