# Task M100 #1393 최종 보고서 — 표 pageBreak 게이트 동승 (방출은 PR #1405 해소)

- 이슈: #1393 "HWPX serializer: 표 pageBreak 속성 미보존 — 일괄 TABLE 방출"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1393`
- 작성일: 2026-06-14

## 1. 결함과 해소 (역할 분리)

#1380 4단계에서 form-002 표 pageBreak 일괄 TABLE 방출로 검출. 두 축으로 분리:

| 축 | 상태 |
|----|------|
| **serializer 방출 + 매핑** | **PR #1405(physwkim) 커밋 `ad55059f`에서 해소** — `table_page_break_str`를 파서의 정확한 역(`CELL↔RowBreak`, `TABLE↔CellBreak`)으로 정정. 전수 43파일 pageBreak 멀티셋 불일치 0 |
| **게이트 사각** | **본 타스크에서 해소** — `diff_documents`가 표 page_break 미비교였음. `TablePageBreak` 동승으로 회귀 봉인 |

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | `TablePageBreak` 게이트 동승 (Table arm 1지점) + 테스트 2종 | `84740cf3` |
| 2 | 매뉴얼 + CI + 최종 보고서 | (본 커밋) |

수정 파일: `src/serializer/hwpx/roundtrip.rs` — 게이트 1지점 (방출 무변경).

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1393/`)

- PASS 53 / **IR_DIFF 0** (page_break 게이트 동승 후에도 차이 0 — 전수 보존) /
  SERIALIZE_FAIL 0 / PARSE_FAIL 1(제외) / ROUND2_DIFF 0
- baseline 4 passed — **B=0 유지, 신규 xfail 0**.

### 3.2 단위 테스트

- `task1393_table_page_break_diff_in_gate` (변형 검출) +
  `task1393_form_002_page_break_roundtrips` (실샘플 게이트 0).

### 3.3 CI급 검증 (release-test 프로필)

- `cargo test --profile release-test --tests` — **2328 passed, 0 failed** (기존 2326 + 신규 2: 게이트 검출 + 실샘플)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. 매핑 정합 (확인)

| HWPX | 파서 → IR | serializer ← IR |
|------|-----------|-----------------|
| CELL/ROW | RowBreak | RowBreak → "CELL" |
| TABLE | CellBreak | CellBreak → "TABLE" |
| (없음) | None | None → "NONE" |

한컴 HWPX "CELL" = HWP5 row-break bit 의미 (파서·serializer 주석 명시) — 역관계 정합.

## 5. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| 열거 속성 표면 표기 정합 검사 | #1402 |
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |
| numbering 등록 축 잠재 불일치 | #1409 |

신규 발견 없음. 이슈의 "표 분할 배치 시프트" 잔존분은 PR #1405 방출 정정으로
보존되어 해소 (페이지 수 시프트는 #1388 여백 해소로 이미 정리됨).

## 6. 산출물

- 계획서: `mydocs/plans/task_m100_1393{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1393_stage1.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (게이트 항목 + #1393 해소)
- 검증 산출물: `output/poc/task1393/`
