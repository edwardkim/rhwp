//! 장면 픽스처 스키마 — JSON 한 장이 한 계약.
//!
//! 픽스처는 실제 HWP 가 아니다. `SceneSpec` 을 직렬화한 것이며,
//! 통합 시험이 디렉터리를 읽어 TraceBackend 산출물과 expected_trace 를
//! 맞댄다. 생성기는 `tools/render_backend/gen_m06f.py` 다.

use std::path::{Path, PathBuf};

use super::catalog::OpBounds;
use super::scenes::{SceneOp, SceneSpec};

/// 픽스처 JSON 한 장.
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureScene {
    /// 스키마 버전.
    pub schema: u32,
    /// 장면.
    pub scene: SceneSpec,
    /// 기대 추적 줄. `None` 이면 시험이 직접 계산한다.
    pub expected_trace: Option<Vec<String>>,
    /// 기대 replay kind 순서.
    pub expected_kinds: Vec<String>,
}

impl FixtureScene {
    /// 현재 스키마 버전.
    pub const SCHEMA: u32 = 1;

    /// `SceneSpec` 에서 만든다. expected_trace 는 비운다.
    pub fn from_scene(scene: SceneSpec) -> Self {
        let expected_kinds = scene
            .expected_replay_kinds()
            .into_iter()
            .map(str::to_string)
            .collect();
        Self {
            schema: Self::SCHEMA,
            scene,
            expected_trace: None,
            expected_kinds,
        }
    }

    /// JSON 객체로 직렬화한다. 외부 crate 없이 손으로 쓴다.
    pub fn to_json_value(&self) -> String {
        let mut ops = String::new();
        for (i, op) in self.scene.ops.iter().enumerate() {
            if i > 0 {
                ops.push(',');
            }
            ops.push_str(&format!(
                "{{\"kind\":{},\"x\":{},\"y\":{},\"w\":{},\"h\":{},\"text\":{},\"gradient\":{},\"image\":{}}}",
                json_string(&op.kind),
                json_f64(op.bounds.x),
                json_f64(op.bounds.y),
                json_f64(op.bounds.width),
                json_f64(op.bounds.height),
                match &op.text {
                    Some(t) => json_string(t),
                    None => "null".into(),
                },
                if op.gradient { "true" } else { "false" },
                if op.image { "true" } else { "false" },
            ));
        }
        let kinds: Vec<String> = self.expected_kinds.iter().map(|k| json_string(k)).collect();
        let trace = match &self.expected_trace {
            Some(lines) => {
                let parts: Vec<String> = lines.iter().map(|l| json_string(l)).collect();
                format!("[{}]", parts.join(","))
            }
            None => "null".into(),
        };
        format!(
            "{{\n  \"schema\": {},\n  \"id\": {},\n  \"width\": {},\n  \"height\": {},\n  \"contract\": {},\n  \"ops\": [{}],\n  \"expectedKinds\": [{}],\n  \"expectedTrace\": {}\n}}\n",
            self.schema,
            json_string(&self.scene.id),
            json_f64(self.scene.width),
            json_f64(self.scene.height),
            json_string(&self.scene.contract),
            ops,
            kinds.join(","),
            trace
        )
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_f64(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{:.1}", value)
    } else {
        format!("{}", value)
    }
}

/// 최소 JSON 객체 파서 — 픽스처 필드만 읽는다.
pub fn parse_fixture_json(text: &str) -> Result<FixtureScene, String> {
    let value = MiniJson::parse(text)?;
    let obj = value.as_object().ok_or("픽스처 루트는 객체여야 한다")?;
    let schema = obj.get_u32("schema")?;
    if schema != FixtureScene::SCHEMA {
        return Err(format!("지원하지 않는 schema {schema}"));
    }
    let id = obj.get_string("id")?;
    let width = obj.get_f64("width")?;
    let height = obj.get_f64("height")?;
    let contract = obj.get_string("contract")?;
    let ops_val = obj.get("ops").ok_or("ops 없음")?;
    let ops_arr = ops_val.as_array().ok_or("ops 는 배열")?;
    let mut ops = Vec::new();
    for item in ops_arr {
        let op = item.as_object().ok_or("op 는 객체")?;
        let mut scene_op = SceneOp {
            kind: op.get_string("kind")?,
            bounds: OpBounds {
                x: op.get_f64("x")?,
                y: op.get_f64("y")?,
                width: op.get_f64("w")?,
                height: op.get_f64("h")?,
            },
            text: op.get_string_opt("text")?,
            gradient: op.get_bool("gradient").unwrap_or(false),
            image: op.get_bool("image").unwrap_or(false),
        };
        if scene_op.kind.is_empty() {
            return Err("빈 kind".into());
        }
        let _ = &mut scene_op;
        ops.push(scene_op);
    }
    let kinds_val = obj.get("expectedKinds").ok_or("expectedKinds 없음")?;
    let kinds_arr = kinds_val.as_array().ok_or("expectedKinds 는 배열")?;
    let mut expected_kinds = Vec::new();
    for item in kinds_arr {
        expected_kinds.push(item.as_str().ok_or("kind 는 문자열")?.to_string());
    }
    let expected_trace = match obj.get("expectedTrace") {
        None | Some(MiniJson::Null) => None,
        Some(MiniJson::Array(lines)) => {
            let mut out = Vec::new();
            for line in lines {
                out.push(line.as_str().ok_or("trace 줄은 문자열")?.to_string());
            }
            Some(out)
        }
        Some(_) => return Err("expectedTrace 는 배열 또는 null".into()),
    };
    Ok(FixtureScene {
        schema,
        scene: SceneSpec {
            id,
            width,
            height,
            ops,
            contract,
        },
        expected_trace,
        expected_kinds,
    })
}

/// 픽스처 디렉터리. `CARGO_MANIFEST_DIR` 기준.
pub fn fixture_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .join("tests")
        .join("fixtures")
        .join("render_backend")
}

