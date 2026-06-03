# 최종 결과 보고서 — Task #1239: 미주 인라인 수식 다행 병합 수정

- **이슈**: #1239 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1239-equation-multiline-merge` (base: `stream/devel`)
- **대상 문서**: `samples/3-11월_실전_통합_2022.hwpx` 13쪽 문20
- **작성일**: 2026-06-02

## 1. 증상

문20 풀이 정렬 블록 `S = ∫… = … = … = 17` 이 PDF(한글 2022)는 5줄 분리인데,
rhwp 는 줄2/줄3 사이가 빈 줄이 되고 두 수식이 한 줄에 수평 병합되어 **4줄로 출력**.

## 2. 근본 원인 (Stage 1)

- S= 블록은 **단일 미주 문단 pi=602**, 텍스트 char 없이 줄마다 인라인 수식(U+FFFC, treat-as-char)만 존재.
- LINE_SEG 5개·vpos 균등은 정상이나, 인라인 수식 줄 배정이 **char 위치 기준**.
- `model/paragraph.rs::control_text_positions` 가 **한 char_offsets 갭의 연속 컨트롤을 모두 같은
  position(i+1)** 으로 복원 → 사이 텍스트 char 가 없는 연속 수식(eq3·eq4)이 같은 position(2) →
  char 기반 줄 배정(`tac_on_line`)이 둘을 같은 줄로 병합, 직전 LINE_SEG 는 공백.

## 3. 수정 (Stage 2)

`src/renderer/layout/paragraph_layout.rs`:

- `equation_only_tac_line_assignment()` 추가 — **모든 줄이 수식만(빈 runs)** 이고 char_start 가
  비구분(연속 동일/감소)인 미주류 문단에서, **같은 char_start 의 줄들에 같은 position 의 연속 TAC 를
  순서대로 분배**해 `tac_idx → line_idx` 매핑 생성.
- `tac_on_line` 을 #1221 의 줄수==tac수 1:1(`index_based_tac`)에서 위 **m:n 분배 매핑**으로 일반화.
  비대상(None)이면 기존 char 기반 배정 유지.

`control_text_positions`(편집/커서/렌더 공유 핵심 함수)는 건드리지 않고 **렌더 줄 배정에서만**
보정 → 편집 위치 의미 보존.

## 4. 검증 (Stage 3)

| 항목 | 결과 |
|------|------|
| 문20 S= 블록 | 병합·빈 줄 해소 → **5줄 분리** |
| PDF(한글 2022) 13쪽 대조 | 5줄 구조 **정확 일치** (`/tmp/pdf13_1239-13.png` ↔ rhwp `output/poc/task1239/svg/…_013.svg`) |
| 페이지 전체(문20·21·23) | 무영향, 정상 |
| 골든 스냅샷(수식 카나리아) | 8 passed |
| **전체 `cargo test`** | **1933 passed, 0 failed** |

게이트(전 줄 빈 runs + char_start 비구분)가 좁아 텍스트 포함 일반 문단·#1221(셀/표 1:1)은
무영향(전체 통과로 확인).

## 5. 산출물

- 소스: `src/renderer/layout/paragraph_layout.rs`
- 시각: `output/poc/task1239/{mun20_after.png, svg/3-11월_실전_통합_2022_013.svg}`
- 문서: `plans/task_m100_1239{,_impl}.md`, `tech/endnote_inline_eq_line_1239.md`,
  `working/task_m100_1239_stage{1,2}.md`, 본 보고서.

## 6. 결론

미주 문단의 연속 인라인 수식이 한컴 LINE_SEG 경계를 무시하고 한 줄로 병합되던 문제를,
렌더 줄 배정의 m:n 분배로 해소. PDF 정합·회귀 0. **PR 진행 준비 완료.**
