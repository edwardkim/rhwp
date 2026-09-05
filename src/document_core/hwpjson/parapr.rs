//! 문단모양(`hh:paraPr`) · 테두리/배경(`hh:borderFill`) XML 생성기.
//!
//! 파이썬 정본 `hj/conv/parapr.py` 를 옮긴 것이다(원본 대조 실적 paraPr 97/97·borderFill 20/20).
//!
//! 🔴 호출 순서: [`emit_borderfills`] 를 먼저 부른 뒤 [`emit_parapr`] 을 불러야 한다.
//!    paraPr 의 `<hh:border @borderFillIDRef>` 가 `ids.borderfill_id` 를 참조하기 때문이다
//!    (본문 표·글자모양도 같은 표를 본다).
//!
//! 🔴 여백/줄간격 두 갈래(`hp:switch`)
//!    원본은 같은 `<hh:margin>`/`<hh:lineSpacing>` 을 두 번 적는다.
//!        `<hp:case hp:required-namespace="…HwpUnitChar">`  … 값의 1/2 …
//!        `<hp:default>`                                    … 값 그대로 …
//!    클립보드 모델의 `mi/ml/mr/mp/mn`(그리고 길이형 `lv`)은 **hp:default 쪽 값**이다.
//!    실측(header.xml 139개 paraPr): case 와 default 가 다른 105개 전부에서 정확히 비율 2.0,
//!    같은 34개는 값이 0 이거나 lineSpacing 이 PERCENT(비율이라 스케일 없음)인 경우다.
//!    → case = default / 2, 단 lineSpacing type=PERCENT 는 case = default.

use serde_json::Value;

use super::ctx::{attrs, av, b01, enum0, getb, geti, gets, sorted_items, sub, Ids, Model};

/// 클립보드 COLORREF(`0x00BBGGRR`) → HWPX 색 문자열. 규칙은 `fonts_charpr::color` 와 같다.
fn colorref(v: i64) -> String {
    super::fonts_charpr::color(v)
}

/// HWP5 선 굵기 인덱스 → mm 표기 (원본 실측분: 0/1/6/7)
const LINE_WIDTH: [&str; 16] = [
    "0.1 mm", "0.12 mm", "0.15 mm", "0.2 mm", "0.25 mm", "0.3 mm", "0.4 mm", "0.5 mm", "0.6 mm",
    "0.7 mm", "1.0 mm", "1.5 mm", "2.0 mm", "3.0 mm", "4.0 mm", "5.0 mm",
];

/// HWP5 테두리 선 종류 열거 (원본에는 NONE/SOLID 만 등장)
const BORDER_TYPE: [&str; 18] = [
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
    "WAVE",
    "DOUBLE_WAVE",
    "THICK_3D",
    "THICK_3D_INSET",
    "3D",
    "3D_INSET",
];

const SLASH_TYPE: [&str; 5] = ["NONE", "CENTER", "CENTER_BELOW", "CENTER_ABOVE", "ALL"];
const CENTER_LINE: [&str; 4] = ["NONE", "CROSS", "VERTICAL", "HORIZONTAL"];

const ALIGN_H: [&str; 6] = [
    "JUSTIFY",
    "LEFT",
    "RIGHT",
    "CENTER",
    "DISTRIBUTE",
    "DISTRIBUTE_SPACE",
];
const ALIGN_V: [&str; 4] = ["BASELINE", "TOP", "CENTER", "BOTTOM"];
const HEADING_TYPE: [&str; 4] = ["NONE", "OUTLINE", "NUMBER", "BULLET"];
/// 🔴 breakLatinWord 는 3값 열거(사양서 반증 참조). 본 문서는 0(KEEP_WORD)뿐.
const BREAK_LATIN: [&str; 3] = ["KEEP_WORD", "HYPHENATION", "BREAK_WORD"];
const LINE_WRAP: [&str; 3] = ["BREAK", "SQUEEZE", "KEEP"];
const LINESPACING_TYPE: [&str; 4] = ["PERCENT", "FIXED", "BETWEEN_LINES", "AT_LEAST"];

