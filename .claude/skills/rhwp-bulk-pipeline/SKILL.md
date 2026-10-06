---
name: rhwp-bulk-pipeline
description: 폴더의 HWP/HWPX 문서 수백 건을 rhwp batch 로 한 번에 처리합니다. batch info(메타 스윕)/export-text(본문)/export-structure(개요·조문)/export-tables(표)/fields(서식 조사)/search(전역 검색)/extract-data(날짜·금액·수량)/convert(형식 변환)/fill(메일머지) — stdin 한 줄당 경로 → stdout 순수 NDJSON, stderr 사람용 요약, 실패 행 봉투 격리·jq 재시도, 입력 N=성공+실패 게이트까지 닫습니다. 트리거 — 사용자가 "폴더 전체를 텍스트로/한꺼번에 변환", "여러 hwp 대량 처리/코퍼스 추출", "아카이브 전역 검색", "서식 하나에 여러 명 데이터 채워(메일머지)", "rhwp batch" 등을 요청할 때.
---

# rhwp-bulk-pipeline — 폴더 대량 처리 Skill

이 스킬은 **실 에이전트 경로**다. gym 이 아니고, 새 CLI 명령을 발명하지 않는다.
표면은 이미 있는 `rhwp batch <축>` 만 부른다. 에이전트가 필요한 것은 구현이
아니라 **어느 축을 치고, 실패 행을 어디로 보내고, 숫자가 맞을 때 멈추는가**.

상세는 `references/` 를 축·게이트별로 연다. SKILL.md 는 인덱스와 정지 규칙만 담는다.

