//! 사람용 `--help` 출력의 순서 보존 조립 경계.

mod diagnostics;
mod edit;
mod protocol;
mod public;
mod sink;

use sink::println;

pub(crate) fn print_help() {
    println!("rhwp v{} - HWP 파일 뷰어", rhwp::version());
    println!();
    println!("사용법: rhwp <명령> [옵션]");
    println!("       rhwp <명령> [<하위명령>] --help    해당 명령의 상세 안내");
    println!();
    println!("전역 옵션 (일반 HWP5 열기·내보내기·변환 명령):");
    println!("      --password <pw>         EncryptVersion 4 암호 문서 열기");
    println!("      --password-stdin        표준 입력 첫 줄에서 비밀번호 읽기 (권장)");
    println!("                              --password 값은 프로세스 목록에 노출될 수 있음");
    println!();
    println!("명령 (이름순, 상세는 rhwp <명령> --help):");
    for (name, summary) in root_command_index() {
        println!("  {name:<28} {summary}");
    }
    println!();
    println!("계층형 명령:");
    println!("  rhwp edit --help            편집 하위 명령의 이름순 index");
    println!("  rhwp inspect --help         검사 하위 명령의 이름순 index");
    println!();
    println!("자기서술 JSON: rhwp capabilities");
    println!("옵션:");
    println!("  -h, --help      도움말 표시");
    println!("  -V, --version   버전 표시");
}

/// capabilities 선언의 dispatcher 순서는 did-you-mean 동률 해소 계약이므로 바꾸지 않고,
/// 사람용 root index에서만 이름순으로 정렬한다.
fn root_command_index() -> Vec<(String, String)> {
    let caps = crate::cli::metadata::capabilities::capabilities_value();
    let visible: std::collections::BTreeSet<&str> = crate::cli::catalog::commands()
        .iter()
        .filter(|command| command.in_help())
        .map(|command| command.name)
        .collect();
    let mut commands: Vec<(String, String)> = caps["commands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry["name"].as_str()?;
            if !visible.contains(name) {
                return None;
            }
            Some((name.to_string(), entry["summary"].as_str()?.to_string()))
        })
        .collect();
    commands.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    commands
}

/// 통짜 도움말을 한 번 흘려 `head`(+`sub`) 절만 모은다.
fn section_lines(head: &str, sub: Option<&str>) -> Vec<String> {
    sink::collect(head, sub, || {
        public::print();
        edit::print();
        protocol::print();
        diagnostics::print();
    })
}

/// 절이 아직 없는 명령의 최소 답 — `capabilities` 자기서술에서만 만든다.
///
/// 없는 사용법을 지어내지 않는다. 요약·선언 플래그·JSON 자기서술 경로까지만 낸다.
fn fallback_lines(head: &str, sub: Option<&str>) -> Option<Vec<String>> {
    let caps = crate::cli::metadata::capabilities::capabilities_value();
    let entry = caps["commands"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c["name"].as_str() == Some(head)))?;
    let mut out = Vec::new();
    match sub {
        Some(want) => {
            let s = entry["subcommands"]
                .as_array()
                .and_then(|a| a.iter().find(|s| s["name"].as_str() == Some(want)))?;
            out.push(format!("  {head} {want}"));
            if let Some(summary) = s["summary"].as_str() {
                out.push(format!("      {summary}"));
            }
            out.push(String::new());
            out.push(format!(
                "      상세 절은 아직 없다. 부모 명령 전체: rhwp {head} --help"
            ));
        }
        None => {
            out.push(format!("  {head}"));
            if let Some(summary) = entry["summary"].as_str() {
                out.push(format!("      {summary}"));
            }
            if let Some(flags) = entry["flags"].as_array() {
                let list: Vec<&str> = flags.iter().filter_map(|f| f.as_str()).collect();
                if !list.is_empty() {
                    out.push(String::new());
                    out.push(format!("      선언 옵션: {}", list.join(" ")));
                }
            }
        }
    }
    Some(out)
}

