//! 조회 CLI의 명령별 도움말. 파일을 열거나 조회 handler를 실행하지 않는다.

use super::help_catalog::{self, Description, Inventory};
use std::fmt::Write;

struct CommandHelp {
    command: String,
    group: &'static str,
    usage: String,
    summary: String,
    detail: String,
    tool: String,
}

/// 값으로 전달한 `--q --help` 등을 도움말 요청으로 오인하지 않는다.
fn asks_help(args: &[String]) -> bool {
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        if matches!(arg.as_str(), "--help" | "-h") {
            return true;
        }
        index += if option_takes_value(arg) { 2 } else { 1 };
    }
    false
}

fn option_takes_value(name: &str) -> bool {
    matches!(
        name,
        "--page"
            | "--section"
            | "--list"
            | "--para"
            | "--pos"
            | "--offset"
            | "--ci"
            | "--cell"
            | "--cell-para"
            | "--index"
            | "--id"
            | "--chart"
            | "--slot"
            | "--x"
            | "--y"
            | "--key"
            | "--q"
            | "--limit"
            | "--max-characters"
            | "--apply-to"
            | "--note-para"
            | "--mode"
            | "--profile"
    ) || matches!(
        name,
        "--start-para" | "--start-pos" | "--end-para" | "--end-pos"
    )
}

pub(super) fn render(args: &[String]) -> Option<String> {
    let head = args.first()?.as_str();
    let grouped = matches!(head, "kit" | "more" | "pack");
    if grouped && matches!(args.get(1)?.as_str(), "--help" | "-h") {
        return Some(group_help(head));
    }
    let spec = if grouped {
        let child = args.get(1)?.as_str();
        if !asks_help(&args[2..]) {
            return None;
        }
        grouped_spec(head, child)?
    } else {
        let Description {
            usage,
            summary,
            detail,
        } = help_catalog::single(head)?;
        if !asks_help(&args[1..]) {
            return None;
        }
        CommandHelp {
            command: head.into(),
            group: "query",
            usage: usage.into(),
            summary: summary.into(),
            detail: detail.into(),
            tool: format!("rhwp-q-{head}"),
        }
    };
    Some(command_help(&spec))
}

fn inventory(group: &str) -> &'static [Inventory] {
    match group {
        "more" => help_catalog::MORE,
        "pack" => help_catalog::PACK,
        _ => &[],
    }
}

fn grouped_spec(group: &str, child: &str) -> Option<CommandHelp> {
    let (usage, summary, detail) = if group == "kit" {
        let (_, usage, summary) = super::kit::help_specs().find(|(name, _, _)| *name == child)?;
        (usage.into(), summary.into(), kit_detail(child).into())
    } else if child == "volume-probe" {
        (format!("rhwp-q {group} volume-probe <파일> --slot <0-49> [--json]"),
            "슬롯별 문서 순회 진단값 조회".into(),
            format!("{group} 그룹의 진단 계산값 acc를 반환한다. 같은 slot도 more/pack의 계산 의미가 다르다. acc는 개체 수나 문서 품질 점수가 아니다."))
    } else {
        let item = inventory(group).iter().find(|item| item.name == child)?;
        (format!("rhwp-q {group} {child} <파일> [--json]"), item.summary.into(),
            format!("JSON은 count와 items를 반환한다. items의 데이터 필드: {}. {}\n  현재 조회는 구역의 본문 문단을 순회한다. 중첩 셀·머리말 내부까지 모두 재귀 순회하는 것으로 해석하지 않는다.\n  기본 텍스트 출력은 항목 수이며 상세 값은 --json으로 확인한다.", item.fields, item.note))
    };
    Some(CommandHelp {
        command: format!("{group} {child}"),
        group: match group {
            "kit" => "kit",
            "more" => "more",
            _ => "pack",
        },
        usage,
        summary,
        detail,
        tool: format!("rhwp-q-{group}"),
    })
}

