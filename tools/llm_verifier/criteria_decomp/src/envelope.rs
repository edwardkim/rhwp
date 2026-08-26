//! 기존 `--json` 봉투에서 허용된 필드만 읽는다.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Observed {
    Missing,
    Null,
    Bool(bool),
    U64(u64),
    I64(i64),
    Text(String),
    Seq(Vec<Value>),
    Map,
}

impl Observed {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn seq_len(&self) -> Option<usize> {
        match self {
            Self::Seq(v) => Some(v.len()),
            _ => None,
        }
    }
}

/// `verify.identical` 같은 점 경로를 허용된 키에 한해 따라간다.
pub fn read_field(envelope: &Value, field: &str) -> Observed {
    let mut cur = envelope;
    for part in field.split('.') {
        match cur {
            Value::Object(map) => match map.get(part) {
                Some(next) => cur = next,
                None => return Observed::Missing,
            },
            _ => return Observed::Missing,
        }
    }
    from_value(cur)
}

fn from_value(v: &Value) -> Observed {
    match v {
        Value::Null => Observed::Null,
        Value::Bool(b) => Observed::Bool(*b),
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Observed::U64(u)
            } else if let Some(i) = n.as_i64() {
                Observed::I64(i)
            } else {
                Observed::Text(n.to_string())
            }
        }
        Value::String(s) => Observed::Text(s.clone()),
        Value::Array(a) => Observed::Seq(a.clone()),
        Value::Object(_) => Observed::Map,
    }
}

/// 배열 길이 별칭 (`matches` → `matchCount`) 을 기존 키에서만 채운다.
pub fn read_named(envelope: &Value, field: &str) -> Observed {
    match field {
        "matchCount" => match envelope.get("matches") {
            Some(Value::Array(a)) => Observed::U64(a.len() as u64),
            Some(_) => read_field(envelope, field),
            None => read_field(envelope, field),
        },
        "itemCount" => match envelope.get("items") {
            Some(Value::Array(a)) => Observed::U64(a.len() as u64),
            Some(_) => read_field(envelope, field),
            None => read_field(envelope, field),
        },
        _ => read_field(envelope, field),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nested_verify_identical() {
        let env = json!({"verify": {"identical": false, "diffCount": 3}});
        assert_eq!(read_field(&env, "verify.identical"), Observed::Bool(false));
        assert_eq!(read_field(&env, "verify.diffCount"), Observed::U64(3));
        assert_eq!(read_field(&env, "identical"), Observed::Missing);
    }

    #[test]
    fn match_count_alias_reads_matches_len() {
        let env = json!({"matches": [{"text": "가"}, {"text": "나"}]});
        assert_eq!(read_named(&env, "matchCount"), Observed::U64(2));
    }
}
