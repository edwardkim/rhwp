//! 본문(`Contents/section0.xml`) 생성기.
//!
//! 파이썬 정본 `hj/conv/body.py` 를 옮긴 것이다. 클립보드 문서모델의 `ro`/`sl`/`cs` 풀을
//! HWPX `<hs:sec>` 로 직렬화한다(원본 대조 실적: 문단 259/259·표 15/15·셀 63/63,
//! 마스킹 후 원시 문자열 153,523자 동일).
//!
//! 사양서 `hj/schema-map.md` 의 "문단·문자 ro/cs" · "개체 sl/bi/bidt" 절을 따른다.

use serde_json::{Map, Value};

use super::ctx::{
    attrs, av, b01, enum_raw, esc_attr, esc_text, getarr, getb, geti, getobj, gets, issue, nullv,
    sorted_items, sub, u32v, Ids, Model,
};

// ---------------------------------------------------------------- 상수

pub const SEC_XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>";

const SEC_OPEN: &str = concat!(
    "<hs:sec",
    " xmlns:ha=\"http://www.hancom.co.kr/hwpml/2011/app\"",
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
    " xmlns:config=\"urn:oasis:names:tc:opendocument:xmlns:config:1.0\">",
);

// `ch[].ci` — 컨트롤 4CC(big-endian ASCII) → 종류
const CI_SECD: i64 = 1_936_024_420; // b"secd"
const CI_COLD: i64 = 1_668_246_628; // b"cold"
const CI_PGNP: i64 = 1_885_826_672; // b"pgnp"
const CI_TBL: i64 = 1_952_607_264; // b"tbl "
const CI_GSO: i64 = 1_735_618_336; // b"gso "
                                   // `cs[gso].rc.ci` — 도형 레코드 종류(별도 네임스페이스)
const RC_REC: i64 = 611_476_835; // b"$rec"

// --- 열거표 (사양서에서 확정된 쌍은 주석에 원본 근거를 적었다) -------------
const E_NUMBERING: [&str; 4] = ["NONE", "PICTURE", "TABLE", "EQUATION"]; // 1·2 확정
const E_TEXTWRAP: [&str; 4] = [
    "SQUARE",
    "TOP_AND_BOTTOM",
    "BEHIND_TEXT",
    "IN_FRONT_OF_TEXT",
]; // 1 확정
const E_TEXTFLOW: [&str; 4] = ["BOTH_SIDES", "LEFT_ONLY", "RIGHT_ONLY", "LARGEST_ONLY"]; // 0 확정
const E_WREL: [&str; 5] = ["PAPER", "PAGE", "COLUMN", "PARA", "ABSOLUTE"]; // 4 확정
const E_HREL: [&str; 3] = ["PAPER", "PAGE", "ABSOLUTE"]; // 2 확정
const E_VRELTO: [&str; 3] = ["PAPER", "PAGE", "PARA"]; // 2 확정
const E_HRELTO: [&str; 4] = ["PAPER", "PAGE", "COLUMN", "PARA"]; // 0·3 확정
const E_VALIGN: [&str; 5] = ["TOP", "CENTER", "BOTTOM", "INSIDE", "OUTSIDE"]; // 0·1 확정
const E_HALIGN: [&str; 5] = ["LEFT", "CENTER", "RIGHT", "INSIDE", "OUTSIDE"]; // 0 확정
const E_TEXTDIR: [&str; 2] = ["HORIZONTAL", "VERTICAL"]; // 0 확정
const E_LINEWRAP: [&str; 3] = ["BREAK", "SQUEEZE", "KEEP"]; // 0 확정
const E_TBLBREAK: [&str; 3] = ["NONE", "TABLE", "CELL"]; // 2 확정(나머지 미검증)
const E_IMGEFFECT: [&str; 4] = ["REAL_PIC", "GRAY_SCALE", "BLACK_WHITE", "PATTERN8x8"]; // 0 확정
const E_PAGENUMPOS: [&str; 11] = [
    "NONE",
    "TOP_LEFT",
    "TOP_CENTER",
    "TOP_RIGHT",
    "BOTTOM_LEFT",
    "BOTTOM_CENTER",
    "BOTTOM_RIGHT",
    "OUTSIDE_TOP",
    "OUTSIDE_BOTTOM",
    "INSIDE_TOP",
    "INSIDE_BOTTOM",
]; // 5 확정
const E_NUMFORMAT: [&str; 6] = [
    "DIGIT",
    "CIRCLED_DIGIT",
    "ROMAN_CAPITAL",
    "ROMAN_SMALL",
    "LATIN_CAPITAL",
    "LATIN_SMALL",
]; // 0 확정
const E_NOTENUM: [&str; 3] = ["CONTINUOUS", "ON_SECTION", "ON_PAGE"]; // 0 확정
const E_FN_PLACE: [&str; 3] = ["EACH_COLUMN", "MERGED_COLUMN", "RIGHT_MOST_COLUMN"];
const E_EN_PLACE: [&str; 2] = ["END_OF_DOCUMENT", "END_OF_SECTION"];
const E_LINETYPE: [&str; 12] = [
    "NONE",
    "SOLID",
    "DASH",
    "DOT",
    "DASH_DOT",
    "DASH_DOT_DOT",
    "LONG_DASH",
    "CIRCLE",
    "DOUBLE_SLIM",
    "SLIM_THICK",
    "THICK_SLIM",
    "SLIM_THICK_SLIM",
]; // 0·1 확정
const E_LINEWIDTH: [&str; 16] = [
    "0.1 mm", "0.12 mm", "0.15 mm", "0.2 mm", "0.25 mm", "0.3 mm", "0.4 mm", "0.5 mm", "0.6 mm",
    "0.7 mm", "1.0 mm", "1.5 mm", "2.0 mm", "3.0 mm", "4.0 mm", "5.0 mm",
]; // 0 확정
const E_ENDCAP: [&str; 3] = ["ROUND", "FLAT", "SQUARE"]; // 1 확정
const E_ARROW: [&str; 7] = [
    "NORMAL",
    "ARROW",
    "SPEAR",
    "CONCAVE_ARROW",
    "EMPTY_DIAMOND",
    "EMPTY_CIRCLE",
    "EMPTY_BOX",
]; // 0 확정
const E_ARROWSZ: [&str; 9] = [
    "SMALL_SMALL",
    "SMALL_MEDIUM",
    "SMALL_LARGE",
    "MEDIUM_SMALL",
    "MEDIUM_MEDIUM",
    "MEDIUM_LARGE",
    "LARGE_SMALL",
    "LARGE_MEDIUM",
    "LARGE_LARGE",
]; // 0 확정
const E_OUTLINE: [&str; 3] = ["NORMAL", "OUTER", "INNER"]; // 0 확정
const E_SHADOW: [&str; 5] = [
    "NONE",
    "LEFT_TOP",
    "RIGHT_TOP",
    "LEFT_BOTTOM",
    "RIGHT_BOTTOM",
]; // 0 확정
const E_GUTTER: [&str; 3] = ["LEFT_ONLY", "LEFT_RIGHT", "TOP_BOTTOM"]; // 0 확정
const E_PAGESTART: [&str; 3] = ["BOTH", "EVEN", "ODD"]; // 0 확정
const E_PBF_TYPE: [&str; 3] = ["BOTH", "EVEN", "ODD"]; // 0·1·2 확정
const E_FILLAREA: [&str; 3] = ["PAPER", "PAGE", "BORDER"]; // 0 확정
const E_COLTYPE: [&str; 3] = ["NEWSPAPER", "BALANCED_NEWSPAPER", "PARALLEL"]; // 0 확정
const E_COLLAYOUT: [&str; 3] = ["LEFT", "RIGHT", "MIRROR"]; // 0 확정

