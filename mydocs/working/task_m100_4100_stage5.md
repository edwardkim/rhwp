---
kind: working
status: done
canonical: mydocs/working/task_m100_4100_stage5.md
last_verified: 2026-08-11
---

# #4100 Stage 5 — CSV 왕복 + CLI 2명령 + 배선 4곳

- **계획서**: [`mydocs/plans/task_m100_4100.md`](../plans/task_m100_4100.md)
- **기준 커밋**: `devel = dd9ecdc4b`
- **산출**: `src/document_core/queries/chart_csv.rs`(신규) · `src/main.rs`(디스패치 2 + 핸들러 2
  + 배선 3) · `src/provenance.rs`(항목 2) · `tests/chart_csv_contract.rs`(신규 14건) ·
  `tests/provenance_contract.rs`(레시피 4) · `mydocs/manual/cli_commands.md`

## 1. 실사용 문서가 조용한 데이터 소실을 드러냈다

배선 게이트(`provenance_contract`)는 새 `--json` 명령을 **실제로 실행해 보라**고 요구한다.
코퍼스 대신 실사용 보고서(`samples/issue2006/1790387_prep_final_report.hwpx`)를 넣자:

```text
chart 1   rowCount 0 x colCount 6
,성생활 함,흥미 없음,파트너 없음,경제적 이유,시간적 이유,신체적 이유
(끝 — 데이터 행 0개)
```

**6계열 × 6값 = 36칸이 통째로 사라진 CSV.** 오류도 경고도 없었다.

### 1-1. 근인 둘

그 문서는 계열 6개 중 **한 계열에 `c:cat` 이 없다**(`c:ser` 6 · `c:cat` 5 · `c:val` 6).

1. 코어가 `series[0]` 의 라벨을 문서 전체 라벨로 보고했다 — 그게 하필 라벨 없는 계열이라
   빈 목록이 나왔다
2. `chart_csv::to_csv` 가 행 수를 `labels.len()` 으로 잡아 데이터 행이 0개가 됐다

### 1-2. 고친 방향 — 행 수는 값이 정한다

- `to_csv` 의 행 수 = `max(labels.len(), 각 계열 값 개수)`. **라벨은 장식이지 데이터의
  길이가 아니다.** 모자란 칸은 빈 문자열로 채운다
- 라벨 조회를 "라벨을 **가진** 첫 계열"로 바꿨다(`label_texts`)
- 라벨 대조에서 라벨 없는 계열을 뺐다 — **없는 것과 다른 것은 다르다**

### 1-3. 왜 코퍼스로는 안 잡혔나

`samples/chart/` 28종은 **전건이 계열마다 `c:cat` 을 갖고 있다.** Stage 1~4 에서 여러 번
"코퍼스 56건 전건 green" 을 보고했지만 이 변종은 그 56건 안에 아예 없었다. 배선 게이트가
실사용 문서를 강제로 태우지 않았다면 PR 까지 갔을 결함이다.

회귀 가드는 두 층에 뒀다 — `chart_csv::values_survive_even_when_labels_are_missing`(단위) ·
`charts_with_partial_category_labels_keep_every_value`(코어) ·
`charts_with_missing_category_labels_still_emit_every_value`(CLI).

## 2. 결정과 근거

### 2-1. CLI 는 얇다 — 검증은 전부 코어에

`csv-to-chart` 가 하는 일은 CSV 를 행렬로 읽어 코어에 넘기고 봉투를 실어 나르는 것뿐이다.
계열 수·값 개수·계열명·라벨·수치 여부는 **`set_chart_data_by_index_native` 하나가** 본다.
검증기가 코어와 CLI 로 갈리면 둘이 서로 다른 것을 허용하기 시작한다.

CLI 가 자체로 거부하는 것은 **CSV 가 표로 성립하는가** 하나다(`csvParse` — 톱니 행·빈 CSV·
계열 열 부재). 그건 차트와 무관한 별개 관심사다.

### 2-2. `--chart N` 은 1부터다

표 CSV 의 `--table` 은 `export-tables` 의 index 라 0부터인데, 차트에는 그런 발견 명령이
없어 번호가 **문서 순서 그 자체**다. 사람이 세는 대로 1부터로 잡고 문서에 명시했다.