const HWPUNITCHAR_NS: &str = "http://www.hancom.co.kr/hwpml/2016/HwpUnitChar";

/// `hp:case`(HwpUnitChar) 값 = `hp:default` 값의 정확히 1/2. 음수도 대칭이어야 한다
/// (파이썬 `//` 는 음수에서 내림이라 그대로 옮기면 어긋난다).
fn half(n: i64) -> i64 {
    if n < 0 {
        -((-n) / 2)
    } else {
        n / 2
    }
}

// ---------------------------------------------------------- 테두리/배경 bf

fn border_line(tag: &str, t: i64, w: i64, c: i64) -> String {
    let width = if w >= 0 && (w as usize) < LINE_WIDTH.len() {
        LINE_WIDTH[w as usize]
    } else {
        LINE_WIDTH[0]
    };
    format!(
        "<hh:{}{}/>",
        tag,
        attrs(&[
            ("type", av(enum0(&BORDER_TYPE, t))),
            ("width", av(width)),
            ("color", av(colorref(c))),
        ])
    )
}

/// `bf.fi` → `<hc:fillBrush>`. 빈 객체면 요소 자체를 생략한다(원본과 동일).
fn fill_brush(fi: &Value) -> String {
    let Some(wb) = fi.get("wb").filter(|v| v.is_object()) else {
        return String::new();
    };
    format!(
        "<hc:fillBrush><hc:winBrush{}/></hc:fillBrush>",
        attrs(&[
            ("faceColor", av(colorref(geti(wb, "fc", 0)))),
            ("hatchColor", av(colorref(geti(wb, "hc", 0)))),
            ("alpha", av(geti(wb, "al", 0))),
        ])
    )
}

/// `<hh:borderFills>` 를 만들고 `ids.borderfill_id[objid]` 를 채운다.
///
/// 🔴 borderFill id 는 HWPX 규약대로 **1-base** 로 발급한다(원본도 1~22). `borderFillIDRef` 는
///    1 이상이어야 유효하므로 0-base 로 매기면 첫 항목이 '참조 없음'으로 읽힐 위험이 있다.
pub fn emit_borderfills(m: &Model, ids: &mut Ids) -> String {
    let items = sorted_items(m.table("bf"));
    let mut out = format!("<hh:borderFills itemCnt=\"{}\">", items.len());
    for (i, (objid, v)) in items.into_iter().enumerate() {
        let hid = i as i64 + 1;
        ids.borderfill_id.insert(objid.clone(), hid);

        out.push_str(&format!(
            "<hh:borderFill{}>",
            attrs(&[
                ("id", av(hid)),
                ("threeD", av(b01(getb(v, "td")))),
                ("shadow", av(b01(getb(v, "sh")))),
                ("centerLine", av(enum0(&CENTER_LINE, geti(v, "cl", 0)))),
                ("breakCellSeparateLine", av(b01(getb(v, "bc")))),
            ])
        ));
        // 대각선(슬래시/백슬래시) — 모델에 backSlash 의 Crooked 키가 없어 0 고정
        out.push_str(&format!(
            "<hh:slash{}/>",
            attrs(&[
                ("type", av(enum0(&SLASH_TYPE, geti(v, "st", 0)))),
                ("Crooked", av(b01(getb(v, "sc")))),
                ("isCounter", av(b01(getb(v, "si")))),
            ])
        ));
        out.push_str(&format!(
            "<hh:backSlash{}/>",
            attrs(&[
                ("type", av(enum0(&SLASH_TYPE, geti(v, "bt", 0)))),
                ("Crooked", av("0")),
                ("isCounter", av(b01(getb(v, "bi")))),
            ])
        ));
        out.push_str(&border_line(
            "leftBorder",
            geti(v, "lt", 0),
            geti(v, "lw", 0),
            geti(v, "lc", 0),
        ));
        out.push_str(&border_line(
            "rightBorder",
            geti(v, "rt", 0),
            geti(v, "rw", 0),
            geti(v, "rc", 0),
        ));
        out.push_str(&border_line(
            "topBorder",
            geti(v, "tt", 0),
            geti(v, "tw", 0),
            geti(v, "tc", 0),
        ));
        out.push_str(&border_line(
            "bottomBorder",
            geti(v, "bbt", 0),
            geti(v, "bbw", 0),
            geti(v, "bbc", 0),
        ));
        // 🔴 원본은 대각선 종류가 NONE 이면 <hh:diagonal> 요소 자체를 생략한다(borderFill id=20
        //    실측). JSON 은 슬롯을 남기므로 dt==0 일 때 빼야 같아진다.
        if geti(v, "dt", 0) != 0 {
            out.push_str(&border_line(
                "diagonal",
                geti(v, "dt", 0),
                geti(v, "dw", 0),
                geti(v, "dc", 0),
            ));
        }
        out.push_str(&fill_brush(sub(v, "fi")));
        out.push_str("</hh:borderFill>");
    }
    out.push_str("</hh:borderFills>");
    out
}