/// `scenes/` 아래 JSON 을 이름 순으로 읽는다.
pub fn load_scene_fixtures(manifest_dir: &Path) -> Result<Vec<(PathBuf, FixtureScene)>, String> {
    let dir = fixture_root(manifest_dir).join("scenes");
    if !dir.is_dir() {
        return Err(format!("픽스처 디렉터리가 없다: {}", dir.display()));
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let text =
            std::fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        let fixture =
            parse_fixture_json(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        out.push((path, fixture));
    }
    Ok(out)
}

/// 매니페스트 JSON (`manifest.json`) 의 최소 필드.
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureManifest {
    /// 스키마.
    pub schema: u32,
    /// 장면 수.
    pub scene_count: usize,
    /// 장면 id 목록.
    pub ids: Vec<String>,
}

/// 매니페스트를 읽는다.
pub fn load_manifest(manifest_dir: &Path) -> Result<FixtureManifest, String> {
    let path = fixture_root(manifest_dir).join("manifest.json");
    let text = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let value = MiniJson::parse(&text)?;
    let obj = value.as_object().ok_or("매니페스트 루트는 객체")?;
    let schema = obj.get_u32("schema")?;
    let scene_count = obj.get_u32("sceneCount")? as usize;
    let ids_val = obj.get("ids").ok_or("ids 없음")?;
    let ids_arr = ids_val.as_array().ok_or("ids 는 배열")?;
    let mut ids = Vec::new();
    for item in ids_arr {
        ids.push(item.as_str().ok_or("id 는 문자열")?.to_string());
    }
    if ids.len() != scene_count {
        return Err(format!("sceneCount {scene_count} != ids {}", ids.len()));
    }
    Ok(FixtureManifest {
        schema,
        scene_count,
        ids,
    })
}

#[derive(Debug, Clone, PartialEq)]
enum MiniJson {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<MiniJson>),
    Object(MiniObject),
}

#[derive(Debug, Clone, PartialEq)]
struct MiniObject {
    entries: Vec<(String, MiniJson)>,
}

impl MiniObject {
    fn get(&self, key: &str) -> Option<&MiniJson> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn get_string(&self, key: &str) -> Result<String, String> {
        match self.get(key) {
            Some(MiniJson::String(s)) => Ok(s.clone()),
            Some(other) => Err(format!("{key} 는 문자열이어야 한다: {other:?}")),
            None => Err(format!("{key} 없음")),
        }
    }

    fn get_string_opt(&self, key: &str) -> Result<Option<String>, String> {
        match self.get(key) {
            None | Some(MiniJson::Null) => Ok(None),
            Some(MiniJson::String(s)) => Ok(Some(s.clone())),
            Some(other) => Err(format!("{key} 는 문자열 또는 null: {other:?}")),
        }
    }

