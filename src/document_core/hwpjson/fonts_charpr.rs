//! 글꼴표(`hh:fontfaces`) · 글자모양(`hh:charProperties`) XML 생성기.
//!
//! 파이썬 정본 `hj/conv/fonts_charpr.py` 를 옮긴 것이다. 값·배율은 원본 header.xml 과 1:1 대조로
//! 확정된 것만 쓴다(원본 대조 실적 charPr 200/202).
//!
//! 핵심 사실(사양서 확정)
//!   * 클립보드에는 독립 글꼴표가 없다. 글꼴은 cp 항목마다 `f1`~`f7`(이름)·`t1`~`t7`(종류)로
//!     인라인된다. 언어 순서는 [`LANGS`] 와 같다.
//!   * `he`(글자 크기)는 pt×100 = HWPUNIT, 배율 1:1. charProperties 안에는 `hp:switch/HwpUnitChar`
//!     가 0건이므로 '절반 표기' 예외가 없다.
//!   * `r`/`s`/`e`/`o`(장평·자간·상대크기·글자위치)는 전부 퍼센트 정수 1:1.
//!   * 색은 32비트 COLORREF `0x00BBGGRR`. `0xFFFFFFFF` 는 '없음' 센티널(`none`).
//!   * 굵게/기울임/양각/음각/위첨자/아래첨자는 자식 요소의 '유무'로 표현한다.

use serde_json::Value;

use super::ctx::{
    attrs, av, b01, enum0, getb, geti, gets, sorted_items, Ids, Model, LANGS, LANG_SUFFIX,
};

/// `hh:charPr/@symMark`
const SYM_MARK: [&str; 17] = [
    "NONE",
    "DOT_ABOVE",
    "RING_ABOVE",
    "TILDE",
    "CARON",
    "SIDE",
    "COLON",
    "GRAVE",
    "ACUTE",
    "CIRCUMFLEX",
    "MACRON",
    "HOOK_ABOVE",
    "DOT_BELOW",
    "COMMA_ABOVE",
    "REVERSED_COMMA_ABOVE",
    "DOUBLE_GRAVE",
    "BREVE",
];

/// `hh:underline/@type` — 밑줄 위치
const UNDERLINE_TYPE: [&str; 4] = ["NONE", "BOTTOM", "CENTER", "TOP"];

/// `hh:underline/@shape` · `hh:strikeout/@shape` — 선 모양(LineType).
/// 실측 확정은 1↔SOLID, 16↔3D 두 점. 사이 값은 OWPML LineType 순서 추정(사양서 medium).
const LINE_SHAPE: [&str; 18] = [
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
    "THICK_3D_REVERSE",
    "3D",
    "3D_REVERSE",
];

/// `hh:outline/@type`
const OUTLINE_TYPE: [&str; 8] = [
    "NONE",
    "SOLID",
    "DASH",
    "DOT",
    "DASH_DOT",
    "DASH_DOT_DOT",
    "LONG_DASH",
    "CIRCLE",
];

/// `hh:shadow/@type`
const SHADOW_TYPE: [&str; 3] = ["NONE", "DROP", "CONTINUOUS"];

/// 그림자가 꺼져 있을 때 클립보드는 색/오프셋을 0 으로 지운다. 원본 header.xml 은 234건 전부
/// `offsetX/offsetY="10"`, 233건이 `color="#C0C0C0"` 이므로 '전부 0' 상태에서는 기본값을 복원한다.
const SHADOW_DEFAULT_COLOR: &str = "#C0C0C0";
const SHADOW_DEFAULT_OFFSET: i64 = 10;

const COLOR_NONE: i64 = 0xFFFF_FFFF;