/// 하위 명령을 가진 명령의 **목차** — 이름 + 한 줄 요약(자기서술 그대로).
///
/// 그룹을 물었을 때 절 전체(예: `edit` 603줄)를 쏟지 않는다. 알고 싶은 것은
/// "하위가 무엇이 있고 어느 것을 더 볼까"이고, 그다음 한 수가 `rhwp edit <하위> --help` 다.
fn group_index_lines(head: &str) -> Option<Vec<String>> {
    let caps = crate::cli::metadata::capabilities::capabilities_value();
    let entry = caps["commands"]
        .as_array()?
        .iter()
        .find(|c| c["name"].as_str() == Some(head))?;
    let mut subs: Vec<&serde_json::Value> = entry["subcommands"].as_array()?.iter().collect();
    if subs.is_empty() {
        return None;
    }
    subs.sort_unstable_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    let mut out = vec![format!(
        "  {head} <하위명령> [옵션]   (하위 {}종)",
        subs.len()
    )];
    if let Some(summary) = entry["summary"].as_str() {
        // 부모 요약은 하위 나열을 이미 담고 있어 길다. 첫 문장만 쓴다.
        let head_line = summary.split(" — ").next().unwrap_or(summary);
        out.push(format!("      {head_line}"));
    }
    out.push(String::new());
    out.push(format!("      하나만 보기: rhwp {head} <하위명령> --help"));
    out.push(String::new());
    for s in subs {
        let (Some(name), Some(summary)) = (s["name"].as_str(), s["summary"].as_str()) else {
            continue;
        };
        out.push(format!("      {name:<28} {summary}"));
    }
    Some(out)
}

/// `capabilities` 가 `head` 의 하위로 선언한 이름인가.
fn is_declared_subcommand(head: &str, sub: &str) -> bool {
    let caps = crate::cli::metadata::capabilities::capabilities_value();
    caps["commands"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c["name"].as_str() == Some(head)))
        .and_then(|c| c["subcommands"].as_array().cloned())
        .is_some_and(|subs| subs.iter().any(|s| s["name"].as_str() == Some(sub)))
}

/// [#5791] `rhwp <명령> [<하위명령>] --help` — 그 절만 stdout 으로 내고 exit 0.
///
/// `rest` 는 프로그램 이름을 뺀 인자다. 도움말을 물어본 호출이 아니거나 절도
/// 선언도 못 찾으면 `None` 을 돌려 **기존 경로를 그대로** 태운다.
///
/// `--help` 가 **값 자리**일 수 있다(`--find --help` 는 "--help" 를 찾는 치환이다).
/// 바로 앞 토큰이 플래그면 값으로 보고 가로채지 않는다.
pub(crate) fn scoped_help(rest: &[String]) -> Option<i32> {
    let head = rest.first()?.as_str();
    if head.starts_with('-') {
        return None; // `rhwp --help` 는 통짜 경로가 이미 처리한다.
    }
    let asked = rest
        .iter()
        .enumerate()
        .skip(1)
        .any(|(i, a)| (a == "--help" || a == "-h") && !rest[i - 1].starts_with('-'));
    if !asked {
        return None;
    }

    let sub = rest
        .get(1)
        .map(String::as_str)
        .filter(|s| !s.starts_with('-') && is_declared_subcommand(head, s));

    let mut lines = match sub {
        Some(_) => section_lines(head, sub),
        // 그룹 자신을 물었으면 목차가 답이다. 하위가 없는 명령은 자기 절.
        None => group_index_lines(head).unwrap_or_else(|| section_lines(head, None)),
    };
    if lines.is_empty() {
        lines = fallback_lines(head, sub)?;
    }
    if lines.is_empty() {
        return None;
    }

    for line in &lines {
        println!("{line}");
    }
    println!();
    match sub {
        Some(s) => {
            println!("(전체 도움말: rhwp --help · JSON 자기서술: rhwp capabilities --search {s})")
        }
        None => println!(
            "(전체 도움말: rhwp --help · JSON 자기서술: rhwp capabilities --search {head})"
        ),
    }
    Some(0)
}
