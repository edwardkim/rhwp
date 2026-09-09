//! `Contents/header.xml` 조립과 그림 바이너리 추출.
//!
//! 파이썬 정본 `hj/conv/pack.py` 를 옮긴 것이다. `refList` 순서는 원본 HWPX 와 같게 맞춘다:
//! fontfaces → borderFills → charProperties → tabProperties → numberings → bullets
//! → paraProperties → styles → memoProperties.

use serde_json::Value;

use super::ctx::{esc_attr, getarr, getb, geti, gets, sorted_items, sub, Ids, Model};

const HEAD_OPEN: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>",
    "<hh:head xmlns:ha=\"http://www.hancom.co.kr/hwpml/2011/app\"",
    " xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\"",
    " xmlns:hp10=\"http://www.hancom.co.kr/hwpml/2016/paragraph\"",
    " xmlns:hs=\"http://www.hancom.co.kr/hwpml/2011/section\"",
    " xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\"",
    " xmlns:hh=\"http://www.hancom.co.kr/hwpml/2011/head\"",
    " xmlns:hhs=\"http://www.hancom.co.kr/hwpml/2011/history\"",
    " xmlns:hm=\"http://www.hancom.co.kr/hwpml/2011/master-page\"",
    " xmlns:hpf=\"http://www.hancom.co.kr/schema/2011/hpf\"",
    " xmlns:dc=\"http://purl.org/dc/elements/1.1/\"",
    " xmlns:opf=\"http://www.idpf.org/2007/opf/\"",
    " xmlns:ooxmlchart=\"http://www.hancom.co.kr/hwpml/2016/ooxmlchart\"",
    " xmlns:hwpunitchar=\"http://www.hancom.co.kr/hwpml/2016/HwpUnitChar\"",
    " xmlns:epub=\"http://www.idpf.org/2007/ops\"",
    " xmlns:config=\"urn:oasis:names:tc:opendocument:xmlns:config:1.0\"",
    " version=\"1.5\" secCnt=\"1\">",
);

const TAB_TYPE: [&str; 4] = ["LEFT", "RIGHT", "CENTER", "DECIMAL"];
const LEADER: [&str; 4] = ["NONE", "SOLID", "DASH", "DOT"];
const NUM_FORMAT: [&str; 15] = [
    "DIGIT",
    "CIRCLED_DIGIT",
    "ROMAN_CAPITAL",
    "ROMAN_SMALL",
    "LATIN_CAPITAL",
    "LATIN_SMALL",
    "CIRCLED_LATIN_CAPITAL",
    "CIRCLED_LATIN_SMALL",
    "HANGUL_SYLLABLE",
    "CIRCLED_HANGUL_SYLLABLE",
    "HANGUL_JAMO",
    "CIRCLED_HANGUL_JAMO",
    "HANGUL_PHONETIC",
    "IDEOGRAPH",
    "CIRCLED_IDEOGRAPH",
];
const ALIGN: [&str; 3] = ["LEFT", "CENTER", "RIGHT"];
const TEXT_OFFSET: [&str; 2] = ["PERCENT", "HWPUNIT"];