/// COLORREF(`0x00BBGGRR`) → HWPX 색 문자열.
///
/// 🔴 `body.rs` 의 색 변환과 다르다 — 이쪽은 상위 바이트가 살아 있으면 8자리를 그대로 적는다
/// (원본 header.xml 실측 `#FF000000`).
pub fn color(v: i64) -> String {
    let n = v & 0xFFFF_FFFF;
    if n == COLOR_NONE {
        return "none".to_string();
    }
    if n > 0xFF_FFFF {
        return format!("#{:08X}", n);
    }
    let (r, g, b) = (n & 0xFF, (n >> 8) & 0xFF, (n >> 16) & 0xFF);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// cp 항목을 모델 내부 순번(id) 오름차순으로.
fn cp_order(m: &Model) -> Vec<(&String, &Value)> {
    sorted_items(m.table("cp"))
}

// --------------------------------------------------------------------------
// ① 글꼴 수집
// --------------------------------------------------------------------------

/// cp 의 `f1`~`f7` / `t1`~`t7` 을 훑어 언어별 글꼴 목록을 만든다.
/// 등장 순서가 곧 그 언어표 안의 id 다(원본도 등장 순서).
pub fn collect_fonts(m: &Model, ids: &mut Ids) {
    ids.fonts = vec![Vec::new(); LANGS.len()];
    ids.font_id.clear();
    for (_key, c) in cp_order(m) {
        for (i, suf) in LANG_SUFFIX.iter().enumerate() {
            let name = gets(c, &format!("f{suf}")).to_string();
            let ftype = geti(c, &format!("t{suf}"), 1);
            if ids.font_id.contains_key(&(i, name.clone())) {
                continue;
            }
            ids.font_id.insert((i, name.clone()), ids.fonts[i].len());
            ids.fonts[i].push((name, ftype));
        }
    }
}

/// `<hh:fontfaces itemCnt="7"> … </hh:fontfaces>`
pub fn emit_fontfaces(ids: &Ids) -> String {
    let mut out = format!("<hh:fontfaces itemCnt=\"{}\">", LANGS.len());
    for (i, lang) in LANGS.iter().enumerate() {
        let list = ids.fonts.get(i).map_or(&[][..], |v| &v[..]);
        out.push_str(&format!(
            "<hh:fontface lang=\"{}\" fontCnt=\"{}\">",
            lang,
            list.len()
        ));
        for (idx, (name, ftype)) in list.iter().enumerate() {
            out.push_str(&format!(
                "<hh:font{}/>",
                attrs(&[
                    ("id", av(idx)),
                    ("face", av(name)),
                    // 글꼴 파일 종류: 1=TTF, 2=HFT
                    ("type", av(if *ftype == 2 { "HFT" } else { "TTF" })),
                    ("isEmbedded", av("0")),
                ])
            ));
        }
        out.push_str("</hh:fontface>");
    }
    out.push_str("</hh:fontfaces>");
    out
}

// --------------------------------------------------------------------------
// ② 글자모양
// --------------------------------------------------------------------------

/// `r`/`s`/`e`/`o` 처럼 언어 7개를 속성으로 펼치는 요소의 속성 목록.
fn lang_attrs(c: &Value, prefix: &str) -> String {
    let pairs: Vec<(String, Option<String>)> = LANGS
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            (
                lang.to_lowercase(),
                av(geti(c, &format!("{prefix}{}", LANG_SUFFIX[i]), 0)),
            )
        })
        .collect();
    let refs: Vec<(&str, Option<String>)> =
        pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    attrs(&refs)
}

fn fontref_attrs(ids: &Ids, c: &Value) -> String {
    let pairs: Vec<(String, Option<String>)> = LANGS
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            let name = gets(c, &format!("f{}", LANG_SUFFIX[i])).to_string();
            let fid = ids.font_id.get(&(i, name)).copied().unwrap_or(0);
            (lang.to_lowercase(), av(fid))
        })
        .collect();
    let refs: Vec<(&str, Option<String>)> =
        pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    attrs(&refs)
}

/// `cp.bf`(GUID) → borderFill 정수 id.
///
/// 정본은 `ids.borderfill_id`(테두리/배경 단계가 발급). 아직 비어 있으면 bf 표가 들고 있는
/// 1-base 순번으로 떨어뜨린다(원본도 1-base).
fn borderfill_ref(m: &Model, ids: &Ids, c: &Value) -> i64 {
    let key = gets(c, "bf");
    if let Some(v) = ids.borderfill_id.get(key) {
        return *v;
    }
    if let Some(item) = m.table("bf").get(key) {
        if item.get("id").is_some() {
            return geti(item, "id", 1);
        }
    }
    1
}