### 2-3. 나머지는 `table-to-csv` 규약을 그대로 따른다

`-o` 의 뜻이 `--chart` 유무로 갈리는 것, `--bom` 이 **파일에만** 붙고 봉투의 `csv` 문자열에는
붙지 않는 것, `-o` 도 `--json` 도 없으면 본문을 stdout 으로 흘리는 것까지 같다. 같은 도구로
표와 차트를 왕복시켜야 하므로 규약이 갈리면 안 된다.

## 3. 배선 4곳

| 축 | 한 일 |
|---|---|
| capabilities | `chart-to-csv`(export) · `csv-to-chart`(edit) 항목. `wrote` 를 record field 로 선언 |
| MCP | `hwp_chart_to_csv` · `hwp_csv_to_chart`. 설명에 "두 표현에 함께 쓴다"를 명시 |
| help | 2칸 들여쓰기 형식(테스트 파서 계약) |
| provenance | `charts[].csv`(문서 파생) · `changed[].from`(편집 전 문서 값) |

**배선은 4곳이 아니라 6곳이었다.** 계획서가 센 4곳 말고 둘이 더 있었고, 둘 다 전체 회귀에서만
드러났다(대상 테스트를 골라 돌릴 때는 안 걸린다).

| 빠뜨린 축 | 잡은 게이트 | 안 하면 |
|---|---|---|
| `agent_profiles.rs` (계획서는 "선택"이라 적었다) | `agent_profile_router_contract::every_stateless_tool_belongs_to_some_specific_profile` | 도구가 존재하되 **업무 프로필로 좁혀 쓰는 에이전트에게는 안 보인다** |
| 지식지도 §2-2 전수 사전 | `knowledge_map_field_dictionary_contract::every_declared_record_field_is_in_the_dictionary` | 매니페스트가 내는 필드를 가이드가 설명하지 못한다 (`chart`·`chartCount`·`charts`·`wrote` 4개) |

차트 CSV 두 도구는 표 CSV 와 같은 **데이터분석** 프로필에 넣고 레시피 두 줄을 붙였다.
지식지도에는 `#### 차트 (#4100)` 소절을 만들고 헤딩의 필드 수를 188 → 192 로 갱신했다.
`chart` 가 **1부터**라는 것(표의 `table` 은 0부터)을 사전에 못박았다 — 두 규약이 다르다.

계획서의 배선 표를 정정했다. 다음 사람이 같은 곳을 빠뜨리지 않게 한다.

`provenance_contract` 는 선언한 필드가 **실물 봉투에 나타나는지**까지 본다. `output`·
`outputFormat`·`verify` 는 파일 산출에서만 나오므로 `-o` 를 준 레시피를 따로 넣었다
(`table-to-csv` 선례와 같다).

## 4. 판정

```text
tests/chart_csv_contract.rs        14 passed   (신규 — CLI 계약)
tests/issue_4100_chart_data_edit   29 passed, 1 ignored
tests/provenance_contract          10 passed
tests/cli_json_contract            31 passed
tests/table_csv_contract           14 passed
cargo fmt --check                  Diff in 0건
cargo clippy --all-targets -D warnings   exit 0
git diff --check                   통과
```

CLI 계약이 덮는 것: 봉투 직사각성 · 라벨 결손 회귀 · 분산형 `X` 표식 · stdout 파이프 ·
`--bom` 이 파일에만 · 없는 차트 번호는 exit 1 · 무편집 왕복 `changedCount 0` ·
편집이 `wrote:["zipPart","nestedCopy"]` · `--dry-run` 무기록 · 불일치 exit 2 + 파일 미생성 ·
비수치 거부 · 톱니 CSV `csvParse` · 필수 인자 누락 exit 2 · 4면 배선 선언.

## 5. 다음 (Stage 6)

한컴 판정 번들(7종 × `.hwpx`/`.hwp` 편집본 + 대조군 = 21 파일) + 최종 보고서.
원형·분산형·주식형·ofPie 는 이번이 첫 한컴 판정이다.
