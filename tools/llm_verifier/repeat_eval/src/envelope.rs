//! 기존 `--json` 봉투에서 점 경로로 필드를 읽는다.

use serde_json::Value;

/// 관측 값. 산문 점수가 아니다.
#[derive(Debug, Clone, PartialEq)]
pub enum Observed {
    Missing,
    Null,
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    Text(String),
    Seq,
    Map,
}

impl Observed {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::U64(v) => Some(*v as f64),
            Self::I64(v) => Some(*v as f64),
            Self::F64(v) => Some(*v),
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn as_display(&self) -> String {
        match self {
            Self::Missing => "missing".into(),
            Self::Null => "null".into(),
            Self::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Self::U64(v) => v.to_string(),
            Self::I64(v) => v.to_string(),
            Self::F64(v) => format_num(*v),
            Self::Text(s) => s.clone(),
            Self::Seq => "seq".into(),
            Self::Map => "map".into(),
        }
    }

    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(*b),
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    Self::U64(u)
                } else if let Some(i) = n.as_i64() {
                    Self::I64(i)
                } else if let Some(f) = n.as_f64() {
                    Self::F64(f)
                } else {
                    Self::Text(n.to_string())
                }
            }
            Value::String(s) => Self::Text(s.clone()),
            Value::Array(_) => Self::Seq,
            Value::Object(_) => Self::Map,
        }
    }
}

fn format_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{:.0}", v)
    } else {
        let s = format!("{:.10}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// `verify.identical` 같은 점 경로를 읽는다. 배열 인덱스는 없다.
pub fn read_path(envelope: &Value, path: &str) -> Observed {
    if path.is_empty() {
        return Observed::Missing;
    }
    let mut cur = envelope;
    for part in path.split('.') {
        match cur {
            Value::Object(map) => match map.get(part) {
                Some(next) => cur = next,
                None => return Observed::Missing,
            },
            _ => return Observed::Missing,
        }
    }
    Observed::from_value(cur)
}

/// 명령 가족별 실패 신호. 기존 봉투 키만 본다.
pub fn fail_signals(envelope: &Value) -> Vec<String> {
    let mut out = Vec::new();
    push_bool_fail(&mut out, envelope, "identical", false);
    push_bool_fail(&mut out, envelope, "verify.identical", false);
    push_bool_fail(&mut out, envelope, "hasSignal", true);
    push_bool_fail(&mut out, envelope, "regression", true);
    push_bool_fail(&mut out, envelope, "pageCountMismatch", true);
    push_bool_fail(&mut out, envelope, "untrustedContent", true);
    if let Observed::U64(n) = read_path(envelope, "diffCount") {
        if n > 0 {
            out.push("diffCount>0".into());
        }
    }
    if let Observed::U64(n) = read_path(envelope, "verify.diffCount") {
        if n > 0 {
            out.push("verify.diffCount>0".into());
        }
    }
    if let Observed::U64(n) = read_path(envelope, "failCount") {
        if n > 0 {
            out.push("failCount>0".into());
        }
    }
    if let Observed::U64(n) = read_path(envelope, "overflowCount") {
        if n > 0 {
            out.push("overflowCount>0".into());
        }
    }
    if let Observed::Text(s) = read_path(envelope, "verdict") {
        if s != "pass" {
            out.push(format!("verdict={s}"));
        }
    }
    if let Observed::Text(s) = read_path(envelope, "status") {
        if !s.eq_ignore_ascii_case("OK") {
            out.push(format!("status={s}"));
        }
    }
    if let Observed::Bool(false) = read_path(envelope, "reproduced") {
        out.push("reproduced=false".into());
    }
    if matches!(
        read_path(envelope, "invalid"),
        Observed::Seq | Observed::Map | Observed::Bool(true)
    ) {
        if !matches!(read_path(envelope, "invalid"), Observed::Seq)
            || array_nonempty(envelope, "invalid")
        {
            out.push("invalid".into());
        }
    }
    out
}

fn array_nonempty(envelope: &Value, path: &str) -> bool {
    let mut cur = envelope;
    for part in path.split('.') {
        match cur {
            Value::Object(map) => match map.get(part) {
                Some(next) => cur = next,
                None => return false,
            },
            _ => return false,
        }
    }
    match cur {
        Value::Array(a) => !a.is_empty(),
        _ => false,
    }
}

fn push_bool_fail(out: &mut Vec<String>, envelope: &Value, path: &str, bad: bool) {
    if read_path(envelope, path).as_bool() == Some(bad) {
        out.push(format!("{path}={bad}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dotted_path_reads_nested() {
        let env = json!({"verify": {"identical": true, "diffCount": 0}});
        assert_eq!(read_path(&env, "verify.identical"), Observed::Bool(true));
        assert_eq!(read_path(&env, "verify.diffCount"), Observed::U64(0));
        assert!(read_path(&env, "verify.missing").is_missing());
    }

    #[test]
    fn fail_signals_catch_verify_false() {
        let env = json!({"verify": {"identical": false, "diffCount": 2}});
        let s = fail_signals(&env);
        assert!(s.iter().any(|x| x == "verify.identical=false"));
        assert!(s.iter().any(|x| x == "verify.diffCount>0"));
    }
}