fn charpr_xml(m: &Model, ids: &Ids, cid: i64, c: &Value) -> String {
    let head = attrs(&[
        ("id", av(cid)),
        ("height", av(geti(c, "he", 1000))),
        ("textColor", av(color(geti(c, "tc", 0)))),
        ("shadeColor", av(color(geti(c, "sc", COLOR_NONE)))),
        ("useFontSpace", av(b01(getb(c, "uf")))),
        ("useKerning", av(b01(getb(c, "uk")))),
        ("symMark", av(enum0(&SYM_MARK, geti(c, "sm", 0)))),
        ("borderFillIDRef", av(borderfill_ref(m, ids, c))),
    ]);
    let mut p = format!("<hh:charPr{head}>");

    // 🔴 자식 순서는 원본 header.xml 그대로:
    //    fontRef, ratio, spacing, relSz, offset, [italic], [bold],
    //    underline, strikeout, outline, shadow, [emboss], [engrave],
    //    [supscript], [subscript]
    p.push_str(&format!("<hh:fontRef{}/>", fontref_attrs(ids, c)));
    p.push_str(&format!("<hh:ratio{}/>", lang_attrs(c, "r")));
    p.push_str(&format!("<hh:spacing{}/>", lang_attrs(c, "s")));
    p.push_str(&format!("<hh:relSz{}/>", lang_attrs(c, "e")));
    p.push_str(&format!("<hh:offset{}/>", lang_attrs(c, "o")));

    if getb(c, "it") {
        p.push_str("<hh:italic/>");
    }
    if getb(c, "bo") {
        p.push_str("<hh:bold/>");
    }

    p.push_str(&format!(
        "<hh:underline{}/>",
        attrs(&[
            ("type", av(enum0(&UNDERLINE_TYPE, geti(c, "ut", 0)))),
            ("shape", av(enum0(&LINE_SHAPE, geti(c, "us", 1)))),
            ("color", av(color(geti(c, "uc", 0)))),
        ])
    ));

    // 취소선: st 는 on/off 게이트. 꺼짐이면 HWPX 에 별도 속성이 없어 shape="NONE".
    let ss = if getb(c, "st") {
        enum0(&LINE_SHAPE, geti(c, "ss", 1))
    } else {
        "NONE"
    };
    p.push_str(&format!(
        "<hh:strikeout{}/>",
        attrs(&[("shape", av(ss)), ("color", av(color(geti(c, "so", 0)))),])
    ));

    p.push_str(&format!(
        "<hh:outline{}/>",
        attrs(&[("type", av(enum0(&OUTLINE_TYPE, geti(c, "ot", 0))))])
    ));

    let ht = geti(c, "ht", 0);
    let (hc, hx, hy) = (geti(c, "hc", 0), geti(c, "hx", 0), geti(c, "hy", 0));
    let (sh_color, sh_x, sh_y) = if ht == 0 && hc == 0 && hx == 0 && hy == 0 {
        // 클립보드가 지운 상태 → HWPX 기본값 복원
        (
            SHADOW_DEFAULT_COLOR.to_string(),
            SHADOW_DEFAULT_OFFSET,
            SHADOW_DEFAULT_OFFSET,
        )
    } else {
        (color(hc), hx, hy)
    };
    p.push_str(&format!(
        "<hh:shadow{}/>",
        attrs(&[
            ("type", av(enum0(&SHADOW_TYPE, ht))),
            ("color", av(sh_color)),
            ("offsetX", av(sh_x)),
            ("offsetY", av(sh_y)),
        ])
    ));

    if getb(c, "em") {
        p.push_str("<hh:emboss/>");
    }
    if getb(c, "en") {
        p.push_str("<hh:engrave/>");
    }
    if getb(c, "su") {
        p.push_str("<hh:supscript/>");
    }
    if getb(c, "sb") {
        p.push_str("<hh:subscript/>");
    }

    p.push_str("</hh:charPr>");
    p
}

/// `<hh:charProperties itemCnt="N"> … </hh:charProperties>`. `ids.charpr_id` 를 채운다.
pub fn emit_charpr(m: &Model, ids: &mut Ids) -> String {
    if ids.font_id.is_empty() {
        collect_fonts(m, ids);
    }
    let order: Vec<String> = cp_order(m).into_iter().map(|(k, _)| k.clone()).collect();
    ids.charpr_id.clear();
    for (idx, key) in order.iter().enumerate() {
        ids.charpr_id.insert(key.clone(), idx as i64);
    }
    let cp = m.table("cp");
    let mut body = String::new();
    for (idx, key) in order.iter().enumerate() {
        if let Some(c) = cp.get(key) {
            body.push_str(&charpr_xml(m, ids, idx as i64, c));
        }
    }
    format!(
        "<hh:charProperties itemCnt=\"{}\">{}</hh:charProperties>",
        order.len(),
        body
    )
}
