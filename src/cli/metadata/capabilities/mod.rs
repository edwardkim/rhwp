//! `capabilities` 자기서술과 명령 검색 메타데이터.
//!
//! 공개 명령 배열은 원래 순서를 보존한 두 기능군에서 조립하고, catalog 대조와
//! 검색·출처 지도 출력은 이 모듈이 소유한다.

mod core;
mod extended;

fn capability_spec(name: &str) -> &'static crate::cli::catalog::CommandSpec {
    let spec = crate::cli::catalog::find(name)
        .unwrap_or_else(|| panic!("capabilities 명령이 catalog에 없습니다: {name}"));
    assert!(
        spec.in_capabilities(),
        "dispatch-only 명령을 capabilities에 노출할 수 없습니다: {name}"
    );
    spec
}

fn cmd(name: &str, summary: &str) -> serde_json::Value {
    let spec = capability_spec(name);
    assert!(!spec.json_contract, "{name}: cmd_json을 사용해야 합니다");
    serde_json::json!({
        "name": name,
        "category": spec.category.as_str(),
        "summary": summary,
    })
}

fn cmd_json(
    name: &str,
    summary: &str,
    flags: &[&str],
    record_fields: &[&str],
) -> serde_json::Value {
    let spec = capability_spec(name);
    assert!(
        spec.json_contract,
        "{name}: catalog JSON 계약이 false입니다"
    );
    serde_json::json!({
        "name": name, "category": spec.category.as_str(), "summary": summary,
        "json": true, "batch": spec.batch, "flags": flags, "recordFields": record_fields,
    })
}

fn cmd_gated(name: &str, summary: &str, available: bool) -> serde_json::Value {
    let spec = capability_spec(name);
    let requires_feature = spec
        .requires_feature
        .unwrap_or_else(|| panic!("{name}: catalog feature 계약이 없습니다"));
    serde_json::json!({
        "name": name, "category": spec.category.as_str(), "summary": summary,
        "requiresFeature": requires_feature, "available": available,
    })
}

