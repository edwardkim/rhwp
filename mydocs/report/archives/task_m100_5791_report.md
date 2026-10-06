# task_m100_5791 처리결과 — 명령별 `rhwp <명령> [<하위명령>] --help`

- 이슈: [#5791](https://github.com/edwardkim/rhwp/issues/5791)
- 분석: [`mydocs/working/task_m100_5791_stage1.md`](../../working/archives/task_m100_5791_stage1.md)
- 기준: `fb434269e` (devel) · `rhwp v0.8.4`

## 1. 변경 요약

| 파일 | 내용 |
| --- | --- |
| `src/cli/metadata/help/sink.rs` (신규) | 도움말 출력 경계. 형제 모듈의 `println!` 을 가려 한 곳(`emit`)으로 모으고, 스코프가 걸리면 절 단위로 고른다 |
| `src/cli/metadata/help/mod.rs` | `scoped_help()` — 절 고르기 · 그룹 목차 · `capabilities` 폴백 |
| `src/cli/metadata/help/{public,edit,protocol}.rs` | `use super::sink::println;` 한 줄씩 (본문 텍스트 무변경) |
| `src/cli/metadata/help/public.rs` | 통짜 도움말 머리에 안내 한 줄 추가 |
| `src/main.rs` | 디스패치 **앞**에서 `scoped_help` 로 가로채기 (4줄) |
| `tests/cli_scoped_help_contract.rs` (신규) | 계약 9본 |
| `tests/cases/threat_scan_cli_contract.rs` | 기존 `--help` 한 줄 동일성 → 호출 형태 포함 판정으로 갱신 |

`println!` **호출 지점 1,163개는 한 줄도 바꾸지 않았다.** 명령을 추가하는 PR 이 지나는
자리를 그대로 두기 위해서다.

## 2. 전후 (실측)

### 종료 코드

| 경로 | 전 | 후 |
| --- | --- | --- |
| `capabilities` 98종 `--help` | exit 2 **79종** · exit 1 **5종** · exit 0 14종 | **98종 전부 exit 0** |
| `edit` 하위 88종 | 전부 exit 2 | 전부 exit 0 |
| `inspect` 하위 4종 | 전부 exit 2 | 전부 exit 0 |
| `edit`/`inspect`/`batch` 그룹 | exit 2 | exit 0 (목차) |

```
$ rhwp fields --help                    # 전: 오류: 알 수 없는 옵션: --help (exit 2)
  fields <파일.hwp|파일.hwpx> [--json]
      누름틀/필드 조사 (읽기 전용) — 이름·안내문·지시문·현재값·위치

      --json                    계약 봉투 JSON을 stdout에 출력

(전체 도움말: rhwp --help · JSON 자기서술: rhwp capabilities --search fields)
                                                                     → exit 0
```

### 읽어야 하는 양

| 물음 | 전 | 후 | 비 |
| --- | --- | --- | --- |
| `fields` 는 어떻게 부르나 | 통짜 71,978 B / 1,163줄 | **302 B / 6줄** | 239배 |
| `edit` 에 무엇이 있나 | 통짜 또는 절 603줄 | 목차 **95줄** (하위 88 + 다음 수) | 6.3배 |
| `edit fill-fields` 옵션 | 통짜에서 찾아야 함 | 해당 절만 | — |

통짜 출력은 안내 한 줄이 늘어 **72,088 B / 1,164줄** (전 71,978 B / 1,163줄).
그 한 줄을 뺀 본문은 바이트 동일하다.

### 절이 없던 자리

top-level 3종(`dump-extents`·`measure-width`·`core-pages`)과 `edit` 하위 19종은
`rhwp --help` 에 절이 없다. 이들에게는 `capabilities` 의 요약·선언 플래그로 최소 답을 낸다.
**사용법 문자열을 지어내지 않는다.**

## 3. 검증

### red → green

`src/main.rs` 의 가로채기 4줄만 주석 처리하고 같은 계약을 돌렸다.

```
test result: FAILED. 3 passed; 6 failed
  every_declared_command_answers_help_on_stdout   FAILED
  every_declared_subcommand_answers_help          FAILED
  group_help_is_an_index_of_subcommands           FAILED
  dump_commands_no_longer_read_help_as_a_file     FAILED
  short_flag_answers_the_same_way                 FAILED
  scoped_help_is_a_fraction_of_the_whole_help     FAILED
```

되돌리면 9/9 통과. 통과한 3본은 **전후 불변**을 지키는 가드다 —
값 자리 보호 · 모르는 명령 오류 경로 · 통짜 도움말 유지.

### 실행한 것

| 명령 | 결과 |
| --- | --- |
| `cargo test --test regression_suite_003 cli_scoped_help` | 9 passed |
| `cargo test --test regression_suite_009` (threat-scan 계약) | 5 passed |
| `cargo test --test regression_suite_011` (skills·cli_json 계약) | 122 passed |
| `cargo test --test regression_suite_027` (진단 명령 종료 코드) | 123 passed |
| `cargo test --test regression_suite_002` (hwp5 inventory/anchor) | 114 passed |
| `cargo test --test regression_suite_024` (agent_codex 계약) | 120 passed |
| `cargo test --test regression_suite_019` | 121 passed / 1 실패 — **이 변경과 무관** (아래) |
| `rustfmt --edition 2021` (변경 파일 per-file) | 차이 없음 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |

### 무관한 기존 실패 1건 (Windows 체크아웃 한정)

`agent_knowledge_map_skill_contract::skill_frontmatter_names_knowledge_map` 이
`SKILL.md` 를 `starts_with("---\n")` 로 본다. Windows 기본 `core.autocrlf=true` 체크아웃에서는
`.claude/skills/*/SKILL.md` 27개가 전부 CRLF 라 이 판정이 거짓 실패한다(같은 패턴의 계약 14본).
저장소 인덱스는 LF 라 Ubuntu CI 는 초록이다. 이 PR 과 무관하며 별도로 다룬다.

## 4. 안전 경계

- `--help` 가 값 자리면 가로채지 않는다 — `edit replace-text <파일> --find --help` 는
  여전히 사용법 오류(exit 2), stdout 비어 있음. 계약으로 못박았다.
- 모르는 명령은 기존 오류 경로 그대로(exit 2, `알 수 없는 명령`).
- 가로채기는 디스패치 **앞**이라 파일을 열거나 편집을 시작하기 전에 끝난다.
- 계약의 오라클은 골든 파일이 아니라 `capabilities` 자기서술이다 — 명령이 늘면 가드 범위도 는다.