    fn get_f64(&self, key: &str) -> Result<f64, String> {
        match self.get(key) {
            Some(MiniJson::Number(n)) => Ok(*n),
            Some(other) => Err(format!("{key} 는 숫자여야 한다: {other:?}")),
            None => Err(format!("{key} 없음")),
        }
    }

    fn get_u32(&self, key: &str) -> Result<u32, String> {
        let n = self.get_f64(key)?;
        if n < 0.0 || n.fract() != 0.0 || n > u32::MAX as f64 {
            return Err(format!("{key} 는 u32 가 아니다: {n}"));
        }
        Ok(n as u32)
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(MiniJson::Bool(b)) => Some(*b),
            _ => None,
        }
    }
}

impl MiniJson {
    fn as_object(&self) -> Option<&MiniObject> {
        match self {
            MiniJson::Object(o) => Some(o),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[MiniJson]> {
        match self {
            MiniJson::Array(a) => Some(a),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            MiniJson::String(s) => Some(s),
            _ => None,
        }
    }

    fn parse(text: &str) -> Result<Self, String> {
        let mut p = Parser {
            chars: text.chars().collect(),
            i: 0,
        };
        let value = p.parse_value()?;
        p.skip_ws();
        if p.i != p.chars.len() {
            return Err("JSON 뒤에 잔여 문자가 있다".into());
        }
        Ok(value)
    }
}

struct Parser {
    chars: Vec<char>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += 1;
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.i += 1;
        }
    }

    fn parse_value(&mut self) -> Result<MiniJson, String> {
        self.skip_ws();
        match self.peek() {
            Some('n') => self.parse_lit("null", MiniJson::Null),
            Some('t') => self.parse_lit("true", MiniJson::Bool(true)),
            Some('f') => self.parse_lit("false", MiniJson::Bool(false)),
            Some('"') => Ok(MiniJson::String(self.parse_string()?)),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c == '-' || c.is_ascii_digit() => Ok(MiniJson::Number(self.parse_number()?)),
            Some(c) => Err(format!("예상치 못한 문자 {c}")),
            None => Err("입력이 끝났다".into()),
        }
    }

    fn parse_lit(&mut self, lit: &str, value: MiniJson) -> Result<MiniJson, String> {
        for expected in lit.chars() {
            match self.bump() {
                Some(c) if c == expected => {}
                _ => return Err(format!("{lit} 리터럴 불일치")),
            }
        }
        Ok(value)
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.bump() != Some('"') {
            return Err("문자열이 \" 로 시작하지 않는다".into());
        }
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('b') => out.push('\u{0008}'),
                    Some('f') => out.push('\u{000c}'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            hex.push(self.bump().ok_or("\\u 자리 부족")?);
                        }
                        let code = u32::from_str_radix(&hex, 16)
                            .map_err(|_| format!("잘못된 \\u{hex}"))?;
                        out.push(char::from_u32(code).ok_or("잘못된 유니코드")?);
                    }
                    Some(c) => return Err(format!("알 수 없는 이스케이프 \\{c}")),
                    None => return Err("문자열 이스케이프가 끝났다".into()),
                },
                Some(c) => out.push(c),
                None => return Err("문자열이 닫히지 않았다".into()),
            }
        }
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.i;
        if self.peek() == Some('-') {
            self.i += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.i += 1;
        }
        if self.peek() == Some('.') {
            self.i += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.i += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.i += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        let raw: String = self.chars[start..self.i].iter().collect();
        raw.parse::<f64>()
            .map_err(|_| format!("숫자 파싱 실패: {raw}"))
    }

    fn parse_array(&mut self) -> Result<MiniJson, String> {
        if self.bump() != Some('[') {
            return Err("[ 필요".into());
        }
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(']') {
            self.i += 1;
            return Ok(MiniJson::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => return Ok(MiniJson::Array(items)),
                _ => return Err("배열 , 또는 ] 필요".into()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<MiniJson, String> {
        if self.bump() != Some('{') {
            return Err("{{ 필요".into());
        }
        self.skip_ws();
        let mut entries = Vec::new();
        if self.peek() == Some('}') {
            self.i += 1;
            return Ok(MiniJson::Object(MiniObject { entries }));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(':') {
                return Err(": 필요".into());
            }
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some('}') => return Ok(MiniJson::Object(MiniObject { entries })),
                _ => return Err("객체 , 또는 }} 필요".into()),
            }
        }
    }
}
