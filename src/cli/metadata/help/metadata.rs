//! 사람용 help의 구조화된 정본 메타데이터.
//!
//! `capabilities`의 명령 선언과 catalog 가시성을 이 형식으로 한 번만 투영한다.
//! 출력기는 이 구조만 보고 사용법·옵션·예시와 root 그룹을 결정한다. 기존 긴 안내문은
//! 이 구조가 식별한 명령에 붙는 보조 설명일 뿐, help 경로나 완전성을 결정하지 않는다.

#[derive(Clone, Debug)]
pub(crate) struct SubcommandSpec {
    pub(crate) name: String,
    pub(crate) summary: String,
}

#[derive(Clone, Debug)]
pub(crate) struct HelpSpec {
    pub(crate) command: String,
    /// 최상위는 `public` 또는 `internal`, 하위 명령은 부모 명령 이름이다.
    pub(crate) group: String,
    pub(crate) usage: String,
    pub(crate) summary: String,
    pub(crate) options: Vec<String>,
    pub(crate) examples: Vec<String>,
    pub(crate) subcommands: Vec<SubcommandSpec>,
}

fn capabilities_commands() -> Vec<serde_json::Value> {
    crate::cli::metadata::capabilities::capabilities_value()["commands"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn declared_options(entry: &serde_json::Value) -> Vec<String> {
    entry["flags"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|flag| flag.as_str().map(str::to_string))
        .collect()
}

/// 하위 명령 선언이 현재 가진 한 줄 요약에서 명시 플래그를 보존한다.
///
/// 기존 capabilities 하위 선언은 아직 별도 flags 배열을 갖지 않는다. 다만 요약에
/// 이미 약속한 `--flag`를 버리지 않아야 하므로 구조화된 options에 옮긴다. 그 외
/// 상세 제약은 명령별 보조 설명에 그대로 남는다.
fn summary_options(summary: &str) -> Vec<String> {
    let mut options = Vec::new();
    for token in summary.split_whitespace() {
        let option =
            token.trim_matches(|ch: char| matches!(ch, '[' | ']' | '(' | ')' | ',' | '.' | '—'));
        if option.starts_with("--") && !options.iter().any(|seen| seen == option) {
            options.push(option.to_string());
        }
    }
    options
}

fn subcommands(entry: &serde_json::Value) -> Vec<SubcommandSpec> {
    let mut subcommands: Vec<SubcommandSpec> = entry["subcommands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|sub| {
            Some(SubcommandSpec {
                name: sub["name"].as_str()?.to_string(),
                summary: sub["summary"].as_str()?.to_string(),
            })
        })
        .collect();
    subcommands.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    subcommands
}

fn top_level_spec(entry: &serde_json::Value) -> Option<HelpSpec> {
    let command = entry["name"].as_str()?.to_string();
    let group = crate::cli::catalog::find(&command)?
        .help_group()?
        .to_string();
    let summary = entry["summary"].as_str()?.to_string();
    let subcommands = subcommands(entry);
    let usage = if subcommands.is_empty() {
        format!("rhwp {command} [옵션]")
    } else {
        format!("rhwp {command} <하위명령> [옵션]")
    };
    let examples = if subcommands.is_empty() {
        vec![
            format!("rhwp {command} --help"),
            format!("rhwp capabilities --search {command}"),
        ]
    } else {
        vec![
            format!("rhwp {command} --help"),
            format!("rhwp {command} <하위명령> --help"),
        ]
    };
    Some(HelpSpec {
        command,
        group,
        usage,
        summary,
        options: declared_options(entry),
        examples,
        subcommands,
    })
}

fn subcommand_spec(parent: &serde_json::Value, sub: &serde_json::Value) -> Option<HelpSpec> {
    let parent_name = parent["name"].as_str()?;
    let command = sub["name"].as_str()?.to_string();
    let summary = sub["summary"].as_str()?.to_string();
    Some(HelpSpec {
        command: command.clone(),
        group: parent_name.to_string(),
        usage: format!("rhwp {parent_name} {command} [옵션]"),
        summary: summary.clone(),
        options: summary_options(&summary),
        examples: vec![
            format!("rhwp {parent_name} {command} --help"),
            format!("rhwp capabilities --search {command}"),
        ],
        subcommands: Vec::new(),
    })
}

/// 모든 capabilities 최상위 명령을 root help 표시 그룹별로 이름순 정렬한다.
pub(crate) fn root_command_index() -> (Vec<HelpSpec>, Vec<HelpSpec>) {
    let mut public = Vec::new();
    let mut internal = Vec::new();
    for entry in capabilities_commands() {
        let Some(spec) = top_level_spec(&entry) else {
            continue;
        };
        match spec.group.as_str() {
            "public" => public.push(spec),
            "internal" => internal.push(spec),
            _ => unreachable!("catalog 밖 help 그룹"),
        }
    }
    public.sort_unstable_by(|left, right| left.command.cmp(&right.command));
    internal.sort_unstable_by(|left, right| left.command.cmp(&right.command));
    (public, internal)
}

pub(crate) fn is_declared_subcommand(head: &str, sub: &str) -> bool {
    capabilities_commands().iter().any(|entry| {
        entry["name"].as_str() == Some(head)
            && entry["subcommands"]
                .as_array()
                .is_some_and(|subs| subs.iter().any(|entry| entry["name"].as_str() == Some(sub)))
    })
}

/// `head`와 선택 `sub`의 help 정본을 반환한다.
pub(crate) fn command_help(head: &str, sub: Option<&str>) -> Option<HelpSpec> {
    let commands = capabilities_commands();
    let parent = commands
        .iter()
        .find(|entry| entry["name"].as_str() == Some(head))?;
    match sub {
        None => top_level_spec(parent),
        Some(want) => parent["subcommands"]
            .as_array()?
            .iter()
            .find(|entry| entry["name"].as_str() == Some(want))
            .and_then(|entry| subcommand_spec(parent, entry)),
    }
}