/// [#3884 G4] edit·inspect 하위 명령의 자기서술 등재 — 이름 + 요약 한 줄.
///
/// 부모 항목의 summary 산문에만 있던 하위 명령을 데이터로 낸다. `capabilities` 만
/// 읽는 에이전트가 `--search redact` 로 edit 하위를 찾게 하는 것이 목적이다
/// (`batch.subcommands` 선례를 commands[] 항목으로 옮긴 모양 — 1차는 이름·요약만,
/// 하위별 recordFields 분화는 별도 판단). 선언 ↔ 디스패치 실물의 대조는
/// `tests/capabilities_subcommands_contract.rs` 가 USAGE 문자열과 실행 거동으로 잡는다.
const EDIT_SUBCOMMANDS: [(&str, &str); 112] = [
    (
        "fill-fields",
        "누름틀(필드) 값 채우기 — --data 이름=값, 같은 이름은 [k] 순번 지목",
    ),
    (
        "replace-text",
        "본문 일괄 치환 — --find/--replace, --occurrence 로 k번째만",
    ),
    ("set-cell", "표 셀 텍스트 기록 — --table/--row/--col/--text"),
    (
        "insert-text-in-cell",
        "표 셀 문단에 텍스트 삽입 — --table/--row/--col/--text [--offset] [--cell-para]",
    ),
    (
        "delete-text-in-cell",
        "표 셀 문단 텍스트 삭제 — --table/--row/--col/--count [--offset] [--cell-para]",
    ),
    (
        "insert-text",
        "문단 좌표에 텍스트 삽입 — --section/--para/--offset/--text",
    ),
    (
        "delete-text",
        "문단 좌표 텍스트 삭제 — --section/--para/--offset/--count",
    ),
    (
        "insert-paragraph",
        "빈 문단 삽입 — --section/--para (앞 문단 서식 상속)",
    ),
    (
        "delete-paragraph",
        "문단 삭제 — --section/--para (마지막 문단은 거부)",
    ),
    (
        "merge-paragraph",
        "문단 병합 — --section/--para (para 를 앞 문단에 합침, 0 거부)",
    ),
    (
        "insert-page-break",
        "쪽 나눔 삽입 — --section/--para/--offset",
    ),
    (
        "insert-column-break",
        "단 나눔 삽입 — --section/--para/--offset",
    ),
    (
        "insert-table",
        "본문 표 생성 — --rows/--cols [--section/--para/--offset]",
    ),
    (
        "set-numbering-restart",
        "번호 다시 시작 — --mode [--count] [--section] [--para]",
    ),
    ("insert-row", "표 행 삽입 — --table/--row [--below]"),
    ("insert-col", "표 열 삽입 — --table/--col [--right]"),
    ("delete-row", "표 행 삭제 — --table/--row"),
    ("delete-col", "표 열 삭제 — --table/--col"),
    (
        "merge-cells",
        "표 셀 병합 — --table/--row/--col/--end-row/--end-col",
    ),
    ("split-cell", "병합 셀 분할 — --table/--row/--col"),
    (
        "split-cell-into",
        "셀 n×m 분할 — --table/--row/--col/--rows/--cols [--equal-row-height] [--merge-first]",
    ),
    (
        "split-table",
        "표 나누기 — --table/--row (row 는 뒤 표 시작 행, 0 거부)",
    ),
    (
        "fit-table",
        "표를 페이지 본문 폭에 맞춤 — --table (축소 전용)",
    ),
    (
        "resize-table",
        "표 행/열 크기 조절 — --table/--row/--col [--vertical] [--forward] [--line]",
    ),
    (
        "resize-table-cell",
        "표 셀 크기 조절 — --table/--row/--col [--vertical] [--forward]",
    ),
    (
        "set-cell-props",
        "표 셀 속성 — --table/--row/--col/--props (JSON)",
    ),
    ("set-table-props", "표 속성 — --table/--props (JSON)"),
    ("move-table", "표 위치 이동 — --table/--dx/--dy (HWPUNIT)"),
    (
        "merge-table",
        "다음 표와 붙이기 — --table (사이는 빈 문단만 허용)",
    ),
    (
        "set-column-widths",
        "표 열 폭 설정 — --table/--widths (HWPUNIT, 개수=열 수)",
    ),
    ("insert-footnote", "각주 삽입 — --section/--para/--offset"),
    ("insert-endnote", "미주 삽입 — --section/--para/--offset"),
    (
        "insert-equation",
        "수식 삽입 — --script [--section/--para/--offset/--font-size/--color]",
    ),
    (
        "delete-footnote",
        "각주/미주 삭제 — --section/--para/--ctrl",
    ),
    (
        "delete-text-in-footnote",
        "각주/미주 텍스트 삭제 — --section/--para/--ctrl/--fn-para/--offset/--count",
    ),
    (
        "insert-footnote-text",
        "각주/미주 문단 텍스트 삽입 — --section/--para/--ctrl/--fn-para/--offset/--text",
    ),
    (
        "split-paragraph-in-footnote",
        "각주/미주 문단 분할 — --section/--para/--ctrl/--fn-para/--offset",
    ),
    (
        "merge-paragraph-in-footnote",
        "각주/미주 문단 병합 — --section/--para/--ctrl/--fn-para",
    ),
    (
        "apply-para-format-in-footnote",
        "각주/미주 문단 서식 적용 — --section/--para/--ctrl/--fn-para/--props",
    ),
    (
        "add-bookmark",
        "책갈피 추가 — --name/--section/--para/--offset",
    ),
    ("delete-bookmark", "책갈피 삭제 — --section/--para/--ctrl"),
    ("delete-table", "표 삭제 — --table (본문 최상위)"),
    (
        "rename-bookmark",
        "책갈피 이름 변경 — --section/--para/--ctrl/--name",
    ),
    (
        "delete-header-footer",
        "머리말/꼬리말 삭제 — --header|--footer [--section] [--apply-to]",
    ),
    (
        "insert-header-footer-text",
        "머리말/꼬리말 텍스트 삽입 — --header|--footer --text [--section] [--apply-to] [--para] [--offset]",
    ),
    (
        "set-header-footer-text",
        "머리말/꼬리말 문단 텍스트 교체 — --header|--footer --text [--section] [--apply-to] [--para]",
    ),
    (
        "delete-hf-text",
        "머리말/꼬리말 텍스트 삭제 — --header|--footer --count [--section] [--apply-to] [--para] [--offset]",
    ),
    (
        "set-hf-picture",
        "머리말/꼬리말 그림 속성 설정 — --section/--para/--ctrl/--inner-para/--inner-ctrl/--props",
    ),
    (
        "apply-hf-template",
        "머리말/꼬리말 마당 적용 — --header|--footer --template [--section] [--apply-to]",
    ),
    (
        "split-paragraph-in-hf",
        "머리말/꼬리말 문단 분할 — --header|--footer [--section] [--apply-to] [--para] [--offset]",
    ),
    (
        "merge-paragraph-in-hf",
        "머리말/꼬리말 문단 병합 — --header|--footer [--section] [--apply-to] [--para]",
    ),
    (
        "apply-para-format-in-hf",
        "머리말/꼬리말 문단 서식 적용 — --header|--footer --props [--section] [--apply-to] [--para]",
    ),
    (
        "toggle-hide-hf",
        "머리말/꼬리말 감추기 토글 — --header|--footer [--page]",
    ),
    (
        "split-paragraph-in-cell",
        "표 셀 문단 분할 — --table/--row/--col [--cell-para] [--offset]",
    ),
    (
        "merge-paragraph-in-cell",
        "표 셀 문단 병합 — --table/--row/--col [--cell-para]",
    ),
    (
        "apply-char-format",
        "본문 글자 서식 적용 — --props JSON [--section] [--para] [--offset] [--count]",
    ),
    (
        "apply-para-format",
        "본문 문단 서식 적용 — --props JSON [--section] [--para]",
    ),
    (
        "apply-style",
        "본문 문단 스타일 적용 — --style N [--section] [--para]",
    ),
    (
        "apply-cell-style",
        "표 셀 문단 스타일 적용 — --table/--row/--col --style [--cell-para]",
    ),
    (
        "apply-para-format-in-cell",
        "표 셀 문단 서식 적용 — --table/--row/--col --props [--cell-para]",
    ),
    (
        "apply-char-format-in-cell",
        "표 셀 글자 서식 적용 — --table/--row/--col [--cell-para] [--start/--end] [--props]",
    ),
    (
        "delete-control",
        "문단 컨트롤 삭제 — --section/--para/--ctrl (갈래 무관)",
    ),
    (
        "insert-header-footer",
        "머리말/꼬리말 생성 — --header|--footer [--section] [--apply-to]",
    ),
    (
        "insert-field-in-hf",
        "머리말/꼬리말 필드 삽입 — --header|--footer --field-type 1|2|3 [--section] [--apply-to] [--para] [--offset]",
    ),
    (
        "set-column-def",
        "구역 단 정의 — --count [--section] [--type 0|1|2] [--same-width|--mixed-width] [--spacing]",
    ),
    ("delete-equation", "수식 삭제 — --section/--para/--ctrl"),
    (
        "split-paragraph",
        "본문 문단 분할 — --section/--para/--offset",
    ),
    (
        "set-page-hide",
        "쪽 감추기 — [--hide-header/--hide-footer/--hide-master/--hide-border/--hide-fill/--hide-page-num]",
    ),
    (
        "transpose-table",
        "표 행/열 바꿈 — --table (병합 없는 본문 최상위 표)",
    ),
    (
        "set-equation-properties",
        "본문 수식 속성 변경 — --section/--para/--ctrl --props",
    ),
    (
        "insert-image",
        "도장·서명 그림 삽입 — --image/--page/--x/--y (HWPUNIT)",
    ),
    (
        "group-shapes",
        "도형 묶기 — --targets para,ctrl;para,ctrl [--section]",
    ),
    ("set-page-def", "용지 설정 — --props JSON [--section]"),
    ("set-section-def", "구역 정의 — --props JSON [--section]"),
    (
        "apply-endnote-shape",
        "미주 모양 설정 — --props JSON [--section]",
    ),
    (
        "insert-picture",
        "본문 그림 삽입 — --image/--section/--para/--offset (문단 좌표)",
    ),
    ("delete-picture", "본문 그림 삭제 — --section/--para/--ctrl"),
    (
        "set-picture",
        "본문 그림 속성 — --section/--para/--ctrl/--props",
    ),
    (
        "set-page-border-fill",
        "쪽 테두리/배경 — --props JSON [--section]",
    ),
    (
        "redact",
        "개인정보 마스킹 — --kind 선택, findings 봉투, --no-raw",
    ),
    ("sanitize", "메타데이터 제거 — removed 봉투, --in-place"),
    ("rename-bookmark", "책갈피 이름 변경"),
    ("delete-header-footer", "머리말/꼬리말 삭제"),
    ("insert-header-footer-text", "머리말/꼬리말 텍스트 삽입"),
    ("set-header-footer-text", "머리말/꼬리말 문단 텍스트 교체"),
    ("set-hf-picture", "머리말/꼬리말 그림 속성 변경"),
    ("apply-hf-template", "머리말/꼬리말 마당 적용"),
    ("delete-hf-text", "머리말/꼬리말 텍스트 삭제"),
    ("insert-field-in-hf", "머리말/꼬리말 필드 삽입"),
    ("split-paragraph-in-hf", "머리말/꼬리말 문단 분할"),
    ("toggle-hide-hf", "쪽 머리말/꼬리말 감추기 토글"),
    ("merge-paragraph-in-hf", "머리말/꼬리말 문단 병합"),
    ("apply-char-format", "본문 글자 서식 적용"),
    ("split-paragraph", "본문 문단 분할"),
    ("apply-para-format", "본문 문단 서식 적용"),
    ("apply-style", "본문 문단 스타일 적용"),
    ("set-numbering-restart", "문단 번호 다시 시작"),
    ("apply-para-format-in-hf", "머리말/꼬리말 문단 서식 적용"),
    ("apply-endnote-shape", "미주 모양 적용"),
    ("insert-footnote-text", "각주 텍스트 삽입"),
    ("delete-text-in-footnote", "각주/미주 텍스트 삭제"),
    ("split-paragraph-in-footnote", "각주/미주 문단 분할"),
    ("merge-paragraph-in-footnote", "각주/미주 문단 병합"),
    ("apply-para-format-in-footnote", "각주 문단 서식 적용"),
    ("set-chart-data", "차트 숫자 데이터 기록 — --chart/--data JSON"),
    ("insert-number", "쪽 새 번호로 시작 — --number/--section"),
    ("insert-shape", "본문 도형 삽입 — --width/--height"),
    ("delete-shape", "본문 도형 삭제 — --section/--para/--ctrl"),
    ("group-shapes", "본문 도형 묶기 — --section/--para/--ctrl"),
    ("set-form-value", "본문 양식 값 설정 — --section/--para/--ctrl/--value"),
    (
        "set-form-value-in-cell",
        "표 셀 양식 값 설정 — --table/--row/--col/--ctrl/--value",
    ),
    ("ungroup-shape", "본문 도형 묶음 풀기 — --section/--para/--ctrl"),
];

