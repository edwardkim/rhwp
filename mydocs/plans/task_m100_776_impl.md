---
이슈: [#776](https://github.com/edwardkim/rhwp/issues/776) Task #773 후속 정정 (H1' + H3b)
브랜치: local/task776
선행: task_m100_776.md (수행계획서, 승인됨)
작성일: 2026-05-10
---

# Task #776 구현 계획서

총 6 단계. 각 단계 완료 후 `_stage{N}.md` 보고서 작성.

---

## Stage 1 — RED test 가드 작성

### 목표

H1' + H3b 정정 후 통과해야 할 가드 작성. 정정 전 FAIL 확인 (RED).

### 작업

1. 기존 `tests/issue_773.rs` 의 `EXPECTED_OFFSET_MIN=60.0` 가드 검증 (현재 ~21 px → FAIL 예상)
2. `tests/issue_776.rs` 신규 작성:
   - shortcut.hwp 페이지 1 pi=0 (heading) y_top 가드 (PDF 26.83 ± 5 px)
   - shortcut.hwp 페이지 1 pi=2 (body) y_top 가드 (PDF 137.87 ± 5 px)
   - sungeo.hwp pi=0 heading y_top 가드
   - treatise sample.hwp pi=0 heading y_top 가드

### 산출

- `tests/issue_776.rs` (신규)
- `tests/issue_773.rs` 강화 (필요 시)
- `mydocs/working/task_m100_776_stage1.md` — RED 결과

### 완료 기준

- 신규 test 모두 RED 확인 (정정 전)
- 기존 test 회귀 없음

---

## Stage 2 — H1' 정정 + 검증

### 목표

`paragraph_layout.rs:744-748` 의 `is_column_top` 가드 정정. 셀 안 paragraph 가드 추가.

### 작업

1. `paragraph_layout.rs:744-748` 정정:
   - `is_column_top` 가드 제거 또는 `is_in_cell` 가드로 대체
2. cargo build 통과
3. issue_776 의 sungeo / treatise heading test 통과 확인 (H1' 단독 효과)
4. shortcut.hwp pi=0 위치 변화 확인

### 산출

- `src/renderer/layout/paragraph_layout.rs` 정정
- `mydocs/working/task_m100_776_stage2.md` — H1' 적용 결과

### 완료 기준

- sungeo / treatise heading test 통과
- 셀 안 paragraph 회귀 없음 (시각 검증)
- shortcut.hwp pi=2 (body) 는 부분 통과 (H3b 미적용 상태)
- cargo test 회귀 0 (또는 알려진 회귀만)

---

## Stage 3 — H3b 정정 + typeset 동기화 + 검증

### 목표

`layout.rs:1240` 영역에 zone 전환 ColumnDef.spacing / 2 가산. typeset 측 base_available_height 동기화.

### 작업

1. `layout.rs:1240` 영역 정정:
   - `is_new_zone` 분기에 `extract_columndef_spacing_px(col_content) / 2.0` 가산
   - `current_zone_start_y` 갱신
2. `extract_columndef_spacing_px` 함수 작성 (col_content → 첫 PageItem 의 paragraph → ColumnDef control → spacing)
3. `typeset.rs::process_multicolumn_break` 의 base_available_height 차감에 H3b 반영
4. cargo build 통과
5. shortcut.hwp pi=2 (body) test 통과 확인
6. shortcut.hwp 페이지 2-7 시각 검증

### 산출

- `src/renderer/layout.rs` 정정
- `src/renderer/typeset.rs` 정정 (필요 시)
- `mydocs/working/task_m100_776_stage3.md` — H3b 적용 결과

### 완료 기준

- shortcut.hwp 모든 페이지의 본문 baseline ±5 px 정합
- issue_773 + issue_776 모두 GREEN
- 페이지 over-flow 없음 (cargo test page count 회귀 없음)

---

## Stage 4 — 회귀 검증

### 목표

cargo test 전체 통과 확인 + 다단 layout 회귀 검증.

### 작업

1. cargo test 전체 실행 (≥1217 passed)
2. 다단 layout 회귀:
   - `tests/issue_768.rs` (Distribute 다단 column-break)
   - `tests/issue_770.rs` (TAC 1x1 표 후속 spacing)
   - `tests/issue_715.rs` (분할 표 orphan)
   - `tests/issue_643.rs` (페이지 분할 드리프트)
3. exam_*.hwp 다단 시각 검증
4. `cargo clippy --all-targets`

### 산출

- `mydocs/working/task_m100_776_stage4.md` — 회귀 결과
- 회귀 발생 시 정정 또는 가드 추가

### 완료 기준

- cargo test 1217+ passed
- clippy 경고 없음 (또는 무관 경고만)
- 시각 검증 통과

---

## Stage 5 — 광범위 검증 + edge case

### 목표

다양한 hwp 샘플에 대해 정합/회귀 검증.

### 작업

1. `samples/basic/*.hwp` 18개 샘플 시각 검증
2. `output/re/` 재현 검증 자동 비교
3. Edge case:
   - ColumnDef.spacing = 0
   - 셀 안 paragraph 의 sb (cell padding 중복 방지 확인)
   - 페이지 break 후 PartialParagraph
   - 큰 폰트 paragraph (BookReview)
4. PDF 정합도 측정 (pdf/basic/*.pdf)

### 산출

- `mydocs/working/task_m100_776_stage5.md` — 광범위 검증 결과
- 발견된 edge case 정정 (있다면)

### 완료 기준

- 18개 샘플 시각 회귀 없음
- 재현 검증 통과
- PDF 정합도 향상 확인 (≥3 샘플)

---

## Stage 6 — 최종 보고서 + 이슈 close

### 작업

1. 모든 단계 결과 종합
2. 정정 코드 diff 검토
3. 회귀 영향 평가
4. 최종 보고서 작성 (`mydocs/report/task_m100_776_report.md`)
5. 이슈 #773, #776 close

### 산출

- `mydocs/report/task_m100_776_report.md`
- (작업지시자 승인 후) PR 생성 — origin/pr-task776 → stream/devel

### 완료 기준

- 모든 단계 GREEN
- 회귀 0
- PR 검토 + 승인

## 작업지시자 결정 요청

본 구현계획서 승인 후 Stage 1 (RED test 가드) 진행.
