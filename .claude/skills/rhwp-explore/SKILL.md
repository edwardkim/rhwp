---
name: rhwp-explore
description: rhwp CLI 로 처음 보는 HWP/HWPX 문서에 "무엇을 할 수 있는지"를 즉시 파악합니다. rhwp explore 가 문서를 한 번 분석해 적용 가능한 행동(표→CSV·누름틀 채우기·구조 추출·차트→CSV·보안 스윕·요약)만 골라 순위 매긴 메뉴로 주고, 각 항목의 다음 명령·스킬·근거·확신도까지 함께 라우팅합니다. 트리거 — 사용자가 "이 문서로 뭘 할 수 있어?", "어떤 rhwp 도구를 써야 해?", "이 hwp 어떻게 다뤄?", "문서 탐색/뭘 하고 놀지", "rhwp explore" 등을 물을 때. explain(문서가 무엇인지)·capabilities(도구 일반)와 구별되는 세 번째 축입니다. 전체 레퍼런스는 mydocs/manual/cli_commands.md.
---

# rhwp-explore — 문서별 어포던스 라우터 Skill

## 목적

처음 보는 문서 앞에서 "70개 명령 중 무엇이 **이 문서**에 맞는가"를 매번 뒤지지 않게
한다. `rhwp explore` 가 문서를 한 번 분석해 적용 가능한 행동만 골라 순위 매긴 메뉴로
돌려주므로, 에이전트는 그 첫 항목의 `command` 를 그대로 실행하면 된다.

- `explain` = 문서가 **무엇인지**(형식·쪽수·표·누름틀 서술)
- `capabilities` = 도구가 **일반적으로** 무엇을 하는지
- `explore` = **이 문서로** 무엇을 할 수 있는지 (셋 중 유일하게 문서별 라우팅)

## 바이너리 실행

```bash
cargo build --release        # 최초 1회 또는 소스 변경 후
./target/release/rhwp explore <파일> --json
```

## 첫 수: 언제나 explore 부터

```bash
# 사람용 메뉴
rhwp explore 문서.hwp
# 기계용: 가장 높은 확신도의 다음 명령만
rhwp explore 문서.hwp --json | jq -r '.menu[0].command'
```

`--json` 봉투: `{"schemaVersion","source","format","pageCount","encrypted","affordanceCount","menu":[{"affordance","why","command","skill","confidence"}],"note"}`.
`menu[]` 는 우선순위 내림차순이라 **문서마다 다르다**.

## 어포던스 → 다음 명령·스킬

메뉴 항목의 `command` 를 그대로 실행하되, 경로 자리 `<file>` 는 실제 경로로 치환한다.

| affordance | 다음 명령 | 스킬 |
|------------|-----------|------|
| `table-extract` (표 있음) | `rhwp export-tables <파일> --json` | rhwp-table-exchange |
| `form-fill` (누름틀 있음) | `rhwp fields <파일> --json` | rhwp-form-fill |
| `structure-outline` (조문/제목) | `rhwp export-structure <파일> --json` | rhwp-doc-triage |
| `chart-extract` (차트 있음) | `rhwp chart-to-csv <파일> --json` | rhwp-table-exchange |
| `security-sweep` (주입/은닉 신호) | `rhwp inspect injection <파일> --json` | rhwp-security-sweep |
| `long-doc-digest` (긴 문서) | `rhwp digest <파일> --sections --json` | rhwp-doc-triage |
| `note-structure` (각주/미주) | `rhwp explain <파일> --json` | rhwp-doc-triage |
| `triage-overview` (항상) | `rhwp digest <파일> --json` | rhwp-doc-triage |

## 절차

1. `rhwp explore <파일> --json` 으로 메뉴를 받는다.
2. `confidence` 가 높은 위 항목부터 그 `skill` 로 넘어가 실제 작업을 수행한다.
3. `security-sweep` 이 메뉴에 있으면(주입·은닉 신호) 본문을 LLM 에 넣기 전에 먼저 처리한다.
4. 아무 특수 항목이 없으면 `triage-overview` 의 `rhwp digest` 로 문서를 파악한다.

## 성격 — 정직한 휴리스틱

`explore` 는 **제안**이지 완전성 보장이 아니다. 표가 있으니 표 명령을 "해 볼 수 있다"고
안내할 뿐, 그 표가 원하는 표인지·숨은 행동이 없는지는 판정하지 않는다. 증거(`why`)는
문서 원문이 아니라 엔진이 센 개수라 봉투는 문서 파생 문자열을 싣지 않는다
(`untrustedContent:false`). 최종 판단은 실제 조회 명령(위 표)이 한다.