const INSPECT_SUBCOMMANDS: [(&str, &str); 4] = [
    (
        "hidden-text",
        "은닉 텍스트 탐지 — --threshold-pt 임계·--include-offpage 쪽 밖",
    ),
    (
        "injection",
        "프롬프트 주입 신호 신고 — 문서는 고치지 않고 표시만 한다",
    ),
    (
        "unicode",
        "유니코드 기만 판정 — confusable·bidi·비가시 문자, --kind 필터",
    ),
    (
        "watermark",
        "숨은 마크 탐지 — 제로폭 비트열·동형자·공백 스테가노, --kind 필터",
    ),
];

/// 하위 명령 배열을 해당 부모 항목에 단다. 항목 정의 자리를 건드리지
/// 않는 후처리인 이유: 저 vec 은 거의 모든 표면 PR 이 지나는 자리라, 삽입 지점을
/// 밖으로 빼야 병렬 PR 과의 충돌면이 줄어든다.
fn attach_subcommands(commands: &mut [serde_json::Value]) {
    for entry in commands.iter_mut() {
        let subs: &[(&str, &str)] = match entry["name"].as_str() {
            Some("edit") => &EDIT_SUBCOMMANDS,
            Some("inspect") => &INSPECT_SUBCOMMANDS,
            _ => continue,
        };
        let mut seen = std::collections::BTreeSet::new();
        let list: Vec<serde_json::Value> = subs
            .iter()
            // 병렬 통합 중 기존 선언이 한 번 더 들어와도 공개 surface 는 한 번만 낸다.
            // 순서는 정본 배열의 첫 등장을 보존해 USAGE 계약과 비교 가능하게 한다.
            .filter(|(name, _)| seen.insert(*name))
            .map(|(name, summary)| serde_json::json!({ "name": name, "summary": summary }))
            .collect();
        entry["subcommands"] = serde_json::json!(list);
    }
}

