---
kind: working
status: done
canonical: mydocs/working/task_m100_5652_stage4.md
last_verified: 2026-08-23
---

# #5652 Stage 4 — CLI `--structure` 옵트인 · `set-chart-data --dry-run` 코어 경유 · MCP/프로필/문서

- **계획서**: [`mydocs/plans/task_m100_5652.md`](../../plans/archives/task_m100_5652.md) §3-5
- **브랜치**: `task5652` (`upstream/devel` `bf30bd792` 기준)

## 1. 무엇을 만들었나

| 표면 | 변경 |
|---|---|
| `src/cli/commands/tabular_import.rs` `csv_to_chart` | `--structure` 플래그 → 코어 edits 에 `"structure": true`. 플래그 없으면 B1 그대로(치수 불일치 exit 2). 독스트링·사용법 갱신 |
| `src/cli/commands/edit/document_objects.rs` `edit_set_chart_data` | `--dry-run` 도 코어 호출(`dryRun: true` 주입) — 가드가 dry-run 에서 발화하고 `changed[].op` 가 보인다. 봉투에 `changedCount`·`changed`·`wrote` 추가 |
| 자기서술 3면 | `capabilities/extended.rs`(설명·`--structure` 플래그), `help/public.rs`(옵션 줄·가드 안내), `mcp/read.rs`(`hwp_csv_to_chart` 설명·`structure` 속성·optional_args), `mcp/advanced.rs`(`hwp_set_chart_data` 설명·`data` 스키마 문구·outputFields += `changedCount`·`changed`·`wrote`·`invalid`), `agent_profiles.rs`(데이터분석 recipe 1줄 추가) |
| 출처 표지 | `crates/rhwp-contracts/src/provenance.rs` — `edit` 에 `changed[].from`(set-chart-data 변경 전 값) 추가, `csv-to-chart` 기원·note 갱신. 스킬 픽스처 `command-untrusted-fields.json`·`command-field-catalog.md` 동기 |
| 문서 | `cli_commands.md`(csv-to-chart `--structure` 절·가드·행렬 규칙·예시, set-chart-data `--data` 스키마·dry-run), `agent_knowledge_map.md`(`changed[].op`·구조 거부 사유), codex `20_표와_데이터.md`·`70_자기서술.md` 재생성(`RHWP_BIN=target/debug/rhwp python tools/gen_agent_codex.py`) |
| 테스트 | 신규 `tests/cases/csv_to_chart_structure_contract.rs` 6건 |

## 2. 결정과 근거

- **CLI 는 검증기를 갖지 않는다** — `--structure` 는 불리언 하나를 코어에 넘길 뿐이고 가드·규칙·거부
  사유는 코어 `validate_structure` 에서 그대로 흐른다(`agent_surface_playbook` 규칙 2). 그래서
  `csv-to-chart` 의 `result["ok"] != true` 분기는 한 줄도 바뀌지 않았다.
- **`edit set-chart-data --dry-run` 의 현행 동작을 고쳤다** — 코어를 건너뛰어 가드가 침묵하던 것이
  안전 규약(`rhwp-safe-edit` "dry-run 으로 먼저")과 충돌했다. `dryRun: true` 를 주입하므로 코어는
  한 바이트도 쓰지 않고, `finish_edit_write(dry_run=true)` 가 파일을 만들지 않는다
  (`set_chart_data_contract::dry_run_no_file` 무수정 green).
- **`changed[].from` 출처 표지** — set-chart-data 봉투에 코어 diff 가 실리면서 문서 파생 값(변경 전
  값·계열명·라벨)이 `edit` 봉투에 새로 등장했다. 지도(`provenance.rs`)와 스킬 픽스처·카탈로그를 함께
  갱신했다(`agent_provenance_skill_contract::fixture_commands_match_provenance_map` 이 강제).
- **codex 는 차트·출처 장만 재생성** — `--check` 가 6장 drift 를 보고했으나 `10_조회`(info
  `lastSavedWith`, #5935)·`40_변환과_렌더`(Windows 경로 구분자)·`30`/`50`(`outputSha256` — 재생성마다
  바뀌는 비결정 값)은 이 브랜치 이전의 drift·플랫폼 노이즈라 되돌렸다. `20`·`70` 만 커밋한다.
- `mcp_integration_guide.md` 에는 차트 행이 없어 갱신 대상이 아니다.

## 3. 판정 (`tests/cases/csv_to_chart_structure_contract.rs`)

| 테스트 | 고정 내용 |
|---|---|
| `extra_row_without_structure_flag_is_refused_with_a_hint` | 플래그 없으면 exit 2 + `valueCountMismatch`, 메시지에 `structure` 안내, 파일 없음 |
| `structure_flag_appends_a_row_and_round_trips_through_chart_to_csv` | `--structure` 행 추가 → wrote 2 · `appendPoints` → `chart-to-csv` 재추출에 6행 |
| `structure_flag_adds_and_removes_series_and_renames` | 열 추가(`appendSeries`), 열 삭제+계열명 변경(`truncateSeries`·`renameSeries`) 왕복 |
| `pie_series_add_via_csv_exits_two_with_pie_guard` | 원형 계열 추가 CSV → exit 2 + `pieSeriesCountFixed`, 파일 없음 |
| `set_chart_data_dry_run_goes_through_core_validation` | dry-run 가드 거부 exit 2; 통과 dry-run 은 `dryRun:true`·`changedCount>0`·`appendPoints`·파일 없음 |
| `capabilities_mcp_and_help_declare_structure` | capabilities flags·`--mcp` `hwp_csv_to_chart.structure`·`hwp_set_chart_data` data 설명·outputFields `invalid`·`--help` |

### 게이트 실측 (2026-08-23)

| 게이트 | 결과 |
|---|---|
| `csv_to_chart_structure_contract` 6 · `chart_csv_contract` 17 · `set_chart_data_contract` 4 · `charts_contract` 3 · `agent_codex_contract` 2 · `agent_codex_skill_contract` 20 · `agent_provenance_skill_contract` 12 · `provenance_contract` 10 · `agent_profile_router_contract` 8 · `capabilities_schema_contract` 17 · `capabilities_subcommands_contract` 4 · `cli_catalog_contract` 19 · `cli_scoped_help_contract` 9 · `mcp_tool_annotations_contract` 5 · `agent_surface_skill_contract` 9 | 전건 passed |
| fmt `--check` · suite-manifest `--prepare`→`--check` · unit-tiers `--base-ref upstream/devel`(4225 불변) · clippy `-D warnings` | 통과 |

## 4. 확인된 것

- 에이전트 경로 전체(CSV → `--structure` → 봉투 `changed[].op`/`invalid[].reason`)가 코어 한 곳의
  판정을 그대로 실어 나른다. `hwp_csv_to_chart structure=true` 와 `hwp_set_chart_data data.structure=true`
  가 같은 코어 계약을 쓴다.

## 5. 다음

S5 — 엔진 산출 회귀(렌더 반영·12변종 재스캔) + 한컴 판정 번들 생성기 → 사용자 판정 → 자산·원장·트립와이어·보고서.
