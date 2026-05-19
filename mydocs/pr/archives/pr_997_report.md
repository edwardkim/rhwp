# PR #997 처리 보고서 — fix: HWP5 line_segs 누락 paragraph 겹침 해소 (composer word wrap 합성)

- 처리일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- 결정: **옵션 A (수용)** — 작업지시자 승인 + 시각 판정 통과
- 머지: (no-ff, local/devel → devel)
- closes #994

## 1. 결정 사유

@jangster77 24+ 사이클 핵심 컨트리뷰터. HWP3→HWP5 변환 시 일부 long-text paragraph(sample16 `󰏅` PUA bullet 59개)의 PARA_LINE_SEG 누락으로 `compose_lines` fallback이 단일 ComposedLine 생성 → layout이 한 y에 모든 char를 그려 시각 겹침(page 19~24). word-boundary 기반 분할 + `has_line_break=true`(Justify 비활성화)로 해소. 실제 시각 결함을 해소하고 작업지시자 시각 판정 통과, 검토 쟁점 sweep 회귀 부재 입증.

## 2. 처리 내역

- 본질 커밋 단일 `de21afa6` (작성자 Taesup Jang, 7파일 +572/-18) cherry-pick → **충돌 없음** (composer.rs auto-merge, 현 devel `3923b693`와 양립)
- 변경: `compose_lines(para)` 의 `para.line_segs.is_empty()` fallback — 단일 ComposedLine → word-boundary `CHARS_PER_LINE=35` 분할, non-last line `has_line_break=true`

## 3. 검토 쟁점 → sweep 검증 결과

검토(`pr_997_review.md`)에서 제기한 핵심 쟁점을 BEFORE(devel `3923b693`) ↔ AFTER(cherry-pick) sweep으로 검증:

| Fixture | 변화 | 쟁점 | 판정 |
|---------|------|------|------|
| `samples/hwpx/hy-001.hwpx` (HWPX 표) | 2→2, diff=0 | B (Task #671 셀 경로 충돌) | ✅ 회귀 없음 |
| `samples/hwpx/hancom-hwp/hy-001.hwp` (HWP5 변환, 표) | 2→2, diff=0 | B | ✅ 회귀 없음 |
| `samples/hwp3-sample16.hwp` (HWP3, line_segs 보유) | 64→64, diff=0 | A (고정 휴리스틱 일반화) | ✅ 무영향 (fallback 미발동) |
| `samples/hwp3-sample16-hwp5.hwp` (HWP5 변환, 타깃) | 62→67, diff=46 | — | 의도된 겹침 해소 |

- **쟁점 B (`feedback_fix_scope_check_two_paths`)**: 검토 시 Task #671 `recompose_for_cell_width` 셀 경로와의 가드 충돌(hy-001 셀 줄겹침 회귀)을 우려했으나, hy-001 HWPX/HWP5 변환본 모두 diff=0 → **회귀 없음 입증**. PR 본문이 인정한 "신규 hy-001 변동 1건"은 현 최신 devel 기준 cherry-pick에서는 발생하지 않음 (중간 devel 변화로 흡수, PR sweep 시점 base 차이).
- **쟁점 A (`feedback_hancom_compat_specific_over_general`)**: CHARS_PER_LINE=35 고정 휴리스틱의 다른 샘플 회귀 우려 → sample16-hwp3(line_segs 보유) diff=0 으로 fallback 미발동 시 무영향 확인. 임시 휴리스틱 명시(주석) + 향후 reflow_line_segs 대체 약속.

## 4. 자기 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | 1307 passed / 0 failed / 2 ignored |
| `cargo clippy --release --lib -D warnings` | 통과 |
| `cargo fmt --check` | exit 0 |
| sweep (4 fixture, BEFORE/AFTER SVG) | 쟁점 A/B 회귀 0, 타깃만 의도 변경 |

산출물: `output/poc/pr997/{before,after}/` (`project_output_folder_structure` 규약, git 미추적)

## 5. 작업지시자 시각 판정

sample16-hwp5 page 19~24 PUA bullet(`󰏅`) paragraph 줄겹침 해소 + 페이지 수 62→67 변화 — **시각 판정 통과**.

## 6. 후속 (PR 본문 정합)

- **Page count 차이 (별도 issue / 후속 #999 시리즈)**: HWP3 64 vs HWP5(post G4) 67 vs HWPX 자동보정 69 — paragraph height 누적 차이 별도 root cause. #997(겹침 시각) → #999(closes #998, 페이지 수 HWP3 정합 64) 연쇄 의존.
- HWPX 변종 영향: parser path 동일하나 본 sweep에서 hy-001.hwpx diff=0 확인 (PR scope 외였으나 무회귀 입증)
- 임시 휴리스틱(CHARS_PER_LINE=35) → 향후 reflow_line_segs 정식 호출로 대체 (PR 주석 명시 기술 부채)

## 7. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 24+ 사이클, #997~#1011 연속 시리즈 (순차 처리)
- `feedback_fix_scope_check_two_paths` — 쟁점 B 셀 경로 양립 점검 → sweep 회귀 0 입증 (권위 사례: 검토 단계 우려 → 검증 단계 sweep 으로 해소)
- `feedback_hancom_compat_specific_over_general` — 쟁점 A 고정 휴리스틱, 임시 명시 + HWP3 무영향 확인으로 완화
- `feedback_visual_judgment_authority` — sample16 겹침 해소 작업지시자 시각 판정이 최종 게이트
- `project_output_folder_structure` — sweep 산출물 output/poc/pr997 배치
- `feedback_self_verification_not_hancom` — 페이지 수 잔존(67 vs HWP3 64)은 후속 #999 분리, 본 PR은 겹침 해소 시각 본질에 한정
