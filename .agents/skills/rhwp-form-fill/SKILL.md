---
name: rhwp-form-fill
description: rhwp CLI 로 HWP/HWPX 서식의 누름틀에 값을 채우고 메일머지 산출물을 만듭니다. fields 조사 → fill-fields 단건 채움(반복 필드 `이름[순번]` 지목) → batch fill(서식 1 + 데이터 N행) → --dry-run/--verify 판정 → sanitize 제출 정리까지 수행합니다. 트리거 — 사용자가 "이 서식/신청서/양식 채워줘", "누름틀에 값 넣어줘", "명단으로 N명분 만들어줘", "메일머지", "서식에 뭘 채워야 하는지 알려줘", "제출용으로 만들어줘" 등을 요청할 때. 판정 규칙 실측은 mydocs/manual/recipes/01·05.
---

# rhwp-form-fill — 서식 채우기·메일머지 Skill

누름틀이 있는 서식(`.hwp`/`.hwpx`)에 값을 채워 제출 가능한 산출물을 만든다.
이 스킬은 **실 에이전트 경로**다. gym 이 아니고, 새 edit 로직을 발명하지 않는다.
코어는 이미 있는 `set_field_value_by_name` / `fields` / `batch fill` / `sanitize` 를
그대로 부른다.

"값이 들어갔다"와 "제출할 수 있다"는 다른 명제다. 단계마다 기계 판정
(`notFound` / `ambiguous` / `verify`)으로 확인하고, 사람 눈만으로 넘어가지 않는다.

상세는 `references/` 를 단계별로 연다. SKILL.md 는 인덱스와 정지 규칙만 담는다.

## 바이너리

```bash
cargo build --release
./target/release/rhwp <명령> [옵션]
```

네이티브는 로컬 cargo. Docker 는 WASM 전용.
산출물은 `output/` 아래 분리 권장(gitignore). 원본은 어떤 실패에서도 불변이다.

## 사다리 (강제 순회 아님)

`fields → --dry-run → fill-fields|batch fill → --verify → [insert-image] → sanitize`

질문이 이미 답이면 다음 단으로 내려가지 않는다. 각 단의 정지 조건은
[11_failure_signals.md](references/11_failure_signals.md) 와 아래 정지 표.

```
fields --json
  ├─ exit 1 ──▶ 중단 (F01)
  ├─ fieldCount 0 ──▶ rhwp-table-exchange 인계 (F02)
  ├─ textSecurity.status ≠ clean ──▶ rhwp-security-sweep 인계 (F03)
  └─ fieldCount ≥ 1
       ├─ 조사만 ──▶ names/guide/memo 보고 정지 (F04)
       ├─ 단건 채움
       │    ├─ 같은 이름 반복 ──▶ 이름[N] (F05)
       │    ├─ --dry-run ──▶ notFound/ambiguous 확인 (F06)
       │    ├─ fill-fields -o --verify ──▶ 통과 게이트 (F07)
       │    └─ 제출 ──▶ [insert-image] → sanitize (F08)
       └─ 명단 N행
            ├─ --data .jsonl|.csv 준비 (F09)
            ├─ batch fill --dry-run (F10)
            └─ batch fill --verify --name-field? (F11)
```

## 요청 → 명령

