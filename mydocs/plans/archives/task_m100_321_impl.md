# Task #321: 구현 계획서

상위: `task_m100_321.md`
브랜치: `task321` ← `task318`

## Stage 1 — 드리프트 origin 정량화

### 목표
TypesetEngine의 `fmt.total_height` 누적과 LayoutEngine의 실제 `y_offset` 진행(vpos 보정 포함) 차이를 정량화하고 origin 식별.

### 작업

1. `src/renderer/typeset.rs`에 진단 출력 훅 추가:
   - env `RHWP_TYPESET_DRIFT=1`일 때 각 문단 배치 시 `(para_idx, formatter_total_h, vpos_derived_h, diff)` 를 eprintln.
   - `vpos_derived_h` = `(last_seg.vpos + last_seg.lh + last_seg.line_spacing) - first_seg.vpos` (px 환산).
2. 21_언어 샘플 1페이지 dump 수집 → 표로 정리.
3. pi별 drift를 누적하여 col 1 합계 15px의 origin 분해.

### 산출물
- 코드: 진단 출력 훅 (추후 제거 가능한 임시 코드).
- 문서: `mydocs/working/task_m100_321_stage1.md` (표 + 해석).

### 완료 조건
drift가 어느 문단/어느 항목에서 발생하는지 수치로 설명되면 완료.

---

## Stage 2 — TypesetEngine height 산정 LINE_SEG 정렬

### 목표
`FormattedParagraph.total_height`와 `height_for_fit`를 LINE_SEG 기반 실측값으로 전환 (가능한 경우).

### 설계

```rust
struct FormattedParagraph {
    // 기존 필드 유지
    total_height: f64,           // 포맷터 계산 (fallback용)
    height_for_fit: f64,

    // 신규
    vpos_total_height: Option<f64>,   // LINE_SEG 기반 실측값
    vpos_height_for_fit: Option<f64>,
}

impl FormattedParagraph {
    fn effective_total_height(&self) -> f64 {
        self.vpos_total_height.unwrap_or(self.total_height)
    }
    fn effective_height_for_fit(&self) -> f64 {
        self.vpos_height_for_fit.unwrap_or(self.height_for_fit)
    }
}
```

- `format_paragraph()`: line_segs가 있으면 vpos 기반 값 계산하여 `Some(..)` 로 채움.
- `typeset_paragraph()`: fit 판단/누적에 `effective_*` 사용.

### 안전장치

- LINE_SEG 실측값과 포맷터 계산값의 차이가 크면(예: `|diff| > max(포맷터값 * 0.3, 30px)`) `debug_assertions` 빌드에서 경고 eprintln.
- fallback: `vpos_*_height`이 `None`이면 포맷터값 사용 (HWPX 일부 경로, 합성 Paragraph 등).

### 단위 테스트

- 합성 Paragraph에 line_segs를 수동 구성하여 `vpos_total_height` 정확도 검증.
- line_segs가 없을 때 fallback 동작 검증.

### 산출물
- 코드 수정: `src/renderer/typeset.rs`.
- 문서: `mydocs/working/task_m100_321_stage2.md`.

---

## Stage 3 — 21_언어 샘플 적중 확인

### 작업

1. `cargo build --release` + `export-svg -p 0 --debug-overlay`.
2. LAYOUT_OVERFLOW 3건 소거 확인.
3. `dump-pages -p 0,1` 비교 (Before/After).
4. 페이지 2~15 전체 훑어 눈에 띄는 회귀 유무 확인.

### 산출물
- 문서: `mydocs/working/task_m100_321_stage3.md` (Before/After 표, SVG 스크린샷 경로 기재).

---

## Stage 4 — 회귀 + 최종 보고서

### 작업

1. `cargo test --release` 및 `cargo test` (debug) 모두 실행.
2. 4개 핵심 샘플 페이지 수:
   - 21_언어, exam_math, exam_kor, exam_eng
3. 전체 `samples/*.hwp` export-svg 실행 → LAYOUT_OVERFLOW 건수 Before/After 비교 (자동 집계 스크립트 작성 또는 수동).
4. 골든 SVG 회귀 발생 시 이슈별 판단 (UPDATE_GOLDEN vs 원인조사).
5. 최종 보고서 작성 + 이슈 #321 코멘트 초안 준비.

### 산출물
- 문서: `mydocs/report/task_m100_321_report.md`.
- 오늘할일 갱신: `mydocs/orders/20260425.md`.

---

## 커밋 스킴

- Stage 1: `Task #321: Stage 1 - 드리프트 정량화 (진단 훅)`
- Stage 2: `Task #321: Stage 2 - TypesetEngine height LINE_SEG 정렬`
- Stage 3: `Task #321: Stage 3 - 21_언어 오버플로우 해소 확인`
- Stage 4: `Task #321: Stage 4 - 회귀 + 최종 보고서`

각 stage 완료 시 stage 보고서와 코드 변경을 함께 커밋.