fn capabilities_command_entries() -> Vec<serde_json::Value> {
    let mut commands = Vec::new();
    core::extend(&mut commands);
    extended::extend(&mut commands);
    attach_subcommands(&mut commands);
    commands
}

/// [#3694] 명령 이름 목록 (did-you-mean 후보).
pub(crate) fn capabilities_command_names() -> Vec<String> {
    crate::cli::catalog::commands()
        .iter()
        .filter(|command| command.in_capabilities())
        .map(|command| command.name.to_string())
        .collect()
}

/// [#3694] 레벤슈타인 거리 — 의존성 없이 소형 구현 (이름 환각 교정용).
pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// [#3694] 후보 중 가장 가까운 이름 — 임계(길이 대비 1/3, 최소 1·최대 3) 초과면 None.
/// 오제안 0 원칙: 애매하면 제안하지 않는 편이 경량 에이전트에게 안전하다.
pub(crate) fn closest_name<'a, I: IntoIterator<Item = &'a str>>(
    input: &str,
    candidates: I,
) -> Option<String> {
    let mut best: Option<(usize, &str)> = None;
    for c in candidates {
        let d = levenshtein(input, c);
        if best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, c));
        }
    }
    let (d, name) = best?;
    let cap = (input.chars().count() / 3).clamp(1, 3);
    (d <= cap).then(|| name.to_string())
}

