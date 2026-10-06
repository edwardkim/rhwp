//! 클립보드 문서모델(hwpjson) 로더와 변환 공용 문맥.
//!
//! 파이썬 정본 `hj/conv/ctx.py` 를 옮긴 것이다. 모델은 GUID(24자리 16진 문자열)를 키로 하는
//! 표 여러 벌이고, 본문 문단은 배열이 아니라 `np`(next paragraph) **연결리스트**로 이어져 있다.
//! 표 안 문단도 같은 방식이라 순회 함수를 두 벌 둔다.
//!
//! 🔴 모델(불변)과 발급 id 표(가변)를 [`Model`]·[`Ids`] 두 구조체로 나눈 이유는 Rust 대여 규칙
//! 때문이다. 파이썬은 한 `Ctx` 에 둘을 같이 담고 `ctx.cs[...]` 를 읽으면서 `ctx.charpr_id` 를
//! 채우지만, Rust 에서 그러면 `&Value`(모델 참조)와 `&mut` 가 겹친다. 두 인자로 나눠 받으면
//! 서로 다른 필드라 동시에 빌릴 수 있다.

use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};

/// cp 항목의 언어 슬롯 순서. `f1`~`f7` / `t1`~`t7` 의 접미 번호가 이 순서와 같다(사양서 확정).
pub const LANGS: [&str; 7] = [
    "HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER",
];

/// `LANGS` 와 짝을 이루는 필드 접미 번호.
pub const LANG_SUFFIX: [&str; 7] = ["1", "2", "3", "4", "5", "6", "7"];

/// 이 변환기가 읽는 모델 표 이름. 없으면 빈 표를 끼워 넣어 이후 접근을 무조건 성공시킨다
/// (부분 모델이 와도 패닉하지 않게 하려는 것).
const OBJECT_TABLES: [&str; 11] = [
    "bf", "cp", "tp", "nu", "bu", "pp", "st", "mp", "ro", "sl", "cs",
];

/// 클립보드 문서모델 — 읽기 전용.
pub struct Model {
    d: Value,
}

impl Model {
    /// JSON 값을 받아 표 존재를 보장한 모델로 만든다.
    pub fn new(mut d: Value) -> Option<Self> {
        let root = d.as_object_mut()?;
        for name in OBJECT_TABLES {
            root.entry(name.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
        }
        root.entry("bi".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        root.entry("bidt".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        Some(Model { d })
    }

    /// 이름으로 표(객체)를 얻는다. `Model::new` 가 존재를 보장하므로 없는 이름만 빈 표가 된다.
    pub fn table(&self, name: &str) -> &Map<String, Value> {
        match self.d.get(name).and_then(Value::as_object) {
            Some(m) => m,
            None => empty_map(),
        }
    }

    /// 그림 목록 `bi` (배열 순서가 곧 `imageN` 번호 − 1).
    pub fn bi(&self) -> &[Value] {
        self.d
            .get("bi")
            .and_then(Value::as_array)
            .map_or(&[], |v| v)
    }

    /// 그림 원본 `bidt` — 저장소 이름 → 표준 base64 문자열.
    pub fn bidt(&self) -> &Map<String, Value> {
        self.table("bidt")
    }

    /// 본문 문단 id 를 `np` 연결리스트 순서로 (`ro.hp` 가 첫 문단).
    pub fn body_paragraph_ids(&self) -> Vec<String> {
        let ro = self.table("ro");
        let head = ro.get("hp").and_then(Value::as_str).unwrap_or("");
        chain(ro, head)
    }

    /// 표 칸·글상자 안 문단 id 를 `np` 연결리스트 순서로.
    pub fn sublist_paragraph_ids(&self, head: &str) -> Vec<String> {
        chain(self.table("sl"), head)
    }
}

/// `np` 연결리스트 순회. 순환 모델이 와도 멈추도록 방문 집합을 둔다(파이썬 정본과 같은 방어).
fn chain(pool: &Map<String, Value>, head: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut cur = head.to_string();
    while !cur.is_empty() {
        let Some(node) = pool.get(&cur) else { break };
        if seen.insert(cur.clone(), ()).is_some() {
            break;
        }
        out.push(cur.clone());
        cur = node
            .get("np")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }
    out
}

/// 변환 중 발급한 HWPX 정수 id 표 — 가변.
#[derive(Default)]
pub struct Ids {
    /// 언어별 글꼴 목록 `[(이름, 종류)]`. 위치가 곧 그 언어표 안의 id.
    pub fonts: Vec<Vec<(String, i64)>>,
    /// (언어 슬롯 번호, 글꼴 이름) → 그 언어표 안의 id.
    pub font_id: HashMap<(usize, String), usize>,
    /// cp objid → charPr id (0-base)
    pub charpr_id: BTreeMap<String, i64>,
    /// pp objid → paraPr id (0-base)
    pub parapr_id: BTreeMap<String, i64>,
    /// bf objid → borderFill id (🔴 1-base — 0 은 '참조 없음'으로 읽힐 위험이 있다)
    pub borderfill_id: BTreeMap<String, i64>,
    /// st objid → style id (0-base)
    pub style_id: BTreeMap<String, i64>,
    /// bidt 키(저장소 이름) → imageN 번호 (1-base)
    pub bin_id: BTreeMap<String, u16>,
}

/// objid → 정수 id. 이미 다른 단계가 채워 놨으면 그 값을, 아니면 표 크기로 새로 발급한다.
/// 빈 objid 는 `None` — 호출부가 속성 자체를 생략한다.
pub fn issue(table: &mut BTreeMap<String, i64>, objid: &str) -> Option<i64> {
    if objid.is_empty() {
        return None;
    }
    if let Some(v) = table.get(objid) {
        return Some(*v);
    }
    let nid = table.len() as i64;
    table.insert(objid.to_string(), nid);
    Some(nid)
}

// ---------------------------------------------------------------- 값 읽기

/// 빈 표 — `table()` 의 안전한 반환값.
fn empty_map() -> &'static Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(Map::new)
}

/// 파이썬 `dict.get(k, dflt)` + `int()`. bool 은 0/1, 실수는 0 방향 절단.
pub fn geti(v: &Value, k: &str, dflt: i64) -> i64 {
    match v.get(k) {
        None | Some(Value::Null) => dflt,
        Some(Value::Bool(b)) => i64::from(*b),
        Some(Value::Number(n)) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .unwrap_or(dflt),
        Some(Value::String(s)) => s.parse::<i64>().unwrap_or(dflt),
        _ => dflt,
    }
}

/// 파이썬 truthiness — 없음/null/false/0/빈 문자열/빈 배열·객체가 거짓.
pub fn getb(v: &Value, k: &str) -> bool {
    match v.get(k) {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// 문자열 필드. 없으면 빈 문자열.
pub fn gets<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

/// 객체 필드.
pub fn getobj<'a>(v: &'a Value, k: &str) -> Option<&'a Map<String, Value>> {
    v.get(k).and_then(Value::as_object)
}

/// 배열 필드.
pub fn getarr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k).and_then(Value::as_array).map_or(&[], |v| v)
}