// ---------------------------------------------------------------- 유틸

/// JSON 색 정수는 COLORREF `0x00BBGGRR` — 바이트를 뒤집어 `#RRGGBB` 로.
///
/// 🔴 `fonts_charpr::color` 와 다르다 — 본문 쪽은 음수와 `0xFFFFFFFF` 를 모두 `none` 으로 보고,
///    8자리 표기를 쓰지 않는다(파이썬 정본이 두 갈래로 갈라져 있었고 그대로 검증됐다).
fn color(v: i64) -> String {
    if v == 0xFFFF_FFFF || v < 0 {
        return "none".to_string();
    }
    let (r, g, b) = (v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// 행렬 실수 표기 — 정수값이면 정수로(원본이 `e1="1"` 처럼 적는다).
fn num(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "0".to_string(),
        Some(Value::Bool(b)) => b01(*b).to_string(),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                if f.is_finite() && f == f.trunc() {
                    format!("{}", f as i64)
                } else {
                    format!("{f}")
                }
            } else {
                n.to_string()
            }
        }
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// 파이썬 `'%s' % value` 를 그대로 흉내 낸다 — 정본이 `hp:integerParam/@value` 한 자리에서만
/// 이런 날 것 포맷을 쓴다(표본에는 이 분기가 등장하지 않지만 출력이 갈리지 않게 맞춰 둔다).
fn pyfmt(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "None".to_string(),
        Some(Value::Bool(b)) => if *b { "True" } else { "False" }.to_string(),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                let s = format!("{f}");
                // 파이썬 repr 은 정수값 실수도 소수점을 남긴다(1.0 → "1.0")
                if s.contains(['.', 'e', 'E']) || !f.is_finite() {
                    s
                } else {
                    format!("{s}.0")
                }
            } else {
                n.to_string()
            }
        }
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// 파이썬 truthiness 로 본 '비지 않은 객체'.
fn is_truthy_obj(v: &Value) -> bool {
    v.as_object().is_some_and(|o| !o.is_empty())
}

// ---------------------------------------------------------------- id 발급

fn charpr_ref(ids: &mut Ids, objid: &str) -> Option<i64> {
    issue(&mut ids.charpr_id, objid)
}

fn parapr_ref(ids: &mut Ids, objid: &str) -> Option<i64> {
    issue(&mut ids.parapr_id, objid)
}

fn style_ref(ids: &mut Ids, objid: &str) -> Option<i64> {
    issue(&mut ids.style_id, objid)
}

fn borderfill_ref(ids: &mut Ids, objid: &str) -> Option<i64> {
    issue(&mut ids.borderfill_id, objid)
}

/// `img.bi`(저장소 이름) → `imageN`. `bin_id` 가 비면 `bi` 배열 순서로 채운다.
fn bin_ref(m: &Model, ids: &mut Ids, key: &str) -> String {
    if ids.bin_id.is_empty() {
        for (i, item) in m.bi().iter().enumerate() {
            let sr = match item {
                Value::String(s) => s.as_str(),
                other => other.get("sr").and_then(Value::as_str).unwrap_or(""),
            };
            if !sr.is_empty() {
                ids.bin_id.insert(sr.to_string(), (i + 1) as u16);
            }
        }
    }
    let n = match ids.bin_id.get(key) {
        Some(n) => *n,
        None => {
            // 못 찾으면 등장 순서로 뒤에 붙인다
            let n = (ids.bin_id.len() + 1) as u16;
            ids.bin_id.insert(key.to_string(), n);
            n
        }
    };
    format!("image{n}")
}

