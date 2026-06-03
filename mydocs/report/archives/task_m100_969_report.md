# Task #969 최종 보고서 — HWPX preset lineSegArray double-count 해소 (partial)

- 이슈: [#969](https://github.com/edwardkim/rhwp/issues/969)
- 부모: [#942](https://github.com/edwardkim/rhwp/issues/942) (증상 A 분리)
- 후속: [#988](https://github.com/edwardkim/rhwp/issues/988) (잔존 +7 페이지 drift 별도 분리)
- 브랜치: `local/task969`
- 일자: 2026-05-18

## 1. 작업 결과

`format_paragraph` 의 composed branch 에서 `Percent`/Fixed/SpaceOnly/Minimum 재계산이 발동할 때 preset `line_spacing` 을 별도 가산하던 double-count 버그 해소.

### 변경 파일
- `src/renderer/typeset.rs` (+12, -2 lines)

### 효과
- HWPX sample16-hwp5: **72 → 71 페이지** (-1)
- HWP5/HWP3 변종: 변동 없음 (62/64)
- 240 샘플 회귀: 1 건 (hwpx-02.hwpx 6→5, side effect — PDF 권위 없음)
- cargo test --release 전체 통과

## 2. Root cause (해소된 부분)

HWPX 파서가 `<hp:linesegarray>` 의 `vertsize` / `spacing` 을 IR LineSeg 로 emit (`lh=1299 HU`, `ls=779 HU` = 합 ≈ 160% of 13pt font).

composer 가 ComposedLine 으로 복사 → `format_paragraph` 의 `raw_lh(17.3) < max_fs(17.33)` 발동 → `lh = max_fs * 1.6 = 27.7 px` 재계산 (이미 160% 포함) → 그러나 `ls = 10.4 px` **별도 가산** → `38.1 px / line` (= 220% double-count).

D6 fix: 재계산 발동 시 `line_spacing = 0` 으로 처리 → `27.7 + 0 = 27.7 px` 정상.

## 3. 잔존 drift (#988)

D6 적용 후에도 +7 페이지 잔존. 측정:
- HWPX total fmt = 36919.6 px
- HWP5 total fmt = 37002.0 px
- Diff -82.4 px (**format 정합**)

→ 페이지 수 차이 원인은 **typeset_paragraph page break 결정**, **table path**, **wrap_around 상태** 등 format 영역 밖. 별도 task #988 분리.

## 4. 부모 #942 증상 영향

| 증상 | D6 효과 |
|------|--------|
| A. 페이지 수 +8 | -1 페이지 (잔존 +7, #988 으로 분리) |
| B. z-order | 변동 없음 (drift 해소되어야 정상화 기대) |
| C. 다이어그램 분리 | 변동 없음 (drift 해소되어야 정상화 기대) |

## 5. 단계별 진행

| Stage | 산출물 | 상태 |
|-------|--------|------|
| 1. 진단 정밀화 | `working/task_m100_969_stage1.md` | ✓ |
| 2. 후보 평가 + 구현 계획서 | `plans/task_m100_969_impl.md` | ✓ |
| 3. D6 패치 적용 + 검증 | `working/task_m100_969_stage3.md` | ✓ |
| 4. 회귀 검증 | `working/task_m100_969_stage4.md` | ✓ |
| 5. #942 B/C 재검증 | `working/task_m100_969_stage5.md` | ✓ |
| 6. PR + 후속 | 본 보고서 + #988 분리 | ✓ |

## 6. 결론

D6 는 **진정한 bug fix** — `ls_type=Percent` 재계산 시 line_spacing 이중계산이 본질적으로 잘못된 동작. +8 페이지 inflate 의 1/8 만 해소했지만 의미 있는 fix.

잔존 +7 페이지는 별도 [#988](https://github.com/edwardkim/rhwp/issues/988) 분리. 부모 #942 의 증상 B/C 도 #988 해소 시 자연 회복 기대.