권위: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
§batch + `rhwp capabilities` 의 batch 항목(stdin·NDJSON·종료 집계·출력 충돌·인증).
실측 원형: 레시피 9(PR #4182)·레시피 5·[`cli_json_pipeline_guide.md`](../../../mydocs/manual/cli_json_pipeline_guide.md).
처리 결과: [`mydocs/working/agent_bulk_pipeline.md`](../../../mydocs/working/archives/agent_bulk_pipeline.md).

## 바이너리

```bash
cargo build --release
./target/release/rhwp batch <축> --json [옵션] < 목록.txt
```

네이티브는 로컬 cargo. Docker 는 WASM 전용.
`batch convert` / `batch fill` 산출은 `output/` 아래 분리 권장. 원본은 불변.

## 세 규약 — 이 스킬 전체를 지배한다

1. **입력은 stdin, 한 줄당 파일 경로 하나.** 인자로 늘어놓지 않는다.
   `batch fill` 만 예외 — `--form` + `--data` (stdin 목록이 아님).
2. **stdout 은 순수 NDJSON.** 한 줄이 문서(또는 fill 행) 하나의 봉투다.
   사람용 요약(`batch: N건 중 …`)은 stderr. 파이프에는 stdout 만 태운다.
3. **실패도 봉투다.** 한 파일이 깨져도 파이프는 죽지 않는다.
   `{"schemaVersion":"1.0","source","error","exitClass":"runtime"}` 를 낸 뒤
   다음 파일로 간다. `--threads` 병렬에서도 **출력 순서는 입력 순서**.

## 사다리 (강제 순회 아님)

`목록 → batch info 선점검 → 본작업 축 → jq 성공/실패 분리 → 실패만 재시도 → N=성공+실패`

질문이 이미 답이면 다음 단으로 내려가지 않는다. 정지 조건은
[28_retry_classes.md](references/28_retry_classes.md) 와 아래 표.

```
목록.txt (한 줄 = 경로)
  ├─ 빈 목록 / 확장자 오인 ──▶ 중단 (B01)
  └─ batch info --json
       ├─ 전건 error ──▶ 경로·권한 먼저 (B02)
       ├─ 암호 신호 ──▶ 단건 --password 로 분리. batch 에 --password 금지 (B03)
       └─ 성공 행 선별
            ├─ 규모만 ──▶ info 보고 정지 (B04)
            ├─ 본문 ──▶ batch export-text (B05)
            ├─ 개요/조문 ──▶ batch export-structure --mode (B06)
            ├─ 표 ──▶ batch export-tables (B07)
            ├─ 서식 조사 ──▶ batch fields (B08)
            ├─ 전역 검색 ──▶ batch search --query (B09)
            ├─ 날짜·금액 ──▶ batch extract-data [--kind] [--limit] (B10)
            ├─ HWP5 변환 ──▶ batch convert --out-dir (이름 예약) (B11)
            └─ 메일머지 ──▶ batch fill --form --data --out-dir (B12)
```

## 요청 → 명령

| 사용자 요청 | 명령 | 레퍼런스 |
| --- | --- | --- |
| 폴더 문서 규모/형식 먼저 | `batch info --json < 목록.txt` | 04_axis_info.md |
| 본문 전부 | `batch export-text --json [--threads N]` | 05_axis_export_text.md |
| 개요/조문 일괄 | `batch export-structure --json [--mode auto\|outline\|clause]` | 06_axis_export_structure.md |
| 표 전부 수확 | `batch export-tables --json` | 07_axis_export_tables.md |
| 서식 템플릿 일괄 조사 | `batch fields --json` | 08_axis_fields.md |
| 아카이브 전역 검색 | `batch search --query <검색어> --json` (`--query` 필수) | 09_axis_search.md |
| 날짜·금액·수량 일괄 | `batch extract-data --json [--kind date\|amount\|number\|all] [--limit N]` | 10_axis_extract_data.md |
| 편집 가능한 HWP5 로 변환 | `batch convert --out-dir <폴더> [--verify] [--verify-pages] --json` | 11_axis_convert.md |
| 서식 1개에 여러 행 채움 | `batch fill --form <서식> --data <행.jsonl\|.csv> --out-dir <폴더> --json` | 12_axis_fill.md |

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| B01 | 목록이 비었거나 경로가 디렉터리 | 중단. find/Get-ChildItem 부터 |
| B02 | info 전건 `error` | 작업 디렉터리·상대경로 확인. 본작업 금지 |
| B03 | `batch --password` / `--password-stdin` | **exit 2**. 암호 문서는 단건 명령으로 뺀다 |
| B04 | 질문이 '몇 쪽/무슨 형식' 이고 info 가 나왔다 | 본문을 뽑지 않는다 |
| B05 | export-text 후 `error` 행 | jq 로 실패만 재시도. 성공 행을 다시 돌리지 않는다 |
| B06 | export-structure `--mode` 오타 | exit 2. `auto\|outline\|clause` 만 |
| B07 | export-tables `tableCount: 0` | 실패가 아니다. 표 없는 문서 |
| B08 | fields `fieldCount: 0` | 누름틀 없음. 그 파일은 table-exchange 후보 |
| B09 | search 에 `--query` 없음 | **exit 2**. 입력을 소비하지 않음 |
| B10 | extract-data `--limit` 로 `truncated:true` | `counts`/`totalItemCount` 는 절단 전 총량. 문서마다 한도 |
| B11 | convert 산출 이름 충돌(대소문자 포함) | **exit 2, 한 파일도 쓰지 않음**. 목록을 갈라 `--out-dir` 분리 |
| B12 | fill 에 stdin 파일 목록을 파이프 | 잘못된 축. `--form`+`--data` 로 다시 |
| B13 | 입력 N ≠ 성공+실패 | 결과 파일이 아니라 파이프 중간(head/grep 버퍼)을 의심 |
| B14 | 종료 코드 1 인데 실패 행이 안 보임 | stdout 을 사람용 요약과 섞어 읽지 않았는지. stderr 분리 |
| B15 | convert/fill `--verify` 차이 | exit 3. 산출은 남는다. 행별 `verify` 봉투 |
| B16 | convert `--verify-pages` 불일치 | exit 4 (error 행이 없을 때). IR 차이(3)와 섞지 말 것 |
| B17 | 사용자 질문이 이미 답변 가능 | 다음 축으로 내려가지 않는다 |
| B18 | `--out-dir` 가 `-` 로 시작 | `./-결과` 로 명시. 다음 플래그로 오인되면 exit 2 |

**금지 기본값**

- 새 `batch *` 서브커맨드·플래그 발명
- gym pack / gym 과제 작성
- `batch` 에 전역 `--password` / `--password-stdin` / `--output-password`
- 실패 행을 침묵 삭제하고 성공만 세기
- `batch fill` 에 stdin 경로 목록을 넣기
- convert 이름 충돌을 무시하고 일부만 쓰기
- 입력 N 게이트를 건너뛰기
- 이 스킬 안에서 onboarding / mcp-session / safe-edit / provenance / doc-triage / form-fill 본문을 재작성

## 인계

- 단건 문서 파악만 → `rhwp-doc-triage`
- 서식 1건 채우기·순번 → `rhwp-form-fill`
- 표 CSV 왕복 → `rhwp-table-exchange`
- 배포 전 점검 → `rhwp-security-sweep`
- 원본을 계획서로 여러 번 고침 → `rhwp-safe-edit`
- MCP 로 붙이기 → `rhwp-mcp-session` (`hwp_batch` 에는 convert 쓰기 축 없음)

상세: [20_handoff.md](references/20_handoff.md)

## 봉투·종료 집계

성공 레코드는 단건 `--json` 과 **같은 스키마**. fill 만 `row`(0 기준)가 붙는다.
실패 레코드는 공통: `schemaVersion` · `source` · `error` · `exitClass:"runtime"`.

| 집계 | 코드 |
| --- | --- |
| 전부 통과 | 0 |
| error 레코드가 하나라도 | 1 |
| 사용법(암호 플래그, `--query` 누락, 이름 예약 충돌, 빈 fill 데이터) | 2 |
| error 없고 `--verify` IR 차이만 | 3 |
| error 없고 `--verify-pages` 불일치 | 4 |

성공 4 + 실패 1 이면 **exit 1 이 정상**이다. 종료 코드는 집계이고, 행별 판정은 NDJSON.

게이트 (기계):

```bash
입력=$(wc -l < 목록.txt)
성공=$(jq -s '[.[]|select(.error|not)]|length' 결과.ndjson)
실패=$(jq -s '[.[]|select(.error)]|length' 결과.ndjson)
echo "입력 $입력 = 성공 $성공 + 실패 $실패"
test "$입력" -eq $((성공 + 실패))
```

jq 분리·재시도: [13_jq_split_retry.md](references/13_jq_split_retry.md).
종료 코드: [18_exit_aggregation.md](references/18_exit_aggregation.md).

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_stdin_ndjson.md](references/01_stdin_ndjson.md) — stdin / stdout / stderr
3. [02_failure_envelope.md](references/02_failure_envelope.md) — 실패 봉투
4. [03_order_threads.md](references/03_order_threads.md) — `--threads` 와 순서
5. [04_axis_info.md](references/04_axis_info.md) — batch info
6. [05_axis_export_text.md](references/05_axis_export_text.md) — export-text
7. [06_axis_export_structure.md](references/06_axis_export_structure.md) — export-structure
8. [07_axis_export_tables.md](references/07_axis_export_tables.md) — export-tables
9. [08_axis_fields.md](references/08_axis_fields.md) — fields
10. [09_axis_search.md](references/09_axis_search.md) — search
11. [10_axis_extract_data.md](references/10_axis_extract_data.md) — extract-data
12. [11_axis_convert.md](references/11_axis_convert.md) — convert
13. [12_axis_fill.md](references/12_axis_fill.md) — fill
14. [13_jq_split_retry.md](references/13_jq_split_retry.md) — jq 분리·재시도
15. [14_gate_n_equals.md](references/14_gate_n_equals.md) — N=성공+실패
16. [15_no_global_password.md](references/15_no_global_password.md) — 전역 암호 금지
17. [16_convert_name_reservation.md](references/16_convert_name_reservation.md) — 이름 예약
18. [17_fill_not_stdin.md](references/17_fill_not_stdin.md) — fill 입력 축
19. [18_exit_aggregation.md](references/18_exit_aggregation.md) — 종료 집계
20. [19_pitfalls.md](references/19_pitfalls.md) — 함정
21. [20_handoff.md](references/20_handoff.md) — 인계
22. [21_journeys.md](references/21_journeys.md) — 실사용 여정
23. [22_intent_matrix.md](references/22_intent_matrix.md) — 발화 → 명령
24. [23_envelopes.md](references/23_envelopes.md) — 축별 봉투
25. [24_stderr_summary.md](references/24_stderr_summary.md) — stderr 요약
26. [25_listing.md](references/25_listing.md) — 목록 만들기
27. [26_worked_traces.md](references/26_worked_traces.md) — 재현 트레이스
28. [27_gate_recipes.md](references/27_gate_recipes.md) — jq 게이트
29. [28_retry_classes.md](references/28_retry_classes.md) — 재시도 부류
30. [29_windows_powershell.md](references/29_windows_powershell.md) — PowerShell
31. [30_corpus.md](references/30_corpus.md) — 표본 코퍼스
32. [31_folder_menu.md](references/31_folder_menu.md) — 폴더 유형별 축
33. [README.md](references/README.md) — 장 안내

예제·NDJSON 전사: `examples/`. 기계 가독 픽스처: `fixtures/`.

## 예제 목차

1. [examples/01_list_then_info.md](examples/01_list_then_info.md)
2. [examples/02_export_text_retry.md](examples/02_export_text_retry.md)
3. [examples/03_extract_data_harvest.md](examples/03_extract_data_harvest.md)
4. [examples/04_convert_reserve.md](examples/04_convert_reserve.md)
5. [examples/05_fill_mailmerge.md](examples/05_fill_mailmerge.md)
6. [examples/06_search_archive.md](examples/06_search_archive.md)
7. [examples/07_structure_outline.md](examples/07_structure_outline.md)
8. [examples/08_tables_harvest.md](examples/08_tables_harvest.md)
9. [examples/09_fields_survey.md](examples/09_fields_survey.md)
10. [examples/10_mixed_failure_gate.md](examples/10_mixed_failure_gate.md)
11. [examples/11_password_split.md](examples/11_password_split.md)
12. [examples/12_windows_listing.md](examples/12_windows_listing.md)
