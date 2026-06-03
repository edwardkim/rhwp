# Task #555 구현 계획서 — 옛한글 PUA → 자모 변환 후 폰트 매트릭스 갱신

**수행 계획서**: `mydocs/plans/task_m100_555.md` (승인 완료 2026-05-04)
**브랜치**: `pr-task555` (devel `f807378a` 분기)
**옵션**: A — `display_text` 우선 사용
**작성일**: 2026-05-04

## 단계 분할 (5 단계)

### Stage 1 — 진단 (TDD RED 데이터 수집)

**목적**: 옵션 A 적용 전 광범위 fixture 의 symptom 측정 + 정합 기준 확립.

**작업**:

1. **PUA 영향 fixture 식별**:
   ```bash
   for f in samples/*.hwp; do
     count=$(./target/release/rhwp dump "$f" 2>/dev/null | grep -oE '[\xee-\xef][\x80-\xbf][\x80-\xbf]' | wc -l)
     [ "$count" -gt 0 ] && echo "$f: $count PUA chars"
   done
   ```
   → 영향 fixture 목록 + paragraph 좌표 확보

2. **exam_kor 핵심 paragraph 측정** (수행계획서 대상):
   - 문단 2.5 (149 cc, 4 LINE_SEG, PUA 책괄호 + 옛한글 합자)
   - 문단 2.19 (49 cc, PUA 옛한글 합자)
   - 페이지 17 SVG → IR 의 ts/cs/sw vs SVG 의 텍스트 위치 정합 측정

3. **PUA char vs 자모 시퀀스 폭 차이 정량 측정**:
   - 빈도 높은 PUA char (책괄호 U+F861/F862, 옛한글 합자 등) 의 PDF 한컴 2010 글자폭 측정
   - `estimate_text_width` 가 산출하는 PUA 1-char 폭 vs `display_text` (자모 시퀀스) 폭 비교
   - 차이 값 통계 (평균/최대/분포)

4. **가시 symptom 정량화**:
   - **Square wrap 영역**: PUA 포함 paragraph 가 wrap host 인 경우, 우측 overflow 측정
   - **Line break 위치**: line_seg 의 ts (char_offset) 가 PDF 의 실제 line break 위치와 정합 여부
   - **TAC inline shape**: PUA 텍스트 + inline shape 동거 paragraph 에서 shape 좌표 시프트
   - **줄간격 (lh)**: `composer.rs::estimate_line_seg_width` 가 lh 산출에 사용 → PUA char height 가정 vs 자모 합자 실제 height

5. **PDF 한컴 2010 비교**:
   - exam_kor 페이지 17 의 PDF 200 dpi 글자 좌표 측정
   - rhwp SVG 와 비교 → ±X px 시프트 위치 마커링

**산출물**:
- `mydocs/working/task_m100_555_stage1.md` (진단 보고서)
- 측정 데이터: 영향 fixture 목록, 글자폭 차이 분포, gay 위치 시프트 통계
- Stage 2 의 TDD RED 테스트 케이스 후보 (3-5건)

**완료 조건**: 작업지시자 승인 후 Stage 2 진행. 진단 결과로 옵션 A 외 옵션 (B/C) 검토 가능.

---

### Stage 2 — TDD RED 테스트 추가

**목적**: 옵션 A 정정 전 검증 가능한 회귀 가드 + symptom 검출 테스트 추가.

**작업**:

1. **단위 테스트** (`text_measurement.rs::tests`):
   - `test_estimate_width_pua_oldhangul_uses_jamo_seq`: PUA char 1글자 입력 시 자모 시퀀스 폭으로 산출 (현 코드 RED, 옵션 A 적용 시 GREEN)
   - 다만 현 `estimate_text_width(text, style)` 시그니처는 text 만 받음 → 단위 테스트는 caller 수준에서 해야 함.

2. **통합 테스트** (`integration_tests.rs`):
   - `test_555_pua_paragraph_line_break_p17`: exam_kor p17 paragraph 의 line break 위치 검증 (PDF 정합)
   - `test_555_pua_square_wrap_host_width`: PUA 포함 wrap=Square host 의 우측 경계 검증 (없으면 skip)
   - `test_555_pua_tac_inline_shape_offset`: PUA + inline shape paragraph 의 shape 좌표 검증 (없으면 skip)