/// [#4220 T4] 사용법 오류(exit 2)의 stderr **마지막 줄**에 싣는 정형 수복 한 줄.
///
/// 문법: `수복: ` 접두어 + 한 줄 JSON `{"nextCall":{"name":<명령>,"subcommand"?:<하위>,"why":<이유>}}`.
/// `nextCall` 어휘는 MCP 오류 봉투(R72, `tool_error_with_next`)와 같다 — CLI 와 MCP 가
/// 같은 모양을 쓰면 소비자가 한 어휘로 수복 루프를 짠다. 계약 3면:
///
/// 1. **오제안 0(R72)** — 다음 호출이 결정론적으로 정해지는 실패 부류에서만 호출한다.
///    애매하면 이 줄 자체가 없어야 하므로, 호출부가 확신 판정(#3694 임계 등)을 먼저 한다.
/// 2. **`name` 실존** — 호출부 책임이고 계약 테스트(`tests/nextcall_cli_contract.rs`)가
///    capabilities 단일 출처와 대조해 고정한다. `arguments` 는 싣지 않는다: CLI 는
///    호출자의 나머지 argv 가 옳다고 검증한 바 없고(오제안 0 은 인자에도 적용된다),
///    비밀번호 같은 민감 인자를 stderr 로 되울리지 않는 뜻도 겸한다.
/// 3. **stdout 무침해** — 실패 3면 계약(#2707: exit 2·stdout 0 B·stderr 안내)에
///    stderr 한 줄만 더하는 추가 전용 확장이다. 산문(오류·힌트·사용법)을 모두 낸 뒤
///    마지막에 호출해야 한다 — 소비자는 "마지막 `수복: ` 줄 하나"만 파싱한다.
pub(crate) fn eprint_usage_recovery(next_command: &str, subcommand: Option<&str>, why: &str) {
    let mut next = serde_json::json!({ "name": next_command, "why": why });
    if let Some(sub) = subcommand {
        next["subcommand"] = serde_json::json!(sub);
    }
    eprintln!("수복: {}", serde_json::json!({ "nextCall": next }));
}