/// `nu`(개요)·`mp`(메모) 처럼 별도 발급표가 없는 풀 — 정의표와 같은 순서(id)로 1부터.
fn pool_ref(pool: &Map<String, Value>, objid: &str) -> Option<i64> {
    if objid.is_empty() {
        return None;
    }
    for (i, (k, _)) in sorted_items(pool).into_iter().enumerate() {
        if k == objid {
            return Some(1 + i as i64);
        }
    }
    Some(1)
}

// ---------------------------------------------------------------- 구역 정의

fn emit_secpr(m: &Model, ids: &mut Ids, o: &Value) -> String {
    let a = attrs(&[
        ("id", av("")),
        ("textDirection", av(enum_raw(&E_TEXTDIR, geti(o, "td", 0)))),
        ("spaceColumns", av(geti(o, "sc", 0))),
        // ts 는 HWPUNIT 의 2배 스케일 — tabStop=ts, tabStopVal=ts/2 (사양서 반증)
        ("tabStop", av(geti(o, "ts", 0))),
        ("tabStopVal", av(geti(o, "ts", 0) / 2)),
        ("tabStopUnit", av("HWPUNIT")),
        (
            "outlineShapeIDRef",
            pool_ref(m.table("nu"), gets(o, "os")).map(|v| v.to_string()),
        ),
        (
            "memoShapeIDRef",
            pool_ref(m.table("mp"), gets(o, "ms")).map(|v| v.to_string()),
        ),
        ("textVerticalWidthHead", av(b01(getb(o, "tv")))),
        ("masterPageCnt", av(getarr(o, "mp").len())),
    ]);
    let mut out = format!("<hp:secPr{a}>");
    out.push_str(&format!(
        "<hp:grid lineGrid=\"{}\" charGrid=\"{}\" wonggojiFormat=\"{}\"/>",
        geti(o, "gl", 0),
        geti(o, "gc", 0),
        b01(getb(o, "gw"))
    ));
    out.push_str(&format!(
        "<hp:startNum pageStartsOn=\"{}\" page=\"{}\" pic=\"{}\" tbl=\"{}\" equation=\"{}\"/>",
        enum_raw(&E_PAGESTART, geti(o, "ns", 0)),
        geti(o, "np", 0),
        geti(o, "ni", 0),
        geti(o, "nt", 0),
        geti(o, "ne", 0)
    ));
    // border/fill 은 JSON bool 4개(fb·hb·fi·hi) ↔ HWPX 열거 2개의 대응이 미확정.
    // 표본이 전부 false/SHOW_ALL 이라 SHOW_ALL 고정으로 낸다(사양서 미해결 항목).
    out.push_str(&format!(
        "<hp:visibility hideFirstHeader=\"{}\" hideFirstFooter=\"{}\" \
 hideFirstMasterPage=\"{}\" border=\"SHOW_ALL\" fill=\"SHOW_ALL\" \
 hideFirstPageNum=\"{}\" hideFirstEmptyLine=\"{}\" showLineNumber=\"{}\"/>",
        b01(getb(o, "hh")),
        b01(getb(o, "hf")),
        b01(getb(o, "hm")),
        b01(getb(o, "hp")),
        b01(getb(o, "he")),
        b01(getb(o, "sl"))
    ));
    out.push_str(&format!(
        "<hp:lineNumberShape restartType=\"{}\" countBy=\"{}\" distance=\"{}\" \
 startNumber=\"{}\"/>",
        geti(o, "lr", 0),
        geti(o, "lc", 0),
        geti(o, "ld", 0),
        geti(o, "ls", 0)
    ));
    let pp = sub(o, "pp");
    out.push_str(&format!(
        "<hp:pagePr landscape=\"{}\" width=\"{}\" height=\"{}\" gutterType=\"{}\">",
        if getb(pp, "ls") { "NARROWLY" } else { "WIDELY" },
        geti(pp, "wi", 0),
        geti(pp, "he", 0),
        enum_raw(&E_GUTTER, geti(pp, "gt", 0))
    ));
    out.push_str(&format!(
        "<hp:margin header=\"{}\" footer=\"{}\" gutter=\"{}\" left=\"{}\" \
 right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
        geti(pp, "mh", 0),
        geti(pp, "mf", 0),
        geti(pp, "mg", 0),
        geti(pp, "ml", 0),
        geti(pp, "mr", 0),
        geti(pp, "mt", 0),
        geti(pp, "mb", 0)
    ));
    out.push_str("</hp:pagePr>");
    out.push_str(&emit_notepr(sub(o, "fn"), "footNotePr", &E_FN_PLACE));
    out.push_str(&emit_notepr(sub(o, "en"), "endNotePr", &E_EN_PLACE));
    for pb in getarr(o, "pb") {
        out.push_str(&format!(
            "<hp:pageBorderFill type=\"{}\" borderFillIDRef=\"{}\" \
 textBorder=\"{}\" headerInside=\"{}\" footerInside=\"{}\" fillArea=\"{}\">",
            enum_raw(&E_PBF_TYPE, geti(pb, "ty", 0)),
            opt_str(borderfill_ref(ids, gets(pb, "bf"))),
            if getb(pb, "tb") { "PAPER" } else { "CONTENT" },
            b01(getb(pb, "hi")),
            b01(getb(pb, "fi")),
            enum_raw(&E_FILLAREA, geti(pb, "fa", 0))
        ));
        out.push_str(&format!(
            "<hp:offset left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
            geti(pb, "ol", 0),
            geti(pb, "or", 0),
            geti(pb, "ot", 0),
            geti(pb, "ob", 0)
        ));
        out.push_str("</hp:pageBorderFill>");
    }
    out.push_str("</hp:secPr>");
    out
}