3. **TDD RED 확인**:
   - cargo test --lib `test_555_*` → 모두 RED (또는 #[ignore] 마크) 상태
   - 옵션 A 적용 시 GREEN 전환 확인

**산출물**:
- `src/renderer/layout/integration_tests.rs` (+N LOC, test_555_* 추가)
- `mydocs/working/task_m100_555_stage2.md` (TDD RED 측정값)

**완료 조건**: 작업지시자 승인 후 Stage 3 진행.

---

### Stage 3 — 본질 정정 (옵션 A 적용)

**목적**: `estimate_text_width` 호출처 ~10건을 `display_text` 우선 사용 패턴으로 정정.

**작업**:

1. **헬퍼 함수 추가** (`composer.rs` 또는 별도 위치):
   ```rust
   /// PUA 옛한글 변환 후 폰트 매트릭스 측정용 effective text 반환.
   /// `display_text` 가 있으면 자모 시퀀스 사용, 없으면 `text` (비-PUA) 사용.
   pub fn effective_text_for_metrics(run: &TextRun) -> &str {
       run.display_text.as_deref().unwrap_or(&run.text)
   }
   ```

2. **호출처 패치** (10건 추정):
   - `composer.rs:920` — `estimate_text_width(effective_text_for_metrics(run), &ts)`
   - `layout.rs:3444` — char-by-char 측정 (PUA char 단독 시 매핑 적용)
   - `layout.rs:3510/3516/3522` — run-level 측정 (effective_text 사용)
   - `table_layout.rs:860/1657/1814/1840/1922` — run-level 측정

3. **단일 룰 보장** (`feedback_rule_not_heuristic`):
   - 모든 호출처가 동일 헬퍼 사용 (분기/허용오차 없음)
   - 비-PUA 텍스트는 fallback 으로 동일 동작 (`unwrap_or(&run.text)`)

4. **TDD RED → GREEN**:
   - Stage 2 의 test_555_* 모두 GREEN 전환 확인
   - 1121 baseline + 신규 +N GREEN

**산출물**:
- `src/renderer/composer.rs` (헬퍼 +5 LOC)
- `src/renderer/layout.rs` (호출처 패치 +/-N LOC)
- `src/renderer/layout/table_layout.rs` (호출처 패치 +/-N LOC)
- `mydocs/working/task_m100_555_stage3.md` (정정 디테일)

**완료 조건**: cargo test --lib --release / clippy 통과 + 작업지시자 승인 후 Stage 4 진행.

---

### Stage 4 — 광범위 회귀 검증

**목적**: 비-PUA 영역 byte-identical 보장 + PUA 영역 PDF 정합 확인.

**작업**:

1. **6 샘플 SVG sweep**:
   - exam_kor / exam_eng / exam_science / exam_math / 2010-01-06 / 21_언어_기출
   - before (devel `f807378a`) ↔ after (Stage 3 후) 비교
   - 차이 분류: PUA 영역 (의도) / 비-PUA 영역 (회귀 검출 지점)

2. **PUA fixture 추가 비교**:
   - Stage 1 의 영향 fixture 모두 검증
   - PDF 한컴 2010 좌표 정합 측정 (글자 위치 / 줄바꿈 / TAC inline shape)

3. **회귀 가드**:
   - issue_546/530/505/418/501 회귀 0
   - svg_snapshot 6/6
   - test_544/547/548 GREEN 유지

**산출물**:
- `mydocs/working/task_m100_555_stage4.md` (광범위 sweep 결과 + 회귀 분석)
- `/tmp/diag555/before` ↔ `/tmp/diag555/after` 비교 데이터

**완료 조건**: 작업지시자 시각 판정 통과 후 Stage 5 진행.

---

### Stage 5 — 최종 보고서 + merge + PR 등록

**목적**: 작업 완료 절차 + 새 PR 등록 (`feedback_per_task_pr_branch` 적용).

**작업**:

1. **최종 보고서**: `mydocs/report/task_m100_555_report.md`
2. **orders 갱신**: `mydocs/orders/20260504.md` 또는 다음 날짜 (작업 완료일)
3. **archives 이동**: 검토/보고서 → `mydocs/pr/archives/` 또는 plans/working archives
4. **branch 정리**:
   - `git push -u origin pr-task555`
   - `gh pr create --base devel --head planet6897:pr-task555`
5. **이슈 #555 close** (with cherry-pick reference)

**산출물**:
- `mydocs/report/task_m100_555_report.md`
- 새 PR (예상 PR #562)

**완료 조건**: 작업지시자 시각 판정 통과 + push + PR 등록.

---

## 계획 외 사항

### 옵션 변경 가능성

Stage 1 진단 결과:
- 가시 symptom 없음 (모두 IR 정합) → 옵션 A 라도 byte-identical 가능 (이상적)
- 가시 symptom 광범위 → 옵션 A 적용 + Stage 4 회귀 검증 강화
- IR 자체가 자모 기준 → 옵션 B (인덱싱 변경) 필요할 가능성 (현재로선 낮음)

→ Stage 1 결과 후 옵션 재확정 + 작업지시자 승인 게이트.

### 기존 회귀 가드

- `test_544_passage_box_coords_match_pdf_p4`
- `test_547_passage_text_inset_match_pdf_p4`
- `test_548_cell_inline_shape_first_line_indent_p8`
- `issue_546/530/505/418/501`

본 task 정정은 PUA 텍스트 영역만 영향. 비-PUA 영역 (위 회귀 가드들) 무회귀 보장.

### 시간 추정

- Stage 1: 진단 (1-2 hr — fixture sweep + 측정 + PDF 비교)
- Stage 2: TDD RED (30 min — 테스트 3-5 건)
- Stage 3: 본질 정정 (1 hr — 헬퍼 + 호출처 패치)
- Stage 4: 회귀 검증 (1 hr — sweep + 시각 판정)
- Stage 5: 보고서 + PR (30 min)

총 추정: 4-5 hr (작업지시자 승인 게이트 제외).

## 작업지시자 결정 사항

1. **구현 계획 승인** — 5 단계 분할 / 옵션 A 적용 절차 / 시간 추정
2. 승인 시 Stage 1 (진단) 진행
3. 옵션 변경 의향 (Stage 1 후 재결정 가능)
