---
kind: working
status: active
issue: 5469
---

# M-hwp5 저장 계약 인벤토리 픽스처 고도화 (#5469)

작업 브랜치: `feat/m-hwp5-inventory-fatten`
대상: `tools/hwp5_inventory/` · `mydocs/working/hwp5_inventory_fatten.md`
이슈: [M-hwp5: 저장 계약 인벤토리 픽스처 고도화](https://github.com/edwardkim/rhwp/issues/5469)

## 1. 한 줄

HWPX→HWP 저장 계약의 P0 언어(`hwp5-inventory` / `hwp5-inventory-diff` /
`hwp5-table-probe`)를 태그·컨트롤·테이블 필드·실패 유형 A–F·63개 계약
단위 픽스처와 CLI 모양 전사로 고정한다. 시리얼라이저 페이지 수 로직은
만지지 않는다. `#4882` 석이 맡는다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 기존 inventory-diff / table-probe 픽스처·리포트 고도화
- 추가 10000–20000줄, 최소 10000. 패딩 금지
- `cargo fmt --all -- --check`
- base `devel`, 한국어 PR, `closes #5469`
- isolation worktree, 브랜치 `feat/m-hwp5-inventory-fatten` from `upstream/devel`

금지:

- 시리얼라이저 페이지 수 로직 (`#4882` 분리)
- `canvaskit_policy` · `pdf` · `layout-anomaly` · `oracle_public` ·
  `render_backend` · `proptest` · `fidelity_compare`
- `gym/`
- `git add -A`
- 본진 `rhwp` / `rhwp-desk*` / `rhwp-handoff` / `rhwp-scaffold-final` /
  `rhwp-doc-repro`

## 3. 왜 픽스처만 키우나

devel 에는 이미 세 명령이 있다.

| 명령 | 파일 | 하는 일 |
|---|---|---|
| `hwp5-inventory` | `src/diagnostics/hwp5_inventory.rs` | DocInfo/BodyText 를 안정 행으로 푼다 |
| `hwp5-inventory-diff` | `src/diagnostics/hwp5_inventory_diff.rs` | index/LCS 정렬 + 5개 report |
| `hwp5-table-probe` | `src/diagnostics/hwp5_table_probe.rs` | TABLE 네 축을 이식한 probe HWP |

구멍은 명령은 있는데, 계약 단위를 같은 언어로 모아 둔 픽스처가 거의
없다는 점이다. `tests/cli_exit_codes_hwp5_inventory_anchor.rs` 는 종료
코드만 본다. 표 바깥 여백과 lineSeg 복사와 필드 fourcc 붕괴가 어떤
`diff_kind` / `focus` / probe variant 로 떨어지는지 표본이 없다.

이 파동은 그 표본을 채운다. 저장기를 고치지 않는다.

## 4. 범위

만진 것:

| 경로 | 역할 |
|---|---|
| `tools/hwp5_inventory/catalog.py` | 태그·컨트롤·필드·A–F·종료 코드 |
| `tools/hwp5_inventory/model.py` | inventory 행, index/LCS, table-fields, probe 축 |
| `tools/hwp5_inventory/cases.py` | 63 계약 단위 |
| `tools/hwp5_inventory/render.py` | CLI 제목과 같은 Markdown 전사 |
| `tools/hwp5_inventory/fatten_catalog.py` | 디스크에 픽스처·전사·리포트 |
| `tools/hwp5_inventory/fixtures/` | 카탈로그·케이스·인벤토리·diff·probe plan |
| `tools/hwp5_inventory/transcripts/` | showcase/table 전사 |
| `tools/hwp5_inventory/reports/` | 커버리지·행렬·요약 |
| `tools/hwp5_inventory/tests/` | 33 unittest |
| `mydocs/working/hwp5_inventory_fatten.md` | 이 기록 |

만지지 않은 것:

- `src/` 전부 (진단기 구현 포함)
- `src/serializer` 페이지 수
- `tools/oracle_public` · `tools/fidelity_compare` · `tools/page_roundtrip`
- `gym/`
- `.agents/skills/` 다른 스킬

## 5. 계약 언어 (devel 정본을 그대로)

### 5.1 inventory 행

`record_uid` = `{stream}.{path}#{index}` 예: `BodyText.Section0#4`.
`tuple_role` 은 태그로 고정된다. `CTRL_HEADER` 만 `control_id` /
`control_name` 을 가진다. 필드 fourcc(`%clk`) 의 `ctrl_name` 은
devel 과 같이 `Unknown` 이다. 정체성은 fourcc 바이트에 있다.

### 5.2 정렬

- `index` : `record_uid` 키. `tag_changed` / `size_changed` /
  `payload_changed` / `scope_changed` / `control_changed` 를 따로 낸다.
- `lcs` : 스트림별 `tag|role|ctrl` 서명. 중간 삽입은 extra/missing 으로
  떨어지고 뒤 레코드는 pair 로 남는다.

중간 삽입이 있으면 `lcs` 를 쓴다. `X01` 이 그 표본이다.

### 5.3 report / focus

`diff` · `hints` · `bundles` · `table-fields` · `table-probe-plan`
`all` · `table` · `shape` · `ctrl` · `missing` · `docinfo`

`table` 후보는 `tuple_role == table` 이거나 `control_name == Table`.

### 5.4 table-probe 축

| 축 | 레코드 | 바이트 |
|---|---|---|
| `ctrl_outer_margin` | CTRL_HEADER(Table) | 0x1c..0x23 여백 4필드 |
| `ctrl_common_attr` | CTRL_HEADER(Table) | 0x04 공통 속성 |
| `table_attr` | TABLE | 0x00 첫 4바이트 |
| `table_tail` | TABLE | 0x16 이후 전체 tail |

8 variant (`01`…`08`) 는 한 축 / 결합 / 전체 guard. `08` 만 성공하고
`01`–`04` 가 침묵하면 원인을 분리하지 못한 것이다.

`tail_after_0x16` · `z_order_or_instance` 는 P0 관찰명이다. 확정 계약
이름이 아니다.

### 5.5 종료 코드

인자 없음 = 2, `--help` = 0, 없는 파일 = 1.
stdout 은 데이터, 사용법은 stderr.
`fixtures/cli_contract.json` 이 같은 표를 든다.

## 6. 실패 유형 A–F → inventory

정본은 `mydocs/troubleshootings/hwpx2hwp-rule.md` §5.

| 코드 | 이름 | 이 픽스처가 보는 칸 | 대표 케이스 |
|---|---|---|---|
| A | Container / Stream | `stream_path`, `section`, DocInfo `section_count` | D01, D05, D06, X08 |
| B | Record Tree | `missing`/`extra`, `scope_path`, 다음 태그 | T05, S03, C01, G02 |
| C | Count / Size / Reference | `size`, `payload_hash`, TABLE.rows | T03, T06, D02, F04 |
| D | DocInfo / BinData | `BIN_DATA`, face/char shape 표 | S01, D03, D04 |
| E | Missing HWP Defaults | margin/attr/기본 payload | T01, T02, T04, G01 |
| F | Layout-computed | `PARA_LINE_SEG` payload | P01, P05 |

페이지 나눔 결과가 달라도 이 파동은 쪽수 계산기를 고치지 않는다.
`D01` 은 `DOCUMENT_PROPERTIES.section_count` 필드만 고정한다.

## 7. 케이스 가족

| 가족 | 예 | 다음 도구 |
|---|---|---|
| table | T01–T10 | `hwp5-table-probe` 01–08 |
| shape | S01–S07 | `hwp5-ctrl-data-trace`, BinData 튜플 |
| para | P01–P05 | oracle `PARA_LINE_SEG`, char_count |
| docinfo | D01–D08 | DocInfo 를 BodyText 보다 먼저 |
| equation/note/form | C01–C08 | 컨트롤 다음 concrete record |
| field | F01–F04 | fourcc 정체성, `#4896` command |
| page | G01–G05 | SectionDef 튜플. 쪽수 로직 아님 |
| 정렬 가드 | X01–X16 | index vs lcs, sentinel |

`X16` 은 oracle == generated sentinel. diff_count 0. probe 를 만들지 않는다.

## 8. 생성물

```text
python tools/hwp5_inventory/fatten_catalog.py
```

- `fixtures/tags.jsonl` · `controls.jsonl` · `fields.jsonl`
- `fixtures/failure_classes.json` · `cli_contract.json`
- `fixtures/cases/<id>.json` (63)
- `fixtures/inventories/<id>.{oracle,generated}.jsonl`
- `fixtures/diffs/<id>.{index,lcs}.jsonl`
- `fixtures/table_probe/<id>.plan.json` (표/후보)
- `transcripts/` showcase + table 가족
- `reports/coverage.md` · `pair_index.md` · `failure_class_matrix.md` ·
  `probe_axis_matrix.md`

바이너리 HWP 를 쓰지 않는다. `rhwp` 빌드가 필요 없다.

## 9. 시험

```text
python -m unittest hwp5_inventory.tests.test_cases hwp5_inventory.tests.test_model hwp5_inventory.tests.test_fatten_catalog hwp5_inventory.tests.test_transcripts
```

`PYTHONPATH=tools`. 33 passed.

- 케이스 id/샘플 유일, A–F·가족 커버
- index payload_changed, LCS 삽입, table margin/tail 축
- 생성기가 카탈로그·케이스·리포트를 씀
- 전사가 CLI 제목(`# HWP5 Inventory Diff` 등)과 8 variant 이름을 유지
- `X16` diff_count 0
- 페이지 export 처방을 쓰지 않음

`cargo test` / clippy 는 Rust 원본이 없어 해당 없음.
`node scripts/rust-test-suite-manifest.mjs --check` 도 해당 없음.

## 10. fmt 게이트

```text
cargo fmt --all -- --check
```

Rust 파일을 고치지 않았다. 게이트는 devel 과 같은 상태여야 한다.

## 11. PR 메모

- base `devel`
- head `kevin9327:feat/m-hwp5-inventory-fatten`
- `--body-file` UTF-8 without BOM
- 본문 `closes #5469`
- 첫 체크박스: `cargo fmt --all -- --check`

## 12. 다음 사람

1. 표 조판이 틀리면 `T01`–`T04` 전사와 `table-probe-plan` 을 먼저 읽는다.
2. 그림이 빠지면 `S01` + BinData 튜플. CTRL_HEADER 만 보지 않는다.
3. 문단 직후 손상이면 `P02`/`P03`. lineSeg 를 복사하지 말라는 규칙은 `P01`.
4. 구현은 oracle tuple 이 문서화되고 probe 가 한 축으로 닫힌 뒤에만.

구현 근거는 한컴 통과 산출물이 아니라 oracle-derived lowering contract 다.
