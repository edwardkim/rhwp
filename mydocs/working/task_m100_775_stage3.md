# Task #775 Stage 3 보고서 — cargo test --release 전체 회귀 검증

## 결과 요약

- ✅ **총 통과 1338 / 실패 0 / ignored 5** — 회귀 0
- ✅ Issue #775 본 테스트 GREEN
- ✅ Issue #703 본 케이스 (calendar_year) GREEN
- ✅ Issue #704 (통합재정통계) ignored 유지
- ✅ Issue #643/#712/#713/#716 등 후속 회귀 가드 모두 GREEN

## 명령

```bash
cargo test --release
```

## 집계

```
Compiling rhwp v0.7.10 (/Users/planet/rhwp)
warning: unnecessary parentheses around assigned value (기존 — 본 변경 무관)
warning: function `footnote_emits_autoNum` (기존 — 본 변경 무관)
warning: function `test_merge_then_control_layout_has_colSpan` (기존 — 본 변경 무관)
... (5 warnings — 기존 코드 영역, 본 변경에 의한 신규 경고 0)

Finished `release` profile [optimized] target(s) in 1m 36s

총 통과: 1338  실패: 0  ignored: 5
FAILED 줄: 0
panic 줄: 0
```

## 주요 가드 통과 확인

### 본 task 회귀 차단 가드

| 테스트 | 결과 |
|--------|------|
| `tests/issue_775.rs::issue_775_exam_eng_p4_pi181_table_at_column_top` | ✅ ok |

### 회귀 위험 영역 — Issue #703 + 후속 회귀 task

| 테스트 | 결과 |
|--------|------|
| `tests/issue_703.rs::issue_703_calendar_year_single_page` | ✅ ok (Task #703 본 케이스 보존) |
| `tests/issue_703.rs::issue_703_tonghap_2010_11_single_page` | ⏸ ignored (Issue #704) |
| `tests/issue_703.rs::issue_703_tonghap_2011_10_single_page` | ⏸ ignored (Issue #704) |
| `tests/issue_643.rs` | ✅ ok |
| `tests/issue_712.rs` | ✅ ok |
| `tests/issue_713.rs` | ✅ ok |
| `tests/issue_716.rs` | ✅ ok |

### 골든 SVG 영역

```
Running tests/svg_snapshot.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured (post-Stage 2 변경에도 회귀 0)
```

`issue_147_aift_page3`, `issue_157_page_1`, `issue_267_ktx_toc_page`, `form_002_page_0`,
`issue_617_exam_kor_page5`, `table_text_page_0`, `render_is_deterministic_within_process`
모두 GREEN.

### 다단 영역 가드

```
Running tests/exam_eng_multicolumn.rs
test result: ok. (다단 본 환경 핵심 fixtures 회귀 0)
```

## 변경 영향 범위 요약

`typeset.rs:1553` 의 가드 조건은 기존 `InFrontOfText | BehindText` 매칭에 `&& st.col_count == 1` 만 추가:

| 분기 | column_count | 동작 변경 여부 |
|------|--------------|----------------|
| InFrontOfText/BehindText 표 | 1 (단일 컬럼) | 변경 없음 (Task #703 fix 그대로 적용) |
| InFrontOfText/BehindText 표 | ≥2 (다단) | 종전(Task #703 fix 이전) 동작 복귀 |
| 그 외 표 (TopAndBottom, Square, …) | * | 변경 없음 |

따라서 영향 범위는 **다단 + InFrontOfText/BehindText 표** 조합만이고, 본 fix 가 Task #703 의 본 케이스(단일 컬럼) 와 직교하므로 라이브러리 회귀 0 정합.

## Stage 4 진행 조건

- 본 단계 보고서 승인
- Stage 4: 광범위 sweep (samples/ 다단 + InFrontOfText/BehindText 케이스 + 골든 SVG 회귀 확인 + PDF 권위 자료 정합 시각 판정)