/// 표에 없는 열거값은 0번으로 떨어뜨린다(파이썬 `dict.get(v, 기본)` 과 같다).
fn pick(table: &[&'static str], v: i64) -> &'static str {
    if v < 0 {
        return table[0];
    }
    table.get(v as usize).copied().unwrap_or(table[0])
}

fn b(v: bool) -> &'static str {
    if v {
        "1"
    } else {
        "0"
    }
}

/// 정수 색상 → `#RRGGBB`. 한글은 BGR 순서로 담는다.
fn color(v: &Value, k: &str, dflt: i64) -> String {
    if let Some(s) = v.get(k).and_then(Value::as_str) {
        return s.to_string();
    }
    let n = geti(v, k, dflt) & 0xFF_FFFF;
    let (r, g, bb) = (n & 0xFF, (n >> 8) & 0xFF, (n >> 16) & 0xFF);
    format!("#{:02X}{:02X}{:02X}", r, g, bb)
}

// ---------------------------------------------------------------- 작은 정의표

fn emit_tab_properties(m: &Model) -> String {
    let items = sorted_items(m.table("tp"));
    let mut body = String::new();
    let n = items.len();
    for (i, (_oid, t)) in items.into_iter().enumerate() {
        let mut tab_items = String::new();
        for it in getarr(t, "tp") {
            tab_items.push_str(&format!(
                "<hh:tabItem pos=\"{}\" type=\"{}\" leader=\"{}\"/>",
                geti(it, "po", 0),
                pick(&TAB_TYPE, geti(it, "ty", 0)),
                pick(&LEADER, geti(it, "le", 0))
            ));
        }
        let head = format!(
            "id=\"{}\" autoTabLeft=\"{}\" autoTabRight=\"{}\"",
            i,
            b(getb(t, "al")),
            b(getb(t, "ar"))
        );
        if tab_items.is_empty() {
            body.push_str(&format!("<hh:tabPr {head}/>"));
        } else {
            body.push_str(&format!("<hh:tabPr {head}>{tab_items}</hh:tabPr>"));
        }
    }
    format!("<hh:tabProperties itemCnt=\"{n}\">{body}</hh:tabProperties>")
}

/// 번호매기기·글머리표가 공유하는 `hh:paraHead`.
fn para_head(ids: &Ids, ph: &Value) -> String {
    let cp_ref = gets(ph, "cp");
    let charpr = if cp_ref.is_empty() {
        // 참조 없음 센티널 (uint32 최댓값)
        "4294967295".to_string()
    } else {
        ids.charpr_id.get(cp_ref).copied().unwrap_or(0).to_string()
    };
    let mut head = format!(
        "level=\"{}\" align=\"{}\" useInstWidth=\"{}\" autoIndent=\"{}\" widthAdjust=\"{}\" \
 textOffsetType=\"{}\" textOffset=\"{}\" numFormat=\"{}\" charPrIDRef=\"{}\" checkable=\"0\"",
        geti(ph, "le", 0),
        pick(&ALIGN, geti(ph, "al", 0)),
        b(getb(ph, "ui")),
        b(getb(ph, "ai")),
        geti(ph, "wa", 0),
        pick(&TEXT_OFFSET, geti(ph, "tt", 0)),
        geti(ph, "to", 0),
        pick(&NUM_FORMAT, geti(ph, "uf", 0)),
        charpr
    );
    if ph.get("st").is_some() {
        head.push_str(&format!(" start=\"{}\"", geti(ph, "st", 1)));
    }
    format!(
        "<hh:paraHead {head}>{}</hh:paraHead>",
        esc_attr(gets(ph, "sf"))
    )
}

fn emit_numberings(m: &Model, ids: &Ids) -> String {
    let items = sorted_items(m.table("nu"));
    let n = items.len();
    let mut body = String::new();
    for (i, (_oid, nu)) in items.into_iter().enumerate() {
        let mut heads = String::new();
        for ph in getarr(nu, "ph") {
            heads.push_str(&para_head(ids, ph));
        }
        body.push_str(&format!(
            "<hh:numbering id=\"{}\" start=\"{}\">{}</hh:numbering>",
            i + 1,
            geti(nu, "sn", 1),
            heads
        ));
    }
    format!("<hh:numberings itemCnt=\"{n}\">{body}</hh:numberings>")
}

fn emit_bullets(m: &Model, ids: &Ids) -> String {
    let items = sorted_items(m.table("bu"));
    let n = items.len();
    let mut body = String::new();
    for (i, (_oid, bu)) in items.into_iter().enumerate() {
        body.push_str(&format!(
            "<hh:bullet id=\"{}\" char=\"{}\" useImage=\"{}\">{}</hh:bullet>",
            i + 1,
            esc_attr(gets(bu, "ch")),
            b(getb(bu, "ui")),
            para_head(ids, sub(bu, "ph"))
        ));
    }
    format!("<hh:bullets itemCnt=\"{n}\">{body}</hh:bullets>")
}

fn emit_styles(m: &Model, ids: &mut Ids) -> String {
    let ordered = sorted_items(m.table("st"));
    for (i, (oid, _s)) in ordered.iter().enumerate() {
        ids.style_id.insert((*oid).clone(), i as i64);
    }
    let n = ordered.len();
    let mut body = String::new();
    for (i, (_oid, s)) in ordered.into_iter().enumerate() {
        body.push_str(&format!(
            "<hh:style id=\"{}\" type=\"{}\" name=\"{}\" engName=\"{}\" paraPrIDRef=\"{}\" \
 charPrIDRef=\"{}\" nextStyleIDRef=\"{}\" langID=\"{}\" lockForm=\"{}\"/>",
            i,
            if geti(s, "ty", 0) == 0 {
                "PARA"
            } else {
                "CHAR"
            },
            esc_attr(gets(s, "na")),
            esc_attr(gets(s, "en")),
            ids.parapr_id.get(gets(s, "pp")).copied().unwrap_or(0),
            ids.charpr_id.get(gets(s, "cp")).copied().unwrap_or(0),
            ids.style_id.get(gets(s, "ns")).copied().unwrap_or(0),
            geti(s, "li", 1042),
            b(getb(s, "lf"))
        ));
    }
    format!("<hh:styles itemCnt=\"{n}\">{body}</hh:styles>")
}

fn emit_memo_properties(m: &Model) -> String {
    let items = sorted_items(m.table("mp"));
    let n = items.len();
    let mut body = String::new();
    for (i, (_oid, mm)) in items.into_iter().enumerate() {
        body.push_str(&format!(
            "<hh:memoPr id=\"{}\" width=\"{}\" lineWidth=\"{}\" lineType=\"SOLID\" lineColor=\"{}\" \
 fillColor=\"{}\" activeColor=\"{}\" memoType=\"NOMAL\"/>",
            i + 1,
            geti(mm, "wi", 15591),
            geti(mm, "lw", 1),
            color(mm, "lc", 0),
            color(mm, "fc", 0xCC_FF99),
            color(mm, "ac", 0xFF_FF99)
        ));
    }
    format!("<hh:memoProperties itemCnt=\"{n}\">{body}</hh:memoProperties>")
}

// ---------------------------------------------------------------- 패키지

/// `refList` 순서를 원본과 같게 맞춰 header.xml 을 조립한다.
pub fn build_header(
    m: &Model,
    ids: &mut Ids,
    fontfaces: &str,
    borderfills: &str,
    charprops: &str,
    paraprops: &str,
) -> String {
    let mut out = String::from(HEAD_OPEN);
    out.push_str(
        "<hh:beginNum page=\"1\" footnote=\"1\" endnote=\"1\" pic=\"1\" tbl=\"1\" equation=\"1\"/>",
    );
    out.push_str("<hh:refList>");
    out.push_str(fontfaces);
    out.push_str(borderfills);
    out.push_str(charprops);
    out.push_str(&emit_tab_properties(m));
    out.push_str(&emit_numberings(m, ids));
    out.push_str(&emit_bullets(m, ids));
    out.push_str(paraprops);
    out.push_str(&emit_styles(m, ids));
    out.push_str(&emit_memo_properties(m));
    out.push_str("</hh:refList>");
    out.push_str(
        "<hh:compatibleDocument targetProgram=\"HWP201X\"><hh:layoutCompatibility/></hh:compatibleDocument>",
    );
    out.push_str("<hh:docOption><hh:linkinfo path=\"\" pageInherit=\"1\" footnoteInherit=\"0\"/></hh:docOption>");
    out.push_str("<hh:trackchageConfig flags=\"56\"/>");
    out.push_str("</hh:head>");
    out
}

/// `bidt`(표준 base64) → `(1-base 번호, MIME, 바이트)` 목록. `ids.bin_id` 를 채운다.
///
/// 🔴 모델 `bi` 배열의 위치가 곧 `imageN` 번호다(본문 `binaryItemIDRef` 가 이 번호를 가리킨다).
pub fn bin_entries(m: &Model, ids: &mut Ids) -> Result<Vec<(u16, String, Vec<u8>)>, String> {
    use base64::Engine as _;
    let mut out = Vec::new();
    for (idx, item) in m.bi().iter().enumerate() {
        let sr = gets(item, "sr");
        let ext = match sr.rsplit_once('.') {
            Some((_, e)) => e.to_uppercase(),
            None => "PNG".to_string(),
        };
        let b64: String = m
            .bidt()
            .get(sr)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("hwpjson 이미지 원본이 없다: {sr}"))?
            // 줄바꿈이 섞여 와도 디코드되게 공백류만 걷어낸다(표준 base64 자체는 그대로).
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .collect();
        if b64.is_empty() {
            return Err(format!("hwpjson 이미지 원본이 비어 있다: {sr}"));
        }
        let data = base64::engine::general_purpose::STANDARD
            .decode(b64.as_bytes())
            .map_err(|e| format!("hwpjson 이미지 base64가 손상됐다: {sr}: {e}"))?;
        let mime = {
            let ty = gets(item, "ty");
            if ty.is_empty() {
                format!("image/{}", ext.to_lowercase())
            } else {
                ty.to_string()
            }
        };
        let n = (idx + 1) as u16;
        ids.bin_id.insert(sr.to_string(), n);
        out.push((n, mime, data));
    }
    Ok(out)
}
