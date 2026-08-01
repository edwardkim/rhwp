---
kind: report
status: active
canonical: mydocs/report/task_m100_3759/README.md
last_verified: 2026-08-02
---

# #3759 처리 기록 — 계획 v2 `--dry-run` (#3719 §6-3)

## 문제

계획 실행기(#3703)는 **실행해야만** 계획의 유효성을 알 수 있었다. 선검증이 위반을 잡아
`exit 2` + 출력 파일 부재로 안전하게 끝내긴 하지만, 계획을 *생성하는* 에이전트가 알고
싶은 건 그 전 단계다 — "이 계획, 돌리면 되나? 무엇이 얼마나 바뀌나?"

## 구현

- `rhwp run <계획.json> --dry-run [--json]` — 선검증만 돌고 **디스크 무변경**.
- 유효하면 `preview[]` + `dryRun:true` + exit 0. 위반이 있으면 실행 모드와 **똑같이**
  `invalid[]` + exit 2 (dry-run 이 통과시킨 계획이 실행에서 막히는 일이 없다).
- `preview` 는 선검증이 이미 계산한 값 그대로다 — 판정자와 미리보기가 같은 계산이라
  "검사 결과와 실제 실행이 다를" 여지가 구조적으로 없다.

| action | preview 필드 |
|---|---|
| `fill_fields` | `targets[{name, occurrence, sameNameCount, value}]` |
| `replace_text` | `find`, `matches`(전체 일치), `willReplace`(occurrence 지정 시 1) |
| `set_checkbox` | `occurrence`, `available`(빈 체크박스 총수) |
| `set_cell` | `table`,`row`,`col`, `currentText`(현재값), `newText` |

## 설계 판단 — 플래그를 계획서 필드로

`dryRun` 을 계획서(`plan.dryRun`)에 두면 MCP `hwp_run_plan{plan}` 이 **인자 추가도
cmdTemplate 변경도 없이** 같은 계약을 얻는다. CLI `--dry-run` 은 그 필드를 덮어쓰는
편의 입구일 뿐이고, **의도의 단일 출처는 계획서**다 (#3719 불변식 1 — 단일 출처).

사람 모드는 `검사 통과: N step 실행 가능 (디스크 무변경, 산출 예정 …)` + step 별 한 줄
요약(`preview_line`)을 낸다.

## 검증

- 신규 `run_plan_dry_run_contract` 5건 green:
  ① 유효 계획 preview + **산출 파일 부재**(계약의 핵심) ② 위반 시 실행 모드와 동일 판정
  ③ 계획서가 실은 `dryRun` 이 CLI 플래그와 동등 ④ 실행 모드 무회귀 ⑤ MCP 경로 디스크 무변경
- 무회귀: `run_plan_contract` · `cli_json_contract`
- clippy `-D warnings` 0 · fmt clean

## 남은 것

- 계획 스키마 JSON Schema 공개(#3719 §6-4) — dry-run 이 "검사"라면 스키마는 "생성"의
  정답지다. 둘이 짝을 이뤄야 에이전트가 계획을 안전하게 만들 수 있다.
- 조건부 step(`if` 필드값) — dry-run 의 preview 가 분기까지 예측해야 하므로 이후 과제.
