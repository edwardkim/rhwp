//! `rhwp capabilities` JSON 파싱 — 명령 팔레트 폼 자동 생성의 원천.
//!
//! capabilities 는 도구 정의의 단일 출처다(자기서술 계약). 여기서는 팔레트가
//! 쓰는 최소 필드만 강타입으로 뽑고, 나머지는 원문 그대로 UI 에 넘긴다.
//! 명령이 늘어도 이 파일은 안 바뀌는 것이 목표다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubcommandSpec {
    pub name: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    /// `--json` 봉투 출력 지원 여부.
    #[serde(default)]
    pub json: bool,
    #[serde(default)]
    pub subcommands: Vec<SubcommandSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub commands: Vec<CommandSpec>,
}

/// capabilities stdout 을 파싱한다. 실패 사유는 사람이 읽을 문장으로.
pub fn parse(raw: &str) -> Result<Capabilities, String> {
    let caps: Capabilities =
        serde_json::from_str(raw).map_err(|e| format!("capabilities JSON 파싱 실패: {e}"))?;
    if caps.commands.is_empty() {
        return Err("capabilities 에 commands 목록이 없습니다".into());
    }
    Ok(caps)
}

/// 이 엔진이 특정 명령을 제공하는지 — 버전에 따라 있는 명령(layout-anomaly 등)의
/// 존재 판정. UI(main.js)가 같은 규칙을 쓰며, 여기 두는 이유는 규칙의 정본과 테스트.
#[allow(dead_code)]
pub fn has_command(caps: &Capabilities, name: &str) -> bool {
    caps.commands.iter().any(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 실측 v0.8.4 출력에서 발췌한 축소 표본 — 필드 이름은 실제 계약 그대로.
    const SAMPLE: &str = r#"{
        "version": "0.8.4",
        "schemaVersion": "1.0",
        "tool": "rhwp",
        "commands": [
            {"name":"info","summary":"문서 메타 표시","flags":["--json"],
             "json":true,"batch":true,"category":"query",
             "recordFields":["schemaVersion","source","format"]},
            {"name":"export-svg","summary":"페이지별 SVG 렌더",
             "flags":["-o","-p","--json"],"json":true,"category":"export"},
            {"name":"inspect","summary":"검사 3축","flags":["--json","--kind"],
             "json":true,"category":"query",
             "subcommands":[
                {"name":"hidden-text","summary":"은닉 텍스트 탐지"},
                {"name":"injection","summary":"주입 신호 신고"},
                {"name":"unicode","summary":"유니코드 기만 판정"}
             ]}
        ]
    }"#;

    #[test]
    fn 실측_표본이_파싱된다() {
        let caps = parse(SAMPLE).unwrap();
        assert_eq!(caps.version.as_deref(), Some("0.8.4"));
        assert_eq!(caps.commands.len(), 3);
        let svg = &caps.commands[1];
        assert_eq!(svg.name, "export-svg");
        assert_eq!(svg.flags, vec!["-o", "-p", "--json"]);
        assert!(svg.json);
        let inspect = &caps.commands[2];
        assert_eq!(inspect.subcommands.len(), 3);
        assert_eq!(inspect.subcommands[0].name, "hidden-text");
    }

    #[test]
    fn 모르는_필드는_무시하고_없는_필드는_기본값이다() {
        let caps = parse(SAMPLE).unwrap();
        // recordFields·batch 는 강타입에 없지만 파싱을 깨지 않아야 한다.
        let info = &caps.commands[0];
        assert!(info.subcommands.is_empty());
        assert_eq!(info.category.as_deref(), Some("query"));
    }

    #[test]
    fn 깨진_입력과_빈_명령_목록은_오류다() {
        assert!(parse("not json").is_err());
        assert!(parse(r#"{"version":"1","commands":[]}"#).is_err());
    }

    #[test]
    fn has_command_판정() {
        let caps = parse(SAMPLE).unwrap();
        assert!(has_command(&caps, "inspect"));
        assert!(!has_command(&caps, "layout-anomaly"));
    }
}