/// 널 문서 조각 — `Value::Null` 정적 참조(없는 하위 객체를 빈 것처럼 다루려고).
pub fn nullv() -> &'static Value {
    static NULL: Value = Value::Null;
    &NULL
}

/// 하위 객체를 값으로 얻는다. 없으면 `Value::Null` — 이후 `geti/getb` 가 전부 기본값을 낸다.
pub fn sub<'a>(v: &'a Value, k: &str) -> &'a Value {
    match v.get(k) {
        Some(x) if x.is_object() => x,
        _ => nullv(),
    }
}

// ---------------------------------------------------------------- XML 도우미

/// 속성값 이스케이프 — `& < > "`. 한글 등 비ASCII 는 그대로 둔다(원본 HWPX 도 그렇다).
pub fn esc_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// 텍스트 노드 이스케이프 — `& < >` (따옴표는 그대로).
pub fn esc_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// `[(이름, 값)]` → ` k="v" k="v"`. 값이 `None` 인 항목은 통째로 건너뛴다.
pub fn attrs(pairs: &[(&str, Option<String>)]) -> String {
    let mut out = String::new();
    for (k, v) in pairs {
        if let Some(v) = v {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            out.push_str(&esc_attr(v));
            out.push('"');
        }
    }
    out
}

/// `attrs` 항목을 짧게 쓰기 위한 포장.
pub fn av<T: std::fmt::Display>(v: T) -> Option<String> {
    Some(v.to_string())
}

/// bool → HWPX 표기.
pub fn b01(v: bool) -> &'static str {
    if v {
        "1"
    } else {
        "0"
    }
}

/// JSON 은 부호 있는 int32, HWPX 는 부호 없는 uint32 로 적는다.
pub fn u32v(v: i64) -> u32 {
    v.rem_euclid(1i64 << 32) as u32
}

/// 열거 int → 이름. 표에 없으면 **표 0번**으로 떨어뜨린다(무효 XML 방지).
/// `fonts_charpr.py` / `parapr.py` 의 `_enum` 에 해당한다.
pub fn enum0(table: &[&'static str], v: i64) -> &'static str {
    if v < 0 {
        return table[0];
    }
    table.get(v as usize).copied().unwrap_or(table[0])
}

/// 열거 int → 이름. 표에 없으면 **숫자를 그대로** 적는다.
/// `body.py` 의 `_enum` 에 해당한다(미확정 열거값을 잃지 않으려는 정본 동작).
pub fn enum_raw(table: &[&'static str], v: i64) -> String {
    if v >= 0 {
        if let Some(s) = table.get(v as usize) {
            return (*s).to_string();
        }
    }
    v.to_string()
}

/// 모델 표를 내부 일련번호(`id`) 오름차순으로 — 같은 id 는 키 순으로 묶어 결정론을 보장한다.
pub fn sorted_items<'a>(t: &'a Map<String, Value>) -> Vec<(&'a String, &'a Value)> {
    let mut items: Vec<(&String, &Value)> = t.iter().collect();
    items.sort_by(|a, b| {
        geti(a.1, "id", 0)
            .cmp(&geti(b.1, "id", 0))
            .then_with(|| a.0.cmp(b.0))
    });
    items
}