/// [#3828 B1] `capabilities --search <키워드...> [--json]` — commands[].name·summary 를
/// 대소문자 무시 부분 문자열로 필터한다. 결정론적 매칭(유사도 점수·LLM 없음).
///
/// 키워드를 공백으로 여러 개 주면(예: `--search "표 병합"`) **AND** 조건으로 좁힌다 —
/// 검색 도구의 통상 관례(모든 검색어를 만족해야 좁혀진다)를 따르고, 사용자가 한
/// 단어로는 너무 넓은 결과를 받고 두 번째 단어로 더 좁히고 싶을 때 OR 보다 AND 가
/// 직관과 맞는다. OR 이 필요하면 `--search` 를 두 번 호출하면 된다(별도 결과 두 묶음).
fn show_capabilities_search(query: &str, json_mode: bool) -> i32 {
    let keywords: Vec<String> = query.split_whitespace().map(|k| k.to_lowercase()).collect();
    let commands = capabilities_command_entries();
    let matched: Vec<serde_json::Value> = commands
        .into_iter()
        .filter(|c| {
            let name = c["name"].as_str().unwrap_or_default().to_lowercase();
            let summary = c["summary"].as_str().unwrap_or_default().to_lowercase();
            // [#3884 G4] 하위 명령의 이름·요약도 검색 대상이다 — 이것이 없으면
            // `--search redact` 가 edit 를 못 찾아 R31 발견이 하위 명령 위에서
            // 절반만 동작한다.
            let subs = c["subcommands"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|s| {
                            format!(
                                "{} {}",
                                s["name"].as_str().unwrap_or_default(),
                                s["summary"].as_str().unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default()
                .to_lowercase();
            let haystack = format!("{name} {summary} {subs}");
            keywords.iter().all(|k| haystack.contains(k.as_str()))
        })
        .collect();

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": crate::ENVELOPE_SCHEMA_VERSION,
            "tool": "rhwp",
            "version": rhwp::version(),
            "search": query,
            "commands": matched,
        });
        println!("{}", envelope);
        return crate::EXIT_OK;
    }

    if matched.is_empty() {
        println!("'{query}' 에 매치하는 명령이 없습니다.");
        return crate::EXIT_OK;
    }
    println!("'{query}' 검색 결과 ({}건):", matched.len());
    for c in &matched {
        let name = c["name"].as_str().unwrap_or_default();
        let summary = c["summary"].as_str().unwrap_or_default();
        println!("  {name:<24} {summary}");
    }
    crate::EXIT_OK
}

pub(crate) fn show_capabilities(args: &[String]) -> i32 {
    // [#3828 B1] 처음 오는 에이전트는 정확한 명령 이름을 모른다 — `--search <키워드>`
    // 로 commands[].name·summary 를 부분 문자열(대소문자 무시)로 훑을 수 있게 한다.
    // 결정론적 매칭이다: 유사도 점수·LLM 판단 없음 (#3787 원칙과 동일).
    let mut search_query: Option<String> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--search" => {
                i += 1;
                match args.get(i) {
                    Some(q) => search_query = Some(q.clone()),
                    None => {
                        eprintln!("오류: --search 뒤에 키워드가 필요합니다.");
                        return crate::EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return crate::EXIT_USAGE;
            }
        }
        i += 1;
    }
    if let Some(query) = search_query {
        return show_capabilities_search(&query, json_mode);
    }
    // --search 없이 --json 만 온 경우는 기존과 동일하게 사용법 오류로 처리한다
    // (기본 `capabilities` — 인자 없음 — 의 동작·출력은 절대 바뀌지 않는다).
    if json_mode {
        eprintln!(
            "오류: --json 은 --search 와 함께 사용합니다 (capabilities --search <키워드> --json)."
        );
        return crate::EXIT_USAGE;
    }
    let caps = capabilities_value();
    println!("{}", caps);
    crate::EXIT_OK
}