// ------------------------------------------------------------ 문단모양 pp

/// GUID → 정수 id. 해당 표가 들고 있는 `id` 필드를 그대로 쓴다(번호매기기·글머리표·탭 정의표는
/// `pack` 이 같은 순서로 번호를 매기므로 일치한다).
fn ref_id(m: &Model, table_names: &[&str], objid: &str, dflt: i64) -> i64 {
    if objid.is_empty() {
        return dflt;
    }
    for name in table_names {
        if let Some(item) = m.table(name).get(objid) {
            return geti(item, "id", dflt);
        }
    }
    dflt
}

/// `<hh:margin>`+`<hh:lineSpacing>` 한 벌. `halve=true` 면 HwpUnitChar(case) 쪽.
#[allow(clippy::too_many_arguments)]
fn margin_block(
    mi: i64,
    ml: i64,
    mr: i64,
    mp: i64,
    mn: i64,
    lt: i64,
    lv: i64,
    halve: bool,
) -> String {
    let (mi, ml, mr, mp, mn, lv) = if halve {
        (
            half(mi),
            half(ml),
            half(mr),
            half(mp),
            half(mn),
            // 길이형(FIXED 등)만 절반, 비율(PERCENT=0)은 그대로
            if lt == 0 { lv } else { half(lv) },
        )
    } else {
        (mi, ml, mr, mp, mn, lv)
    };
    let mut out = String::from("<hh:margin>");
    for (tag, val) in [
        ("intent", mi),
        ("left", ml),
        ("right", mr),
        ("prev", mp),
        ("next", mn),
    ] {
        out.push_str(&format!(
            "<hc:{}{}/>",
            tag,
            attrs(&[("value", av(val)), ("unit", av("HWPUNIT"))])
        ));
    }
    out.push_str("</hh:margin>");
    out.push_str(&format!(
        "<hh:lineSpacing{}/>",
        attrs(&[
            ("type", av(enum0(&LINESPACING_TYPE, lt))),
            ("value", av(lv)),
            ("unit", av("HWPUNIT")),
        ])
    ));
    out
}

