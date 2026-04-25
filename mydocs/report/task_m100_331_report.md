# Task #331 최종 보고서 — 문단 trailing line_spacing 누적 drift 해결

- **이슈**: [#331](https://github.com/edwardkim/rhwp/issues/331)
- **브랜치**: `local/task331` (베이스: `task321`)
- **마일스톤**: M100
- **샘플**: `samples/21_언어_기출_편집가능본.hwp` (+ `.pdf` 비교)

---

## 1. 문제 요약

문단의 마지막 줄 뒤에 `line_spacing` 이 항상 advance 에 누적되어, col 1+ 본문이 HWP/PDF 보다 일찍 다음 페이지/단으로 넘어감. `samples/21_언어_기출_편집가능본.hwp` page 1 col 1 의 pi=26 (`2. '프로세스 마이닝'에 대해 추론한 것...`) + 보기 ①②③ 이 page 2 로 밀림.

원인: `src/renderer/typeset.rs:521-525` 에서 `height_for_fit = total_height - trailing_ls` 가 fit 검사에는 적용되었으나 advance (`current_height += fmt.total_height`) 에는 미적용.

---

## 2. 변경 사항

### 2-1. `src/renderer/typeset.rs`

| 위치 | 변경 |
|------|------|
| FullParagraph fits 경로 (line 612) | `current_height += fmt.total_height` → `+= fmt.height_for_fit` |
| line_count==0 경로 (line 622) | 동일 |
| PartialParagraph 분할 경로 (line 671~700) | 마지막 partial(`end_line >= line_count`)에서만 trailing_ls 빼기 |

### 2-2. `src/renderer/layout/paragraph_layout.rs`

본문 partial 의 마지막 visible 줄에서 trailing_ls 제외 (cell 외부에서, 셀 내 마지막 문단 보정과 통합).

### 2-3. 테스트 calibration

`src/document_core/commands/text_editing.rs` 의 5개 페이지 경계 테스트가 fix 전 (각 문단 +9.5px 도둑) 기준으로 calibrate 되어 있어 조정:

- 반복 수 증가 (50→100, 40→80 등)
- `lineSpacing` 변경 검증 테스트는 multi-line 텍스트로 변경 (단일 줄 문단은 trailing_ls 가 advance 에 무영향)

### 2-4. Golden baseline

`tests/golden_svg/issue-147/aift-page3.svg`, `tests/golden_svg/issue-157/page-1.svg` 갱신 (순수 -9.6px y-shift, 콘텐츠 변동 없음).

---

## 3. 핵심 발견

### Trailing line_spacing 의 의미

HWP `vpos_h` (LINE_SEG 기반 실측 문단 높이) 와 우리의 `fmt.total_height` 의 차이가 정확히 trailing_ls 였음. HWP 는 마지막 줄 뒤의 line_spacing 을 advance 에 포함하지 않는다. 줄간격은 본질적으로 "줄 사이" 의 간격이며, 마지막 줄 다음에는 줄이 없으므로 적용되지 않는 것이 자연스러움.

### Typeset 와 Layout 의 정합성

처음 typeset 만 수정했을 때 `LAYOUT_OVERFLOW: para=N, overflow=9.5px` 가 잔존. 원인: layout 의 y advance 가 매 줄 `lh + ls` 로 진행하여 마지막 줄에서 column bottom 을 9.5px 초과. layout 도 동일하게 마지막 visible 줄에서 ls 를 제외하여 정합성 확보.

중간 partial 의 마지막 visible 줄 ls 도 layout 에서 제외해야 함 (페이지 break 가 ls 를 흡수하므로 의미상 올바름).

### 단일 줄 문단의 line_spacing 무영향

trail_ls 보정 후, 단일 줄 문단은 line_spacing 변경에 advance 가 영향받지 않음 (lh 만 사용). 이는 HWP 와 일치하는 정확한 동작이며, line_spacing 의 의미("줄 사이 간격")에 부합.

### 페이지 수용량 증가

평균적으로 문단당 ~9.5px advance 감소 → 페이지당 더 많은 문단 수용. 21_언어 샘플 16 → 15 페이지.

---

## 4. 검증 결과

| 항목 | 결과 |
|------|------|
| 21_언어 page 1 col 1 PDF 일치 | ✅ pi=26+보기 ①②③ fit |
| LAYOUT_OVERFLOW | ✅ 0건 |
| `cargo test --lib` | ✅ 992 passed |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 2 갱신) |
| 기타 통합 테스트 | ✅ 통과 |
| 다른 샘플 회귀 | ✅ 없음 |

---

## 5. 후속 영향 / 주의사항

- 모든 HWP 문서가 평균 ~9.5px/문단 만큼 더 압축되어 페이지 수가 줄어들 수 있음 (의도된 변화)
- 향후 페이지 수 기반 단위 테스트 작성 시 trailing_ls 보정 모델을 전제로 calibrate 해야 함 (특히 단일 줄 문단의 line_spacing 변경은 advance 무영향)
- 셀 내 문단 처리는 변경되지 않음 (기존 `is_last_cell_para` 로직 유지)
- 표/footnote/zone 등 다른 advance 경로(`typeset.rs` line 1051, 1059, 1082, 1090, 1165, 1391, 1404, 1490)는 `fmt.total_height` 를 직접 사용하지 않거나 별도 보정(line 1090)이 있어 본 변경의 영향 받지 않음

---

## 6. 변경 파일 목록

- `src/renderer/typeset.rs` (3개소)
- `src/renderer/layout/paragraph_layout.rs` (1개소)
- `src/document_core/commands/text_editing.rs` (5개 테스트 calibration)
- `tests/golden_svg/issue-147/aift-page3.svg` (baseline 갱신)
- `tests/golden_svg/issue-157/page-1.svg` (baseline 갱신)
- `mydocs/plans/task_m100_331.md` (수행계획서)
- `mydocs/plans/task_m100_331_impl.md` (구현계획서)
- `mydocs/working/task_m100_331_stage{1,2,3}.md` (단계별 보고서)
- `mydocs/report/task_m100_331_report.md` (본 보고서)

## 7. 결론

이슈 #331 의 trailing line_spacing 누적 drift 가 typeset advance + layout y advance 두 경로에서 정합 보정으로 해결됨. HWP `vpos_h` 와 일치하는 정확한 페이지 수용량을 갖게 되어 21_언어 샘플 page 1 col 1 이 PDF 와 일치. 모든 자동화 테스트 통과, golden 2 개 baseline 의도된 갱신.