fn kit_detail(name: &str) -> &'static str {
    match name {
        "chart-csv" => "--chart는 1부터 센다. chart-list로 확인한다. 기본 출력은 CSV, --json은 csv 필드를 가진 JSON이다. 계열마다 라벨/X 값이 달라 CSV로 안전하게 표현할 수 없으면 오류를 반환한다.",
        "source-image-bytes" => "page-images의 keys에서 실제 키를 가져온다. 기본은 present/mime/len 메타데이터다. --include-bytes를 지정하면 존재하는 원본의 bytesBase64도 반환한다. 페이지 PNG를 생성하지 않는다.",
        "bin-data" => "--index는 문서 바이너리 저장 목록의 0부터 세는 인덱스다. 기본은 present/len/id/extension 메타데이터다. --include-bytes는 존재하는 원본의 bytesBase64를 추가한다.",
        "chart-list" => "문서의 차트 목록을 확인한다. CSV 조회는 반환된 차트에 대응하는 1부터 세는 --chart 번호를 사용한다.",
        "search-all" => "--q 문자열을 전 문단에서 검색하고 대응 위치를 반환한다. 검색 문자열은 데이터로 취급한다.",
        "pictures" | "equations" | "hyperlinks" | "hidden-comments" | "index-marks" | "auto-numbers" | "ruby" | "page-hide" => "해당 종류의 항목을 조회한다. JSON count/items를 확인한다. 조회 대상이 없으면 빈 목록도 정상 결과다.",
        "cursor-in-cell" => "원본 구역·문단·컨트롤·셀·셀 문단·문자 오프셋을 함께 지정한다. 각 인덱스는 0부터 시작한다. 실제 문서의 개체 주소를 먼저 확인한다.",
        "cell-shape" | "field-info-at" | "cur-field-state" | "para-bounds" | "line-starts" | "word-starts" | "word-end" | "caret-stops" | "char-index" => "--list는 한글 커서 리스트 ID다. cursor-model로 확인하며 section 번호로 대체하지 않는다. --pos는 스트림 위치이고 문자 offset과 구분한다.",
        "page-border-fill" | "covering-pages" | "form-value" => "section은 원본 구역 번호이며 조판된 쪽 번호와 다르다. 구역·문단·컨트롤 주소를 확인한 뒤 조회한다.",
        "page-text" => "한글 스캔의 해당 쪽 텍스트를 조회한다. JSON text/chars에 텍스트와 문자 수를 반환한다. 첫 쪽은 --page 0이다.",
        _ => "선택한 조회 결과를 반환한다. --page는 0부터 시작한다. 좌표를 받는 명령의 x/y는 쪽 내부의 기본 96 DPI px 좌표다. 반환값은 JSON의 기능별 필드로 확인한다.",
    }
}

fn option_description(name: &str) -> &'static str {
    match name {
        "--json" => "stdout에 JSON 객체 출력; 기본은 명령별 텍스트",
        "--page" => "0부터 세는 쪽 번호; 0은 첫 쪽",
        "--section" => "0부터 세는 원본 구역 번호; 쪽 번호와 다름",
        "--list" => "한글 커서 리스트 ID; cursor-model로 확인",
        "--para" => "해당 구역 또는 리스트 안의 문단 번호; 0부터 시작",
        "--pos" => "한글 커서 스트림 위치; 문자 offset과 구분",
        "--offset" => "문자 오프셋; 0부터 시작",
        "--ci" => "문단 내부 컨트롤 인덱스; 0부터 시작",
        "--cell" => "셀 인덱스; 0부터 시작",
        "--cell-para" => "셀 내부 문단 인덱스; 0부터 시작",
        "--start-para" | "--end-para" => "선택 시작/끝의 각주 내부 문단 번호; 0부터, 기본 0",
        "--start-pos" | "--end-pos" => "선택 시작/끝 위치; 0부터, 기본 0",
        "--note-para" => "각주·미주 내부 문단 인덱스; 0부터 시작",
        "--header|--footer" => "--header(머리말) 또는 --footer(꼬리말) 중 하나만 지정",
        "--apply-to" => "적용 쪽; 0 양쪽 · 1 짝수 · 2 홀수",
        "--index" => "조회 대상 인덱스; 0부터 시작",
        "--id" => "조회할 필드의 실제 ID",
        "--chart" => "1부터 세는 차트 번호; chart-list로 확인",
        "--slot" => "진단 슬롯 번호; 0~49",
        "--x" | "--y" => "쪽 내부의 96 DPI px 좌표; 실수 허용",
        "--key" => "page-images로 얻은 원본 그림 키",
        "--q" => "검색 문자열; 공백이 있으면 인용",
        "--limit" => "최대 스캔 항목 수; 1 이상, 기본 제한 없음",
        "--max-characters" => "진단 문자 제한; 1~4096, 기본 1024",
        "--cp949" => "CP949 밖 문자를 &#N; 참조로 표시; 출력 인코딩 변환 아님",
        "--include-bytes" => "원본이 있으면 bytesBase64 추가; 기본은 메타데이터만",
        "--ignore-case" => "검색에서 대소문자 구분 안 함; 기본은 구분",
        "--include-cells" => "검색에 셀 내용 포함; 기본은 제외",
        "--mode" => "CanvasKit replay mode; default 또는 compat, 기본 default",
        "--profile" => "screen/print/high-quality/fast-preview; 기본 screen",
        _ => "명령별 조회 옵션",
    }
}

