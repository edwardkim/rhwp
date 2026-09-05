//! 사람용 `--help` 출력의 순서 보존 조립 경계.

mod diagnostics;
mod edit;
mod metadata;
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
    let (public, internal) = metadata::root_command_index();
    println!("명령 (이름순, 상세는 rhwp <명령> --help):");
    for command in public {
        println!("  {:<28} {}", command.command, command.summary);
    }
    println!();
    println!("내부 개발·회귀 명령 (이름순, 상세는 rhwp <명령> --help):");
    for command in internal {
        println!("  {:<28} {}", command.command, command.summary);
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

/// 기존 긴 도움말에서 해당 절만 모은 **보조 설명**.
///
/// 명령의 존재·그룹·사용법·옵션·예시는 `metadata::HelpSpec`이 정본이다. 이 캡처는
/// 표·입출력 제약처럼 아직 서술형으로 유지한 세부 설명을 그 정본 아래에 붙일 뿐이다.
fn legacy_detail_lines(head: &str, sub: Option<&str>) -> Vec<String> {
    sink::collect(head, sub, || {
        public::print();
        edit::print();
        diagnostics::print();
    })
}

fn structured_lines(spec: &metadata::HelpSpec) -> Vec<String> {
    let mut out = vec![format!("명령: {}", spec.command)];
    out.push(format!("그룹: {}", spec.group));
    out.push(String::new());
    out.push("사용법:".to_string());
    out.push(format!("  {}", spec.usage));
    out.push(String::new());
    out.push("설명:".to_string());
    out.push(format!("  {}", spec.summary));
    out.push(String::new());
    out.push("옵션:".to_string());
    out.push("  -h, --help                   도움말 표시".to_string());
    if spec.options.is_empty() {
        out.push("  명령 선언에 추가 옵션 없음".to_string());
    } else {
        for option in &spec.options {
            out.push(format!("  {option}"));
        }
    }
    out.push(String::new());
    out.push("예시:".to_string());
    for example in &spec.examples {
        out.push(format!("  {example}"));
    }
    if !spec.subcommands.is_empty() {
        out.push(String::new());
        out.push(format!("하위 명령 (이름순, {}종):", spec.subcommands.len()));
        out.push(format!(
            "  하나만 보기: rhwp {} <하위명령> --help",
            spec.command
        ));
        out.push(String::new());
        for subcommand in &spec.subcommands {
            out.push(format!(
                "      {:<28} {}",
                subcommand.name, subcommand.summary
            ));
        }
    }
    out
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
        .filter(|s| !s.starts_with('-') && metadata::is_declared_subcommand(head, s));
    let spec = metadata::command_help(head, sub)?;
    let mut lines = structured_lines(&spec);

    // 그룹 자체의 짧은 index는 구조화된 메타데이터만으로 충분하다. 개별 명령과
    // 하위 명령은 기존의 긴 계약을 보조 설명으로 덧붙인다.
    if sub.is_some() || spec.subcommands.is_empty() {
        let detail = legacy_detail_lines(head, sub);
        if !detail.is_empty() {
            lines.push(String::new());
            lines.push("세부 설명:".to_string());
            lines.extend(detail);
        }
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