/// 파이썬은 `%s` 로 `None` 을 "None" 이라 적는다. 여기서도 같은 자리(속성 생략이 아닌 곳)에서만
/// 쓰이며, 실제 모델에서는 항상 값이 있다.
fn opt_str(v: Option<i64>) -> String {
    match v {
        Some(v) => v.to_string(),
        None => "None".to_string(),
    }
}

fn emit_notepr(n: &Value, tag: &str, place_table: &[&'static str]) -> String {
    let mut out = format!("<hp:{tag}>");
    out.push_str(&format!(
        "<hp:autoNumFormat type=\"{}\" userChar=\"{}\" prefixChar=\"{}\" \
 suffixChar=\"{}\" supscript=\"{}\"/>",
        enum_raw(&E_NUMFORMAT, geti(n, "at", 0)),
        esc_attr(gets(n, "au")),
        esc_attr(gets(n, "ap")),
        esc_attr(gets(n, "ac")),
        b01(getb(n, "as"))
    ));
    out.push_str(&format!(
        "<hp:noteLine length=\"{}\" type=\"{}\" width=\"{}\" color=\"{}\"/>",
        geti(n, "ll", -1),
        enum_raw(&E_LINETYPE, geti(n, "lt", 1)),
        enum_raw(&E_LINEWIDTH, geti(n, "lw", 0)),
        color(geti(n, "lc", 0))
    ));
    out.push_str(&format!(
        "<hp:noteSpacing betweenNotes=\"{}\" belowLine=\"{}\" aboveLine=\"{}\"/>",
        geti(n, "st", 0),
        geti(n, "sb", 0),
        geti(n, "sa", 0)
    ));
    out.push_str(&format!(
        "<hp:numbering type=\"{}\" newNum=\"{}\"/>",
        enum_raw(&E_NOTENUM, geti(n, "nt", 0)),
        geti(n, "nn", 1)
    ));
    out.push_str(&format!(
        "<hp:placement place=\"{}\" beneathText=\"{}\"/>",
        enum_raw(place_table, geti(n, "pp", 0)),
        b01(getb(n, "pb"))
    ));
    out.push_str(&format!("</hp:{tag}>"));
    out
}

fn emit_colpr(o: &Value) -> String {
    format!(
        "<hp:ctrl><hp:colPr id=\"\" type=\"{}\" layout=\"{}\" colCount=\"{}\" \
 sameSz=\"{}\" sameGap=\"{}\"/></hp:ctrl>",
        enum_raw(&E_COLTYPE, geti(o, "ty", 0)),
        enum_raw(&E_COLLAYOUT, geti(o, "la", 0)),
        geti(o, "co", 1),
        b01(getb(o, "ss")),
        b01(getb(o, "sg"))
    )
}

fn emit_pagenum(o: &Value) -> String {
    let pn = sub(o, "pn");
    format!(
        "<hp:ctrl><hp:pageNum pos=\"{}\" formatType=\"{}\" sideChar=\"{}\"/></hp:ctrl>",
        enum_raw(&E_PAGENUMPOS, geti(pn, "po", 0)),
        enum_raw(&E_NUMFORMAT, geti(pn, "ft", 0)),
        esc_attr(gets(pn, "sc"))
    )
}

// ---------------------------------------------------------------- 개체 공통

fn emit_sz(o: &Value) -> String {
    format!(
        "<hp:sz width=\"{}\" widthRelTo=\"{}\" height=\"{}\" heightRelTo=\"{}\" \
 protect=\"{}\"/>",
        geti(o, "swi", 0),
        enum_raw(&E_WREL, geti(o, "swr", 4)),
        geti(o, "she", 0),
        enum_raw(&E_HREL, geti(o, "shr", 2)),
        b01(getb(o, "spr"))
    )
}

fn emit_pos(o: &Value) -> String {
    format!(
        "<hp:pos treatAsChar=\"{}\" affectLSpacing=\"{}\" flowWithText=\"{}\" \
 allowOverlap=\"{}\" holdAnchorAndSO=\"{}\" vertRelTo=\"{}\" horzRelTo=\"{}\" \
 vertAlign=\"{}\" horzAlign=\"{}\" vertOffset=\"{}\" horzOffset=\"{}\"/>",
        b01(getb(o, "pta")),
        b01(getb(o, "pal")),
        b01(getb(o, "pfw")),
        b01(getb(o, "pao")),
        b01(getb(o, "pha")),
        enum_raw(&E_VRELTO, geti(o, "pvr", 2)),
        enum_raw(&E_HRELTO, geti(o, "phr", 3)),
        enum_raw(&E_VALIGN, geti(o, "pva", 0)),
        enum_raw(&E_HALIGN, geti(o, "ph1", 0)),
        geti(o, "pvo", 0),
        geti(o, "ph2", 0)
    )
}

fn emit_outmargin(o: &Value) -> String {
    format!(
        "<hp:outMargin left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
        geti(o, "ole", 0),
        geti(o, "ori", 0),
        geti(o, "oto", 0),
        geti(o, "obo", 0)
    )
}

fn emit_shapecomment(o: &Value) -> String {
    let sc = gets(o, "sc");
    if sc.is_empty() {
        return String::new();
    }
    // JSON 은 CRLF, HWPX 는 LF (사양서 확정)
    let sc = sc.replace("\r\n", "\n").replace('\r', "\n");
    format!("<hp:shapeComment>{}</hp:shapeComment>", esc_text(&sc))
}

/// `ps = {na, pa:[{na, ty, va}]}` → `hp:parameterset`/`hp:listParam`/`hp:stringParam`.
fn emit_paramset(o: &Value) -> String {
    let Some(ps) = o.get("ps").filter(|v| is_truthy_obj(v)) else {
        return String::new();
    };
    let pa = getarr(ps, "pa");
    let mut out = format!(
        "<hp:parameterset cnt=\"{}\" name=\"{}\">",
        pa.len(),
        geti(ps, "na", 0)
    );
    for it in pa {
        out.push_str(&emit_param(it));
    }
    out.push_str("</hp:parameterset>");
    out
}

fn emit_param(it: &Value) -> String {
    let ty = geti(it, "ty", -1);
    let va = it.get("va");
    if ty == 32768 {
        if let Some(inner_obj) = va.filter(|v| v.is_object()) {
            // 중첩 세트
            let inner = getarr(inner_obj, "pa");
            let name = if inner_obj.get("na").is_some() {
                geti(inner_obj, "na", 0)
            } else {
                geti(it, "na", 0)
            };
            let mut out = format!("<hp:listParam cnt=\"{}\" name=\"{}\">", inner.len(), name);
            for s in inner {
                out.push_str(&emit_param(s));
            }
            out.push_str("</hp:listParam>");
            return out;
        }
    }
    if ty == 1 {
        return format!(
            "<hp:stringParam name=\"{}\">{}</hp:stringParam>",
            geti(it, "na", 0),
            esc_text(va.and_then(Value::as_str).unwrap_or(""))
        );
    }
    format!(
        "<hp:integerParam name=\"{}\" value=\"{}\"/>",
        geti(it, "na", 0),
        pyfmt(va)
    )
}

/// `hp:tbl` / `hp:pic` / `hp:rect` 공통 앞머리 속성.
fn obj_common_attrs(o: &Value, extra: &[(&str, Option<String>)]) -> String {
    let mut v: Vec<(&str, Option<String>)> = vec![
        ("id", av(u32v(geti(o, "id", 0)))),
        ("zOrder", av(geti(o, "zo", 0))),
        (
            "numberingType",
            av(enum_raw(&E_NUMBERING, geti(o, "nt", 0))),
        ),
        ("textWrap", av(enum_raw(&E_TEXTWRAP, geti(o, "tw", 1)))),
        ("textFlow", av(enum_raw(&E_TEXTFLOW, geti(o, "tf", 0)))),
        ("lock", av(b01(getb(o, "lo")))),
        ("dropcapstyle", av("None")),
    ];
    v.extend_from_slice(extra);
    attrs(&v)
}

// ---------------------------------------------------------------- 표

fn emit_tbl(m: &Model, ids: &mut Ids, o: &Value) -> Result<String, String> {
    let a = obj_common_attrs(
        o,
        &[
            ("pageBreak", av(enum_raw(&E_TBLBREAK, geti(o, "pb", 2)))),
            ("repeatHeader", av(b01(getb(o, "rh")))),
            ("rowCnt", av(geti(o, "rc", 0))),
            ("colCnt", av(geti(o, "cco", 0))),
            ("cellSpacing", av(geti(o, "cs", 0))),
            (
                "borderFillIDRef",
                borderfill_ref(ids, gets(o, "bf")).map(|v| v.to_string()),
            ),
            ("noAdjust", av(b01(getb(o, "na")))),
        ],
    );
    let mut out = format!("<hp:tbl{a}>");
    out.push_str(&emit_sz(o));
    out.push_str(&emit_pos(o));
    out.push_str(&emit_outmargin(o));
    out.push_str(&format!(
        "<hp:inMargin left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
        geti(o, "ile", 0),
        geti(o, "iri", 0),
        geti(o, "ito", 0),
        geti(o, "ibo", 0)
    ));
    if getb(o, "cl") {
        out.push_str("<hp:cellzoneList/>"); // 표본 없음 — 빈 목록만 방어적으로
    }
    for row in getarr(o, "tr") {
        out.push_str("<hp:tr>");
        for cell in row.as_array().map_or(&[][..], |v| &v[..]) {
            out.push_str(&emit_tc(m, ids, cell)?);
        }
        out.push_str("</hp:tr>");
    }
    out.push_str(&emit_shapecomment(o));
    out.push_str("</hp:tbl>");
    Ok(out)
}

/// `tr[][].ps = {na:539, pa:[{na:16384, ty:1, va:"셀이름"}]}` → `hp:tc/@name`.
fn cell_name(cell: &Value) -> String {
    let Some(ps) = cell.get("ps").filter(|v| is_truthy_obj(v)) else {
        return String::new();
    };
    for it in getarr(ps, "pa") {
        if geti(it, "na", 0) == 16384 && geti(it, "ty", 0) == 1 {
            return it
                .get("va")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
        }
    }
    String::new()
}

fn emit_tc(m: &Model, ids: &mut Ids, cell: &Value) -> Result<String, String> {
    let head = m.table("sl").get(gets(cell, "so")).unwrap_or(nullv());
    let tc = sub(head, "tc");
    let a = attrs(&[
        ("name", av(cell_name(cell))),
        ("header", av(b01(getb(tc, "he")))),
        ("hasMargin", av(b01(getb(tc, "hm")))),
        ("protect", av(b01(getb(tc, "pr")))),
        ("editable", av(b01(getb(tc, "ed")))),
        ("dirty", av(b01(getb(tc, "di")))),
        (
            "borderFillIDRef",
            borderfill_ref(ids, gets(tc, "bf")).map(|v| v.to_string()),
        ),
    ]);
    let mut out = format!("<hp:tc{a}>");
    out.push_str(&emit_sublist(m, ids, head)?);
    out.push_str(&format!(
        "<hp:cellAddr colAddr=\"{}\" rowAddr=\"{}\"/>",
        geti(tc, "ac", 0),
        geti(tc, "ar", 0)
    ));
    out.push_str(&format!(
        "<hp:cellSpan colSpan=\"{}\" rowSpan=\"{}\"/>",
        geti(tc, "sc", 1),
        geti(tc, "sr", 1)
    ));
    out.push_str(&format!(
        "<hp:cellSz width=\"{}\" height=\"{}\"/>",
        geti(tc, "sw", 0),
        geti(tc, "sh", 0)
    ));
    out.push_str(&format!(
        "<hp:cellMargin left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
        geti(tc, "ml", 0),
        geti(tc, "mr", 0),
        geti(tc, "mt", 0),
        geti(tc, "mb", 0)
    ));
    out.push_str("</hp:tc>");
    Ok(out)
}

/// 표 칸·도형 글상자의 `hp:subList` (문단 포함).
fn emit_sublist(m: &Model, ids: &mut Ids, head: &Value) -> Result<String, String> {
    // JSON 은 객체, HWPX 는 그 JSON 문자열을 속성에 이스케이프해 넣는다
    let metatag = getobj(head, "mt")
        .filter(|mt| !mt.is_empty())
        .and_then(|mt| serde_json::to_string(&Value::Object(mt.clone())).ok());
    let a = attrs(&[
        ("id", av("")),
        (
            "textDirection",
            av(enum_raw(&E_TEXTDIR, geti(head, "td", 0))),
        ),
        ("lineWrap", av(enum_raw(&E_LINEWRAP, geti(head, "lw", 0)))),
        ("vertAlign", av(enum_raw(&E_VALIGN, geti(head, "va", 0)))),
        (
            "linkListIDRef",
            av(non_empty(gets(head, "ll")).unwrap_or("0")),
        ),
        (
            "linkListNextIDRef",
            av(non_empty(gets(head, "ln")).unwrap_or("0")),
        ),
        ("textWidth", av(0)),
        ("textHeight", av(0)),
        ("hasTextRef", av(0)),
        ("hasNumRef", av(0)),
        ("metatag", metatag),
    ]);
    let mut out = format!("<hp:subList{a}>");
    let sl = m.table("sl");
    for pid in m.sublist_paragraph_ids(gets(head, "hp")) {
        if let Some(p) = sl.get(&pid) {
            out.push_str(&emit_paragraph(m, ids, p)?);
        }
    }
    out.push_str("</hp:subList>");
    Ok(out)
}

fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

// ---------------------------------------------------------------- 그리기 개체

fn emit_gso(m: &Model, ids: &mut Ids, o: &Value) -> Result<String, String> {
    let rc = sub(o, "rc");
    if geti(rc, "ci", 0) == RC_REC {
        emit_rect(m, ids, o, rc)
    } else {
        Ok(emit_pic(m, ids, o, rc))
    }
}

fn emit_shape_head(
    o: &Value,
    rc: &Value,
    tag: &str,
    last_attr: &[(&str, Option<String>)],
) -> String {
    let mut extra: Vec<(&str, Option<String>)> = vec![
        ("href", av(gets(rc, "hr"))),
        ("groupLevel", av(geti(rc, "gl", 0))),
        ("instid", av(u32v(geti(rc, "ii", 0)))),
    ];
    extra.extend_from_slice(last_attr);
    let a = obj_common_attrs(o, &extra);
    let mut out = format!("<hp:{tag}{a}>");
    // JSON 은 부호 있는 int32, HWPX 는 부호 없는 uint32
    out.push_str(&format!(
        "<hp:offset x=\"{}\" y=\"{}\"/>",
        u32v(geti(rc, "ox", 0)),
        u32v(geti(rc, "oy", 0))
    ));
    out.push_str(&format!(
        "<hp:orgSz width=\"{}\" height=\"{}\"/>",
        geti(rc, "ow", 0),
        geti(rc, "oh", 0)
    ));
    out.push_str(&format!(
        "<hp:curSz width=\"{}\" height=\"{}\"/>",
        geti(rc, "cw", 0),
        geti(rc, "ch", 0)
    ));
    out.push_str(&format!(
        "<hp:flip horizontal=\"{}\" vertical=\"{}\"/>",
        b01(getb(rc, "fh")),
        b01(getb(rc, "fv"))
    ));
    out.push_str(&format!(
        "<hp:rotationInfo angle=\"{}\" centerX=\"{}\" centerY=\"{}\" rotateimage=\"{}\"/>",
        geti(rc, "ra", 0),
        geti(rc, "rcx", 0),
        geti(rc, "rcy", 0),
        b01(getb(rc, "ri"))
    ));
    out.push_str(&emit_rendering(sub(rc, "in")));
    out
}

fn mat(tag: &str, m: &Value) -> String {
    format!(
        "<hc:{} e1=\"{}\" e2=\"{}\" e3=\"{}\" e4=\"{}\" e5=\"{}\" e6=\"{}\"/>",
        tag,
        num(m.get("e1")),
        num(m.get("e2")),
        num(m.get("e3")),
        num(m.get("e4")),
        num(m.get("e5")),
        num(m.get("e6"))
    )
}

fn emit_rendering(inf: &Value) -> String {
    let mut out = String::from("<hp:renderingInfo>");
    out.push_str(&mat("transMatrix", sub(inf, "tm")));
    for step in getarr(inf, "re") {
        out.push_str(&mat("scaMatrix", sub(step, "sm")));
        out.push_str(&mat("rotMatrix", sub(step, "ro")));
    }
    out.push_str("</hp:renderingInfo>");
    out
}

fn emit_pic(m: &Model, ids: &mut Ids, o: &Value, rc: &Value) -> String {
    let mut out = emit_shape_head(o, rc, "pic", &[("reverse", av(b01(getb(rc, "rc"))))]);
    let img = sub(rc, "img");
    out.push_str(&format!(
        "<hc:img binaryItemIDRef=\"{}\" bright=\"{}\" contrast=\"{}\" \
 effect=\"{}\" alpha=\"{}\"/>",
        bin_ref(m, ids, gets(img, "bi")),
        geti(img, "br", 0),
        geti(img, "co", 0),
        enum_raw(&E_IMGEFFECT, geti(img, "ef", 0)),
        geti(img, "al", 0)
    ));
    out.push_str("<hp:imgRect>");
    for i in 0..4 {
        out.push_str(&format!(
            "<hc:pt{} x=\"{}\" y=\"{}\"/>",
            i,
            geti(rc, &format!("ix{i}"), 0),
            geti(rc, &format!("iy{i}"), 0)
        ));
    }
    out.push_str("</hp:imgRect>");
    out.push_str(&format!(
        "<hp:imgClip left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
        geti(rc, "il", 0),
        geti(rc, "ir", 0),
        geti(rc, "it", 0),
        geti(rc, "ib", 0)
    ));
    out.push_str(&format!(
        "<hp:inMargin left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
        geti(rc, "ml", 0),
        geti(rc, "mr", 0),
        geti(rc, "mt", 0),
        geti(rc, "mb", 0)
    ));
    out.push_str(&format!(
        "<hp:imgDim dimwidth=\"{}\" dimheight=\"{}\"/>",
        geti(rc, "iw", 0),
        geti(rc, "ih", 0)
    ));
    // 효과(ef)는 표본이 전부 빈 객체라 채워진 형태가 미확정 — 항상 빈 요소로 낸다.
    out.push_str("<hp:effects/>");
    out.push_str(&emit_sz(o));
    out.push_str(&emit_pos(o));
    out.push_str(&emit_outmargin(o));
    out.push_str(&emit_shapecomment(o));
    out.push_str(&emit_paramset(o));
    out.push_str("</hp:pic>");
    out
}

fn emit_rect(m: &Model, ids: &mut Ids, o: &Value, rc: &Value) -> Result<String, String> {
    let mut out = emit_shape_head(o, rc, "rect", &[("ratio", av(geti(rc, "rra", 0)))]);
    let ls = sub(rc, "ls");
    out.push_str(&format!(
        "<hp:lineShape color=\"{}\" width=\"{}\" style=\"{}\" endCap=\"{}\" \
 headStyle=\"{}\" tailStyle=\"{}\" headfill=\"{}\" tailfill=\"{}\" \
 headSz=\"{}\" tailSz=\"{}\" outlineStyle=\"{}\" alpha=\"{}\"/>",
        color(geti(ls, "co", 0)),
        geti(ls, "wi", 0),
        enum_raw(&E_LINETYPE, geti(ls, "st", 0)),
        enum_raw(&E_ENDCAP, geti(ls, "ec", 0)),
        enum_raw(&E_ARROW, geti(ls, "hs", 0)),
        enum_raw(&E_ARROW, geti(ls, "ts", 0)),
        b01(getb(ls, "he")),
        b01(getb(ls, "ta")),
        enum_raw(&E_ARROWSZ, geti(ls, "h1", 0)),
        enum_raw(&E_ARROWSZ, geti(ls, "t1", 0)),
        enum_raw(&E_OUTLINE, geti(ls, "os", 0)),
        geti(ls, "al", 0)
    ));
    let wb = sub(rc, "fb").get("wb").filter(|v| is_truthy_obj(v));
    if let Some(wb) = wb {
        out.push_str(&format!(
            "<hc:fillBrush><hc:winBrush faceColor=\"{}\" hatchColor=\"{}\" \
 alpha=\"{}\"/></hc:fillBrush>",
            color(geti(wb, "fc", 0)),
            color(geti(wb, "hc", 0)),
            geti(wb, "al", 0)
        ));
    }
    let sh = sub(rc, "sh");
    if getb(rc, "sh") {
        out.push_str(&format!(
            "<hp:shadow type=\"{}\" color=\"{}\" offsetX=\"{}\" offsetY=\"{}\" alpha=\"{}\"/>",
            enum_raw(&E_SHADOW, geti(sh, "ty", 0)),
            color(geti(sh, "co", 0)),
            geti(sh, "ox", 0),
            geti(sh, "oy", 0),
            geti(sh, "al", 0)
        ));
    }
    let so = gets(rc, "so");
    if let Some(head) = m.table("sl").get(so).filter(|_| !so.is_empty()) {
        let dt = sub(head, "dt");
        // dt.lw 는 클립보드가 -1(미설정)로 버린다 → 소유 컨트롤의 swi 로 복원(사양서 확정)
        let mut lw = geti(dt, "lw", -1);
        if lw < 0 {
            lw = geti(o, "swi", 0);
        }
        out.push_str(&format!(
            "<hp:drawText lastWidth=\"{}\" name=\"\" editable=\"{}\">",
            lw,
            b01(getb(dt, "ed"))
        ));
        out.push_str(&emit_sublist(m, ids, head)?);
        out.push_str(&format!(
            "<hp:textMargin left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\"/>",
            geti(dt, "ml", 0),
            geti(dt, "mr", 0),
            geti(dt, "mt", 0),
            geti(dt, "mb", 0)
        ));
        out.push_str("</hp:drawText>");
    }
    for i in 0..4 {
        out.push_str(&format!(
            "<hc:pt{} x=\"{}\" y=\"{}\"/>",
            i,
            geti(rc, &format!("x{i}"), 0),
            geti(rc, &format!("y{i}"), 0)
        ));
    }
    out.push_str(&emit_sz(o));
    out.push_str(&emit_pos(o));
    out.push_str(&emit_outmargin(o));
    out.push_str(&emit_shapecomment(o));
    out.push_str(&emit_paramset(o));
    out.push_str("</hp:rect>");
    Ok(out)
}

// ---------------------------------------------------------------- 문단·run

/// `{cc, ci, co}` 제어문자 → 그 자리에 인라인 중첩되는 HWPX 요소.
fn emit_control(m: &Model, ids: &mut Ids, ch: &Value) -> Result<String, String> {
    let control_id = gets(ch, "co");
    let Some(o) = m.table("cs").get(control_id) else {
        return Err(format!("지원하지 않는 hwpjson control 참조: {control_id}"));
    };
    match geti(ch, "ci", 0) {
        CI_SECD => Ok(emit_secpr(m, ids, o)), // ctrl 래핑 없음(사양서 반증 확정)
        CI_COLD => Ok(emit_colpr(o)),         // ctrl 래핑 있음
        CI_PGNP => Ok(emit_pagenum(o)),       // ctrl 래핑 있음
        CI_TBL => emit_tbl(m, ids, o),
        CI_GSO => emit_gso(m, ids, o),
        ci => Err(format!(
            "지원하지 않는 hwpjson control 종류: ci={ci}, co={control_id}"
        )),
    }
}

/// 원본은 구역정의 run 과 그 뒤 내용을 분리해 적는다.
///
/// `cc=2`(secd/cold) 뒤에 다른 내용이 이어지면 run 을 둘로 나눈다
/// (원본 첫 문단: run1=[secPr, ctrl/colPr], run2=[ctrl/pageNum, t]).
fn split_run(ch: &[Value]) -> Vec<&[Value]> {
    let mut last2: i64 = -1;
    for (i, c) in ch.iter().enumerate() {
        if geti(c, "cc", 0) == 2 {
            last2 = i as i64;
        }
    }
    if last2 >= 0 && (last2 as usize) < ch.len().saturating_sub(1) {
        let cut = last2 as usize + 1;
        return vec![&ch[..cut], &ch[cut..]];
    }
    vec![ch]
}

fn emit_run(
    m: &Model,
    ids: &mut Ids,
    cp_objid: &str,
    ch: &[Value],
) -> Result<String, String> {
    let cid = charpr_ref(ids, cp_objid);
    let has_obj = ch.iter().any(|c| c.get("t").is_none());
    let mut body = String::new();
    for c in ch {
        if let Some(t) = c.get("t").and_then(Value::as_str) {
            // 개체 없는 run 의 빈 텍스트는 원본이 <hp:run …/> 로만 적는다
            if t.is_empty() && !has_obj {
                continue;
            }
            if t.is_empty() {
                body.push_str("<hp:t/>");
            } else {
                body.push_str(&format!("<hp:t>{}</hp:t>", esc_text(t)));
            }
        } else {
            body.push_str(&emit_control(m, ids, c)?);
        }
    }
    // 🔴 정본은 참조가 없어도 속성을 생략하지 않는다(빈 run 도 charPrIDRef 를 달고 나간다).
    let cid = opt_str(cid);
    if body.is_empty() {
        Ok(format!("<hp:run charPrIDRef=\"{cid}\"/>"))
    } else {
        Ok(format!("<hp:run charPrIDRef=\"{cid}\">{body}</hp:run>"))
    }
}

fn emit_paragraph(m: &Model, ids: &mut Ids, p: &Value) -> Result<String, String> {
    let bf = geti(p, "bf", 0);
    let sec_break = bf & 1; // 구역 나누기 — HWPX 는 구조로 표현, 속성은 전부 0
    let a = attrs(&[
        ("id", av(u32v(geti(p, "id", 0)))),
        (
            "paraPrIDRef",
            parapr_ref(ids, gets(p, "pp")).map(|v| v.to_string()),
        ),
        (
            "styleIDRef",
            style_ref(ids, gets(p, "si")).map(|v| v.to_string()),
        ),
        (
            "pageBreak",
            av(if sec_break != 0 { 0 } else { (bf >> 2) & 1 }),
        ),
        (
            "columnBreak",
            av(if sec_break != 0 { 0 } else { (bf >> 1) & 1 }),
        ),
        ("merged", av(if sec_break != 0 { 0 } else { (bf >> 3) & 1 })),
    ]);
    let mut out = format!("<hp:p{a}>");
    for run in getarr(p, "ru") {
        let ch = getarr(run, "ch");
        let cp = gets(run, "cp").to_string();
        for part in split_run(ch) {
            out.push_str(&emit_run(m, ids, &cp, part)?);
        }
    }
    // hp:linesegarray 는 레이아웃 캐시이며 클립보드 모델에 값이 없다 → 생략
    // (한글이 열 때 다시 계산한다)
    out.push_str("</hp:p>");
    Ok(out)
}

// ---------------------------------------------------------------- 공개 API

/// `<hs:sec …> … </hs:sec>` 전체를 문자열로 만든다.
pub fn emit_section(m: &Model, ids: &mut Ids) -> Result<String, String> {
    let mut out = String::from(SEC_OPEN);
    let ro = m.table("ro");
    for pid in m.body_paragraph_ids() {
        if let Some(p) = ro.get(&pid) {
            out.push_str(&emit_paragraph(m, ids, p)?);
        }
    }
    out.push_str("</hs:sec>");
    Ok(out)
}

/// XML 선언 + 본문.
pub fn emit_section_file(m: &Model, ids: &mut Ids) -> Result<String, String> {
    let mut s = String::from(SEC_XML_DECL);
    s.push_str(&emit_section(m, ids)?);
    Ok(s)
}
