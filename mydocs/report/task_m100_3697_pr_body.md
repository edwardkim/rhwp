---
kind: report
status: active
canonical: mydocs/report/task_m100_3697_pr_body.md
last_verified: 2026-08-01
---

# Task #3697 PR 초안 (파킹)

- 제목: `feat(cli): dump-pages --json 페이지네이션 진단 계약을 추가한다 (#3697)`
- 브랜치: fork `kevin9327/rhwp` `task/dump-pages-json` (push 완료, devel f80b910aa 기준)
- 상태: 열린 PR 볼륨 상한(약 10건, 당시 11건) 초과로 파킹 — #3714 로 개설 직후 close. 큐에 여유가 생기면 리베이스 후 재개설.

## 본문

Closes #3697 (#3608 1-C 잔여 공백)

## 무엇

`dump-pages` 에 `--json` 을 추가해 페이지네이션 진단의 기계 계약을 만듭니다.

```
rhwp dump-pages <파일.hwp> [-p <쪽>] [--respect-vpos-reset] --json
```

단건 JSON 봉투(#3237 규약): `schemaVersion`("1.0") · `source` · `pageCount` ·
`pageFilter` · `respectVposReset` · `pages`.

- 항목 kind 6종: fullParagraph / partialParagraph / table / partialTable / shape /
  endnoteSeparator — 텍스트 덤프의 진단 필드(vpos reset·rewind, line_seg 요약,
  분할 표 rows·cut, 미주 출처 #1082, 단별 usedHeight/hwpUsedHeight/usedDiff)를
  구조화 필드로 그대로 노출.
- `extras`: #1700 계열 items 밖 문단(wrapAroundPara / hiddenEmptyPara)을 텍스트
  덤프와 같은 귀속 규칙(#1705 wrap zone · #1955 글뒤로 앵커)으로 노출.
- 실패 경로 stdout 0바이트, exit 계약(#2707: 0/1/2) 유지. capabilities 광고 갱신.

## 왜 이런 구조

구역 전처리(미주 합침 #1082, items 밖 문단 페이지 귀속 #1700·#1705·#1955)를
`page_dump_section_ctx` helper 로 추출해 **텍스트/JSON 두 출력이 같은 코드를
공유**합니다. 두 표면이 서로 다른 진단을 보고하는 드리프트를 구조적으로 차단하고,
텍스트 출력은 바이트 동일을 유지했습니다.

스코프: #3608 1-C 표는 이 항목에 MCP 도구를 짝짓지 않았으므로(1-D 진단 도구
원칙) CLI 계약만 구현하고, `cli_json_contract.rs` 의 MCP 커버리지 테스트에서
dump-pages 를 명시 제외했습니다. 에이전트 수요가 실증되면 별도 이슈로 승격합니다.

## 검증 (red→green)

| 게이트 | 결과 |
|---|---|
| red — 구현 제거 상태에서 신설 계약 테스트 | 3/4 실패 재현 (`--json` 미지원 exit 2) |
| green — `cargo test --profile release-test --test dump_pages_json_contract` | 4 passed |
| green — `cargo test --profile release-test --test cli_json_contract` | 22 passed |
| 텍스트 출력 바이트 동일 | devel 기준 빌드와 전/후 비교, 3샘플×2변형(전체·`-p 1`) **6/6 IDENTICAL** — 미주 샘플(`3-09월_교육_통합_2024-미주사이20.hwp`)·어울림 표(`issue1510_coanchored_float_tables.hwp`) 포함 |
| clippy `-D warnings` (release-test) | 통과 — 도중 doc 오귀속(doc_lazy_continuation) 1건 검출·수정 |
| rustfmt --check (변경 4파일) | 통과 (Windows CRLF newline-style 오탐 제외) |

green 은 devel(f80b910aa) 리베이스 후 재컴파일로 재실행했습니다.

시각 변화 없음: CLI 진단 출력 전용 변경이고, 기존 텍스트 출력은 위 표의 바이트
동일 비교로 불변을 확인했습니다 (렌더링 경로 미접촉 — 전/후 이미지 해당 없음).

처리결과 문서: `mydocs/report/task_m100_3697_report.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
