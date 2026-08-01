---
kind: report
status: active
canonical: mydocs/report/task_m100_3703/README.md
last_verified: 2026-08-01
---

# #3703 처리 기록 — 계획 실행기 `rhwp run` (#3608 실행 3층)

## 문제

에이전트 실패의 뿌리는 다단 체이닝이다 — 호출 사이에서 상태를 잃고, 중간 실패가
반편집 문서를 남긴다. 절차(도구 호출 나열) 대신 **의도(계획서)** 를 받는 3층을 신설했다.

## 구현

- `rhwp run <계획.json> [--json]` + `--plan-json '<인라인>'` (MCP 경로).
- **정적 선검증(실행 0)**: 필드 존재·순번(`collect_all_fields` 계수), 치환 일치 건수
  (`grep` 읽기 전용), 셀 좌표(`resolve_table_cell`), 체크박스 □ 계수 — 위반을 전부
  모아 `invalid[{step,action,reason}]` + exit 2. 출력 파일을 아예 만들지 않는다.
- **원자 실행**: 전 step 을 인메모리 IR 에만 적용(set_field_value_by_name_at ·
  replace_nth/all_native · delete/insert_text_in_cell + recolor). 중간 실패 = 디스크
  무변경. set_cell 좌표는 앞 step 편집으로 밀릴 수 있어 **실행 시점 재해석**.
- **사후 단언 → 단 한 번 저장**: `assertions.verify` 는 #3702 `edit_verify_report`
  재사용(저장 바이트 재파싱↔IR 대조). 실패 시 저장 없이 exit 3 — 자연 트랜잭션.
- **저널 봉투**: step 별 결과(fill 의 filledCount/ambiguous, replace 의 replacedCount,
  set_cell 의 oldText) + verify + assertions 에코. 판정은 전부 데이터.
- 새 편집 로직 0 — 판정자와 적용자가 기존 edit 3종과 동일 함수.
- MCP `hwp_run_plan{plan}`: capabilities 단일 출처에 cmdTemplate
  `["run","--plan-json","{plan}","--json"]` 로 등록 — mcp_serve 본체 무변경.
- help·capabilities 명령 목록 동기화 (`capabilities_covers_every_help_command` 게이트).

## 실측 (evidence.txt 원문)

1. 정상 계획(fill+replace, verify:true): 저널 steps 2건 + `verify identical:true` + exit 0.
2. 위반 2종 섞인 계획: `invalid[]` 에 **두 위반을 한 번에** 보고(두더지잡기 방지),
   exit 2, 출력 파일 부재 = 실행 0 증명.
3. MCP `hwp_run_plan` 인라인 계획: isError:false + structuredContent 로 동일 저널.

## 검증

- 신규 `run_plan_contract` 6건 green (선검증 exit 2·출력 부재 / 중간 불가 원자성 /
  저널-단언 정합 / 왕복 재독 / capabilities 선언 / MCP 저널).
- 무회귀: cli_json_contract · edit_verify 4 · fill 7 · replace 4 · set-cell 5 · mcp_server.
- clippy `-D warnings` 0 · fmt clean.

## 남은 것 (v2 후보)

- 계획 내 다중 문서(input 배열)·조건부 step — 로드맵 #3608 실행 3층 절에 기록.
- verify 실패 실물 재현 픽스처(의도적 손상 주입) — #3702 와 공통 과제.