| 사용자 요청 | 명령 | 레퍼런스 |
| --- | --- | --- |
| 이 서식에 뭘 채워야 해? | `fields <서식> --json` | 01_fields_survey.md |
| 값 채워줘 (1건) | `edit fill-fields <서식> --data '{"필드":"값"}' -o <출력> --json` | 02_fill_fields.md |
| 같은 이름 필드 중 N번째만 | `--data '{"이름[N]":"값"}'` (0 기준, #3476) | 03_repeat_occurrence.md |
| 명단으로 N명분 (메일머지) | `batch fill --form <서식> --data <행.jsonl\|.csv> --out-dir <폴더> --json` | 04_batch_fill.md |
| 채우기 전에 미리 확인만 | 같은 인자에 `--dry-run --json` | 05_dry_run_verify.md |
| 채워졌는지 재파싱까지 | `--verify` (차이 시 exit 3) | 05_dry_run_verify.md |
| 제출 전 작성자 흔적 지워줘 | `edit sanitize <파일> -o <출력> --json` | 06_sanitize.md |
| 도장/서명 붙여줘 | `edit insert-image … --page --x --y` | 14_insert_image.md |
| 누름틀이 없는 표 칸 서식 | `edit set-cell` — rhwp-table-exchange 로 전환 | 15_axis_choice.md |

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| F01 | fields/fill 이 exit 1 (파일 없음·쓰기 실패) | 중단. 원본 불변, 출력 미생성 |
| F02 | `fieldCount: 0` | 누름틀 서식이 아니다. rhwp-table-exchange 인계 |
| F03 | `textSecurity.status` ≠ `"clean"` | 채우기 전에 rhwp-security-sweep / 레시피 04 |
| F04 | 질문이 '뭘 채워야 해' 이고 fields 가 이름을 냈다 | name/guide/memo 만 보고 멈춘다 |
| F05 | 같은 이름이 여러 번, 순번 없는 키 | `ambiguous` 를 성공으로 읽지 않는다. `이름[N]` 재지목 |
| F06 | `--dry-run` 의 `notFound` 가 비어 있지 않다 | 오타. `fields --json` 의 name 을 그대로 복사 |
| F07 | 단건 `--verify` 후 `identical`·빈 notFound·빈 ambiguous | 채움 완료. 제출이면 sanitize 로 |
| F08 | 제출 요청 | insert-image(선택) 후 sanitize. 두 번째 sanitize 는 removedCount 0 |
| F09 | 데이터 파일이 헤더만 / 행 0개 | exit 2. 상류 명단 조회부터 확인 |
| F10 | batch `--dry-run` 에서 행별 notFound/ambiguous | 그 행만 고치고 실행. 요약 줄만 보지 않는다 |
| F11 | `--name-field` 컬럼이 매 행 notFound | 실패가 아니다. 게이트에서 그 컬럼을 제외 |
| F12 | `verify.identical: false` (exit 3) | 산출은 남는다. export-svg 또는 render-diff |
| F13 | 폴더에 서식 수백 + 명단 1개 | 이 스킬의 batch fill. 서식 N개는 rhwp-bulk-pipeline |
| F14 | 사용자 질문이 이미 답변 가능 | 다음 단으로 내려가지 않는다 |

**금지 기본값**

- 새 fill/merge 로직·새 CLI 플래그 발명
- gym pack / gym 과제 작성
- `ambiguous` 를 침묵 성공으로 제출
- `--name-field` 컬럼의 notFound 를 실패로 오탐
- `batch fill` 에 stdin 파일 목록을 파이프 (이 축은 `--form`+`--data`)
- 원본 `--in-place` 로 덮어쓰기
- 머리말/꼬리말·각주 안의 필드를 fields 가 다 잡았다고 단정
- 이 스킬 안에서 onboarding / mcp-session / safe-edit / provenance / doc-triage 를 재작성

## 인계

- 표 칸 서식 → `rhwp-table-exchange`
- 배포 전 점검·주입 → `rhwp-security-sweep`
- 원본을 계획서로 여러 번 고침 → `rhwp-safe-edit`
- 폴더 수백 서식 스윕 → `rhwp-bulk-pipeline`
- 미지 문서 파악만 → `rhwp-doc-triage` (읽기, 채우지 않음)

상세: [10_handoff.md](references/10_handoff.md)

## 봉투

모든 질의는 `--json` 에서 stdout 순수 JSON. 실패 시 stdout 비움.
`schemaVersion:"1.0"`. 종료 코드 #2707: 0 성공 · 1 런타임 · 2 사용법 · 3 `--verify` IR 차이.

`fill-fields` / `batch fill` 공통:

`{"schemaVersion","source","dryRun","filledCount","filled":[{name,occurrence,value}],"notFound","ambiguous","output"?,"outputFormat"?}`

- `notFound` — 없는 이름 또는 범위 밖 순번. 조용히 무시하지 않는다.
- `ambiguous` — `{name, matched, total}`. 순번 없는 키가 여러 곳.
- `output` / `outputFormat` 은 실제 저장했을 때만 (`--dry-run` 이면 없음).
- `outputFormat` 은 입력 형식 보존 (#3383): HWPX → `"hwpx"`, HWP5/HWP3 → `"hwp5"`.
- batch 는 `row`(0 기준). `--verify` 시 `verify:{identical,diffCount}`.
- batch 요약은 stderr. stdout 은 NDJSON 뿐.

필드 표: [07_envelopes.md](references/07_envelopes.md)

## 통과 게이트 (기계)

```bash
# 단건
rhwp edit fill-fields 신청서.hwp --data @row.json -o out.hwp --verify --json \
  | jq -e '.verify.identical and (.notFound|length==0) and (.ambiguous|length==0)' \
  > /dev/null || { echo "채움 실패"; exit 1; }

# batch — --name-field 컬럼 "성명" 은 notFound 오탐에서 제외
jq -es 'map(select(((.notFound - ["성명"])|length>0) or (.ambiguous|length>0)
        or (.verify != null and .verify.identical==false)))
        | if length==0 then "OK" else error("실패 행 \(length)건") end' filled.ndjson
```

요약 줄("N행 중 M 성공")만 보고 넘어가지 않는다. 게이트는 행별 레코드로 판정한다.

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_fields_survey.md](references/01_fields_survey.md) — fields 조사
3. [02_fill_fields.md](references/02_fill_fields.md) — 단건 채움
4. [03_repeat_occurrence.md](references/03_repeat_occurrence.md) — `이름[순번]`
5. [04_batch_fill.md](references/04_batch_fill.md) — 메일머지
6. [05_dry_run_verify.md](references/05_dry_run_verify.md) — dry-run / verify
7. [06_sanitize.md](references/06_sanitize.md) — 제출 정리
8. [07_envelopes.md](references/07_envelopes.md) — 봉투 필드
9. [08_pitfalls.md](references/08_pitfalls.md) — 함정
10. [10_handoff.md](references/10_handoff.md) — 다른 스킬로
11. [11_failure_signals.md](references/11_failure_signals.md) — 신호 → 처방
12. [12_data_formats.md](references/12_data_formats.md) — JSON / @파일 / CSV / JSONL
13. [13_name_field.md](references/13_name_field.md) — 산출 파일명
14. [14_insert_image.md](references/14_insert_image.md) — 직인·서명
15. [15_axis_choice.md](references/15_axis_choice.md) — fill-fields vs set-cell
16. [16_worked_traces.md](references/16_worked_traces.md) — 재현 트레이스
17. [17_intent_matrix.md](references/17_intent_matrix.md) — 발화 → 명령
18. [18_field_catalog.md](references/18_field_catalog.md) — 표본 필드
19. [19_gate_recipes.md](references/19_gate_recipes.md) — jq 게이트
20. [20_exit_codes.md](references/20_exit_codes.md) — 종료 코드
21. [09_journeys.md](references/09_journeys.md) — 실사용 여정

기계 가독 픽스처: `references/fixtures/`.

## 권위

- [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
  (`fields` · `edit fill-fields` · `batch fill` · `edit insert-image` · `edit sanitize` · 종료 코드)
- [`recipes/01_fill_form_and_submit.md`](../../../mydocs/manual/recipes/01_fill_form_and_submit.md)
- [`recipes/05_mail_merge_batch_fill.md`](../../../mydocs/manual/recipes/05_mail_merge_batch_fill.md)
- [`form_filling_guide.md`](../../../mydocs/manual/form_filling_guide.md)
- 처리 결과: [`mydocs/working/agent_form_fill.md`](../../../mydocs/working/archives/agent_form_fill.md)