fn command_help(spec: &CommandHelp) -> String {
    let mut text = format!("명령: {}\n그룹: {}\n\n사용법:\n  {}\n\n설명:\n  {}\n\n인자:\n  <파일>  조회할 HWP/HWPX 파일 (필수; 공백이 있으면 인용)\n\n옵션:\n  -h, --help  도움말 표시\n",
        spec.command, spec.group, spec.usage.trim_start_matches("사용법: "), spec.summary);
    let mut tokens = spec.usage.split_whitespace().peekable();
    let mut example = format!("rhwp-q {} \"입력.hwp\"", spec.command);
    while let Some(token) = tokens.next() {
        let flag = token.trim_matches(['[', ']']);
        if !flag.starts_with("--") {
            continue;
        }
        let optional = token.starts_with('[');
        let value = if option_takes_value(flag) {
            format!(" {}", tokens.next().unwrap().trim_end_matches(']'))
        } else {
            String::new()
        };
        let description = if spec.command == "kit cursor-in-hf" && flag == "--page" {
            "대표 쪽 힌트; 0부터 시작, 생략하면 자동 선택"
        } else {
            option_description(flag)
        };
        let _ = writeln!(
            text,
            "  {flag}{value}  {description} ({})",
            if optional { "선택" } else { "필수" }
        );
        if !optional {
            if flag == "--header|--footer" {
                example.push_str(" --header");
                continue;
            }
            let example_value = match flag {
                "--chart" => "1",
                "--x" | "--y" => "100",
                "--key" => "\"조회한_그림키\"",
                "--q" => "\"검색할 문자열\"",
                _ => "0",
            };
            let _ = write!(example, " {flag} {example_value}");
        }
    }
    let _ = write!(text, "\n예시:\n  rhwp-q {} --help\n  {example} --json\n\n세부 설명:\n  {}\n  예시의 입력.hwp와 주소·키·ID는 실제 문서에서 확인한 값으로 바꾼다.\n  문서를 편집하거나 저장하지 않는다.\n\n출력:\n  --json 성공 시 stdout은 JSON, 오류 메시지는 stderr다.\n  JSON tool 식별자: {} (기존 소비자 호환용)\n\n종료 코드:\n  0 성공 · 1 실행 오류 · 2 사용법 오류\n\n(전체 도움말: rhwp-q --help · 매뉴얼: mydocs/manual/rhwp_query_cli.md)\n", spec.command, spec.detail, spec.tool);
    text
}

fn group_help(group: &str) -> String {
    let mut text = format!("명령: {group}\n그룹: query\n\n사용법:\n  rhwp-q {group} <명령> <파일> [옵션]\n\n설명:\n  {group} 그룹의 문서 조회 명령\n\n옵션:\n  -h, --help  도움말 표시\n  --json      각 조회 명령 뒤에 지정하여 JSON 출력\n\n예시:\n  rhwp-q {group} --help\n  rhwp-q {group} {} --help\n\n하위 명령:\n", match group { "kit" => "pictures", "more" => "para-empty", _ => "tables-model" });
    if group == "kit" {
        for (_, usage, summary) in super::kit::help_specs() {
            let _ = writeln!(text, "  {usage}\n      {summary}");
        }
    } else {
        for item in inventory(group) {
            let _ = writeln!(
                text,
                "  rhwp-q {group} {} <파일> [--json]\n      {}",
                item.name, item.summary
            );
        }
        let _ = writeln!(text, "  rhwp-q {group} volume-probe <파일> --slot <0-49> [--json]\n      슬롯별 문서 순회 진단값");
    }
    text.push_str("\n세부 설명:\n  하위 명령 뒤 --help로 필수 인자·옵션·출력·예시를 확인한다.\n  종료 코드: 0 성공 · 1 실행 오류 · 2 사용법 오류\n");
    text
}