/// `<hh:paraProperties>` 를 만들고 `ids.parapr_id[objid]` 를 채운다.
/// [`emit_borderfills`] 가 먼저 실행돼 있어야 한다.
pub fn emit_parapr(m: &Model, ids: &mut Ids) -> String {
    let items = sorted_items(m.table("pp"));
    let mut out = format!("<hh:paraProperties itemCnt=\"{}\">", items.len());
    for (i, (objid, v)) in items.into_iter().enumerate() {
        let hid = i as i64;
        ids.parapr_id.insert(objid.clone(), hid);

        let ht = geti(v, "ht", 0);
        // ht=3(BULLET) 이면 bu 표, 1/2(OUTLINE/NUMBER) 이면 nu 표를 가리킨다
        let head_ref = ref_id(
            m,
            if ht == 3 {
                &["bu", "nu"]
            } else {
                &["nu", "bu"]
            },
            gets(v, "hi"),
            0,
        );
        let tab_ref = ref_id(m, &["tp"], gets(v, "tp"), 0);
        let bf_ref = ids.borderfill_id.get(gets(v, "bf")).copied().unwrap_or(0);

        let (mi, ml) = (geti(v, "mi", 0), geti(v, "ml", 0));
        let (mr, mp, mn) = (geti(v, "mr", 0), geti(v, "mp", 0), geti(v, "mn", 0));
        let (lt, lv) = (geti(v, "lt", 0), geti(v, "lv", 0));

        out.push_str(&format!(
            "<hh:paraPr{}>",
            attrs(&[
                ("id", av(hid)),
                ("tabPrIDRef", av(tab_ref)),
                ("condense", av(geti(v, "co", 0))),
                ("fontLineHeight", av(b01(getb(v, "fl")))),
                ("snapToGrid", av(b01(getb(v, "st")))),
                ("suppressLineNumbers", av(b01(getb(v, "sl")))),
                // 🔴 클립보드 모델이 버리는 두 속성. 원본 139개 전부 아래 값이다.
                ("checked", av("0")),
                ("textDir", av("LTR")),
            ])
        ));
        out.push_str(&format!(
            "<hh:align{}/>",
            attrs(&[
                ("horizontal", av(enum0(&ALIGN_H, geti(v, "ah", 0)))),
                ("vertical", av(enum0(&ALIGN_V, geti(v, "av", 0)))),
            ])
        ));
        out.push_str(&format!(
            "<hh:heading{}/>",
            attrs(&[
                ("type", av(enum0(&HEADING_TYPE, ht))),
                ("idRef", av(head_ref)),
                ("level", av(geti(v, "hl", 0))),
            ])
        ));
        out.push_str(&format!(
            "<hh:breakSetting{}/>",
            attrs(&[
                ("breakLatinWord", av(enum0(&BREAK_LATIN, geti(v, "kb", 0)))),
                (
                    "breakNonLatinWord",
                    av(if getb(v, "kn") {
                        "KEEP_WORD"
                    } else {
                        "BREAK_WORD"
                    })
                ),
                ("widowOrphan", av(b01(getb(v, "ko")))),
                ("keepWithNext", av(b01(getb(v, "kk")))),
                ("keepLines", av(b01(getb(v, "kl")))),
                ("pageBreakBefore", av(b01(getb(v, "kp")))),
                ("lineWrap", av(enum0(&LINE_WRAP, geti(v, "kw", 0)))),
            ])
        ));
        out.push_str(&format!(
            "<hh:autoSpacing{}/>",
            attrs(&[
                ("eAsianEng", av(b01(getb(v, "ae")))),
                ("eAsianNum", av(b01(getb(v, "aa")))),
            ])
        ));
        // 🔴 두 갈래: case(HwpUnitChar) = 절반, default = 모델 값 그대로
        out.push_str("<hp:switch>");
        out.push_str(&format!(
            "<hp:case hp:required-namespace=\"{}\">",
            super::ctx::esc_attr(HWPUNITCHAR_NS)
        ));
        out.push_str(&margin_block(mi, ml, mr, mp, mn, lt, lv, true));
        out.push_str("</hp:case>");
        out.push_str("<hp:default>");
        out.push_str(&margin_block(mi, ml, mr, mp, mn, lt, lv, false));
        out.push_str("</hp:default>");
        out.push_str("</hp:switch>");
        out.push_str(&format!(
            "<hh:border{}/>",
            attrs(&[
                ("borderFillIDRef", av(bf_ref)),
                ("offsetLeft", av(geti(v, "bl", 0))),
                ("offsetRight", av(geti(v, "br", 0))),
                ("offsetTop", av(geti(v, "bt", 0))),
                ("offsetBottom", av(geti(v, "bb", 0))),
                ("connect", av(b01(getb(v, "bc")))),
                ("ignoreMargin", av(b01(getb(v, "bi")))),
            ])
        ));
        out.push_str("</hh:paraPr>");
    }
    out.push_str("</hh:paraProperties>");
    out
}