/// [#3828 B2] `capabilities` 본문(표지 전) — `export-agent-manifest` 가 조립할 때도
/// 이 함수 하나를 부른다. 두 곳에서 각자 만들면 매니페스트의 `capabilities` 필드가
/// 실제 `capabilities` 출력과 조용히 갈라질 수 있다.
pub(crate) fn capabilities_value() -> serde_json::Value {
    let commands = capabilities_command_entries();

    serde_json::json!({
        "schemaVersion": crate::ENVELOPE_SCHEMA_VERSION,
        "tool": "rhwp",
        "version": rhwp::version(),
        // hwp5 는 convert·extract-pages·edit -o *.hwp 가 실제로 내는 산출 형식이다
        // (봉투의 format/outputFormat 이 "hwp5"). 쓰기 목록에서 빠져 있어 매니페스트만
        // 읽은 에이전트가 "HWP5 로는 못 쓴다"고 오판했다.
        "formats": { "read": ["hwp5", "hwpx", "hwp3", "hml"], "write": ["hwp5", "hwpx", "hml", "pdf", "svg", "png", "txt", "md", "doclang"] },
        "exitCodes": {
            "0": "성공",
            "1": "런타임 실패 (읽기·파싱·렌더·쓰기)",
            "2": "사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과)",
            "3": "검증 단언 실패 — convert/export-hwpx --verify IR 차이, edit 3종 --verify 저장본 불일치, run 계획 assertions 미충족, render-diff --json 시각 회귀 검출(사람 모드는 종전대로 1), layout-anomaly --strict 확정 신호(overflow·off-canvas·overlap·text-overlap) 검출(기본은 0)",
            "4": "--verify-pages 페이지 수 불일치 (convert/export-hwpx)",
        },
        "jsonContract": {
            "stdout": "데이터(JSON/NDJSON)만 — 진단·진행·요약은 stderr",
            "schemaPolicy": "필드 추가 허용, 변경·삭제는 schemaVersion 범프",
            // [#3884 G3] run 의 예외는 설계다(판정을 데이터로 보고) — 적지 않으면
            // "실패 = stdout 0바이트"를 믿는 소비자가 run 에서 깨진다.
            "failure": "단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1. 예외: run — 실패도 봉투를 stdout 으로 낸다(계획 안 문서 부재 등 입력 오류 exit 1 + error, 계획 무효 exit 2 + invalid[], 단언 실패 exit 3 + verify 저널)",
            // [#3707] 봉투에 담기는 문서 유래 문자열의 유니코드 기만 판정. 이 키가
            // 있으면 바이너리가 검사한다는 뜻이다 — 키가 없으면 '깨끗함'이 아니라
            // '검사하지 않음'으로 읽어야 한다.
            "textSecurity": {
                "field": "textSecurity",
                "status": ["clean", "warning"],
                "kinds": ["confusableFieldName", "mixedScript", "bidiControl", "invisibleChar", "ansiEscape"],
                "policy": "보고 전용 — 문서 문자열을 수정하지 않는다",
                "surfaces": ["fields --json", "edit fill-fields --json(confusable)", "run --json(steps[].confusable)"],
            },
        },
        "batch": {
            "subcommands": ["export-text", "info", "export-structure", "export-tables", "fields", "search", "extract-data", "convert", "fill"],
            "flags": ["--json", "--threads", "--mode", "--query", "--kind", "--limit", "--out-dir", "--verify", "--verify-pages", "--form", "--name-field", "--dry-run"],
            "ordering": "입력 순서 보존 (fill 은 데이터 행 순서)",
            // [#3719] fill 축만 입력 축이 다르다 — 여기를 읽고 stdin 에 경로를 밀어 넣으면
            // 그 프로세스는 아무것도 읽지 않은 채 데이터 파일만 처리한다.
            "input": "stdin, 한 줄당 파일 경로 하나 (batch 에서는 경로 목록 전용). 단 fill 축은 stdin 을 읽지 않는다 — --form 서식 1개 + --data 행 파일(.jsonl|.csv) 1개를 받고, 한 행이 산출물 하나가 된다",
            "authentication": "지원하지 않음 — --password·--password-stdin·--output-password·--output-password-stdin 은 usage error; 암호화 batch 의 credential 전달 계약은 아직 정의되지 않았다",
            // [#3626→#3719] 파일을 쓰는 축(convert·fill)의 목적지·충돌 규약을 밝힌다.
            "output": "convert·fill 축만 파일을 쓴다. convert: 목적지는 --out-dir 하나, 이름은 <입력이름>.hwp — 대소문자만 다른 이름을 포함해 같은 이름이 둘 이상이면 한 건도 쓰지 않고 exit 2. fill: 이름은 --name-field 값(파일명 금지 문자는 _ 로 치환), 없으면 0001 순번이며 겹치면 뒤에 _2·_3 을 붙여 덮어쓰지 않는다",
            // [#3830] extract-data 축의 --limit 는 **배치 전체가 아니라 문서마다** 적용되는
            // 상한이다 — 단건 `extract-data --limit` 과 같은 의미다.
            "limit": "extract-data 의 --limit 는 문서마다 적용된다(전역 상한 아님) — counts·totalItemCount 는 절단 전 그 문서의 총량이다",
            "exitAggregation": "error 레코드가 하나라도 있으면 1, 없고 verifyPages 불일치가 있으면 4, verify 차이만 있으면 3, 전부 통과면 0",
        },
        "commands": commands,
    })
}
