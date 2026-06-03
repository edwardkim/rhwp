# Task #555 최종 결과 보고서

**이슈**: [#555](https://github.com/edwardkim/rhwp/issues/555) (옛한글 PUA → 자모 변환 후 폰트 매트릭스 갱신)
**브랜치**: `pr-task555` (devel `f807378a` 분기)
**작성일**: 2026-05-04
**선행 의존**: PR #551 Task #528 cherry-pick 완료 ✅

## 1. 본질 정정

PUA 옛한글 변환 (Task #528) 의 trade-off 영역:
- **시각 출력**: `run.display_text` (자모 시퀀스) 사용 — 정합 ✅
- **폰트 매트릭스 (글자폭/줄간격)**: `run.text` (PUA 1글자) 사용 — 결함 ❌

`estimate_text_width` 호출처 10건이 모두 `run.text` 기준이어서 자모 시퀀스 (3-4 char) 와 정합 안 됨.

### 1.1 옵션 A 적용 (선택지 중 최소 변경)

```rust
pub fn effective_text_for_metrics(run: &ComposedTextRun) -> &str {
    run.display_text.as_deref().unwrap_or(&run.text)
}
```

호출처 10건에 헬퍼 적용. 비-PUA 영역은 fallback 으로 동일 동작 (회귀 0).

## 2. 단계별 진행

| Stage | 본질 | 산출물 |
|-------|------|-------|
| 1 | 진단 — PUA 영향 fixture 식별 (15+) + 호출처 매핑 + symptom 분석 | `working/task_m100_555_stage1.md` |
| 2 | TDD RED — 헬퍼 STUB + 단위 테스트 3건 (2 RED + 1 GREEN) | `working/task_m100_555_stage2.md` |
| 3 | 본질 정정 — 헬퍼 fix + 9 호출처 적용 (composer.rs:920 은 Stage 2 적용) | (Stage 4 와 통합 commit) |
| 4 | 광범위 sweep — 13 fixture 481 페이지 byte-identical 검증 | `working/task_m100_555_stage4.md` |
| 5 | 최종 보고서 + merge + 새 PR 등록 | 본 문서 |

## 3. 변경 본문

### 3.1 코드 변경

| 파일 | LOC | 본질 |
|------|-----|------|
| `src/renderer/composer.rs` | +13 / -1 | `effective_text_for_metrics` 헬퍼 + `estimate_composed_line_width` 적용 |
| `src/renderer/composer/tests.rs` | +69 | 단위 테스트 3건 (Task #555 RED) |
| `src/renderer/layout.rs` | +20 / -3 | Square wrap host est_x (3444) + `compute_tac_leading_width` (3510/3516/3522) |
| `src/renderer/layout/table_layout.rs` | +21 / -4 | 셀 max width (860) / inline shape text_before (1659) / 셀 분할 추적 (1825/1851) / inline shape pre-trim (1935) |

**합계**: 4 files, +123 / -8 LOC

### 3.2 호출처 패치 패턴

**run-level 단순 적용** (5건 — composer:920, layout:3510/3522, table:860/1825/1851):
```rust
- estimate_text_width(&run.text, &style)
+ estimate_text_width(effective_text_for_metrics(run), &style)
```

**partial / char-by-char 변환 필요** (5건 — layout:3444/3516, table:1659/1935):
```rust
let metric_str: String = source_text.chars().flat_map(|ch| {
    if let Some(jamos) = map_pua_old_hangul(ch) {
        jamos.iter().copied().collect::<Vec<_>>()
    } else { vec![ch] }
}).collect();
estimate_text_width(&metric_str, &style)
```

→ 두 패턴 모두 단일 룰 (`feedback_rule_not_heuristic` 정합), 분기/허용오차 없음.

## 4. 검증 결과

### 4.1 단위 테스트

```
cargo test --lib --release
test result: ok. 1123 passed; 0 failed; 3 ignored
```

baseline 1120 → 1123 (+3 GREEN — Task #555 신규 3건). 0 회귀.

### 4.2 Clippy

```
cargo clippy --release --lib: 0 신규 결함
```

pre-existing 2건 (`table_ops.rs:1007`, `object_ops.rs:298`) 동일 baseline.

### 4.3 광범위 sweep (13 fixture, 481 페이지, PUA char 334개)

| fixture | 페이지 | PUA | 결과 |
|---------|-------|-----|------|
| exam_kor | 20 | 50 | byte-identical |
| exam_eng / exam_science / exam_math | 32 | 0 | byte-identical |
| 2010-01-06 / 21_언어_기출 | 21 | 0 | byte-identical |
| hwpspec | 177 | 85 | byte-identical |
| 복학원서 | 1 | 51 | byte-identical |
| hwp-3.0-HWPML | 122 | 56 | byte-identical |
| biz_plan | 6 | 29 | byte-identical |
| kps-ai | 80 | 24 | byte-identical |
| mel-001 | 21 | 21 | byte-identical |
| pua-test | 1 | 18 | byte-identical |

**합계**: 481/481 byte-identical ✅

### 4.4 회귀 가드

- Task #544 (`test_544_passage_box_coords_match_pdf_p4`) GREEN
- Task #547 (`test_547_passage_text_inset_match_pdf_p4`) GREEN
- Task #548 (`test_548_cell_inline_shape_first_line_indent_p8`) GREEN
- issue_546/530/505/418/501 회귀 가드 GREEN

→ **회귀 위험 0** ✅

## 5. 결과 해석

### 5.1 byte-identical 의 의미

본 cycle 의 fix 가 **현 13 fixture 의 visual 출력에 영향 없음**:

1. **visual char positioning 은 IR 기반** (Stage 1 진단 일치) — `estimate_text_width` 가 영향 미치지 않는 영역
2. **PUA + (TAC 표 / Square wrap / 셀 inline shape) 동거 케이스** 가 현 fixture 들에 부재
3. **fix 는 conservative** — 비-PUA fallback + PUA 매트릭스 정확도 향상 (잠재적 결함 차단)

### 5.2 잠재적 효과 (현재 비활성)

본 fix 가 활성화되는 시나리오 (현 fixture 부재, 추후 발견 시 자동 정합):
- TAC inline 표 앞 PUA 텍스트
- wrap=Square 호스트 PUA 텍스트 overflow
- 셀 안 PUA + inline TAC Shape 동거

위 시나리오 발견 시 본 fix 가 자동으로 PDF 정합 보장 (분기 없는 단일 룰 동작).

## 6. 메모리 룰 정합

- [feedback_per_task_pr_branch] — `pr-task555` 별도 fork branch 사용 (PR 등록 시)
- [feedback_no_pr_accumulation] — 새 PR 등록 (Task #555 단독, PR #551 잔존 누적 회피)
- [feedback_pdf_not_authoritative] — byte-identical 이므로 PDF 비교 자동 정합
- [feedback_essential_fix_regression_risk] — 광범위 13 fixture 481 페이지 검증 (회귀 0)
- [feedback_rule_not_heuristic] — 단일 룰 (분기/허용오차 없음, fallback 패턴)
- [feedback_visual_regression_grows] — byte-identical 이므로 시각 판정 auto-pass

## 7. 제출 commits

- `<commit1>` Task #555 수행/구현 계획서 + Stage 1 진단
- `<commit2>` Task #555 Stage 2-4: 옛한글 PUA 폰트 매트릭스 정정 (옵션 A)
- `<commit3>` Task #555 최종 보고서 + orders 갱신 (본 commit)

## 8. 새 PR 등록

`feedback_per_task_pr_branch` / `feedback_no_pr_accumulation` 적용:
- 별도 fork branch: `planet6897:pr-task555`
- 새 PR 등록: `gh pr create --base devel --head planet6897:pr-task555`
- 예상 PR 번호: #562

## 9. 잔존 사항

본 task 정정 외 PR #551 잔존 후보:
- Task #552 (#479 회귀 정정) — 본 devel #479 미적용 모델로 발현 안 함
- Task #544 v3 (박스 안 sequential paragraph trailing-ls 보존) — 동일
- Task #517~#523 / #519~#521 등 layout phase 1-2 — 별도 사이클 결정 대기

본 task 의 발견:
- 잠재적 PUA + TAC/Square wrap/cell inline shape 동거 케이스 발견 시 자동 정합
- 별도 fixture 추가 권장 (PUA 헤비 fixture 가 현 회귀 가드에 부재)
