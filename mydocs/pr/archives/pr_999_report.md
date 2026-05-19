# PR #999 처리 보고서 — fix: HWP5 sample16 페이지 수 HWP3 reference 정합 (64 페이지)

- 처리일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- 결정: **옵션 A (수용)** — 작업지시자 승인 + 시각 판정 통과
- 머지: (no-ff, local/devel → devel)
- closes #998

## 1. 결정 사유

@jangster77 24+ 사이클. PR #997(Task #994 G4 word wrap, 직전 머지 `f44fd446`)의 직접 후속. HWP5 변환본 페이지 수(HWP3 reference 64 vs HWP5 67) 해소. 2-layer fix: composer CHARS_PER_LINE 35→45 (HWP3 line_segs 실측 평균) + typeset spacing_before=0 (line_segs-missing, HWP3→HWP5 ParaShape 2x 보정). 검토 쟁점 sweep 회귀 부재 입증 + 작업지시자 시각 판정 통과.

## 2. 처리 내역

- 본질 커밋 단일 `c850148d` (Task #998, 작성자 Taesup Jang, 7파일 +351/-3) cherry-pick → **충돌 없음** (#997 머지 `f44fd446` 위 적층, composer.rs 양립). task_994 변경(#997 중복)은 cherry-pick에서 제외.
- 변경: `composer.rs` CHARS_PER_LINE 35→45 (8줄), `typeset.rs` format_paragraph spacing_before 조건부 0 (9줄)

## 3. 검토 쟁점 → sweep 검증 결과

검토(`pr_999_review.md`)의 핵심 쟁점을 BEFORE(devel `f44fd446`, #997만) ↔ AFTER(cherry-pick) sweep 8 fixture로 검증:

| Fixture | before→after | diff | 쟁점 | 판정 |
|---------|-------------|------|------|------|
| sample16-hwp5.hwp (타깃) | 67→**64** | 48 | — | HWP3 reference 64 정확 정합 |
| sample16.hwp (HWP3) | 64→64 | 0 | B | ✅ 무영향 |
| hy-001.hwpx (HWPX 표) | 2→2 | 0 | A | ✅ 회귀 없음 |
| hancom-hwp/hy-001.hwp (HWP5 표) | 2→2 | 0 | A | ✅ 회귀 없음 |
| sample16-hwp5.hwpx (HWPX 변종) | 71→71 | 0 | — | ✅ 무영향 (PR 인정: preset linesegarray로 fix path 미통과) |
| exam_kor / exam_math / aift (일반) | 동일 | 0 | A | ✅ 회귀 없음 |

- **쟁점 A (`feedback_hancom_compat_specific_over_general`)**: typeset spacing_before=0이 공통 typeset 경로(모든 포맷/본문/각주) 무차별 적용 → 다른 line_segs 빈 paragraph 회귀 우려. line_segs 빈 paragraph 보유 가능 일반 샘플(exam/aift) 포함 8 fixture sweep 으로 **회귀 0 입증**. PR 본문 인정 "hy-001 신규 1건"은 현 최신 devel 기준 발생하지 않음 (#997과 동일 패턴 — 중간 devel 변화로 흡수).
- **쟁점 B**: sample16-hwp3(line_segs 보유) diff=0 — fallback 미발동 시 무영향.
- **쟁점 C (`feedback_self_verification_not_hancom`)**: rhwp 64 = HWP3 reference 정합. HWP5/HWPX 한컴 권위 페이지 수는 별도 권위 영역 — 후속 task 분리 (PR 본문 정합).

## 4. 자기 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | 1307 passed / 0 failed / 2 ignored |
| `cargo clippy --release --lib -D warnings` | 통과 |
| `cargo fmt --check` | exit 0 |
| sweep (8 fixture) | 쟁점 A/B 회귀 0, 타깃만 의도 변경 (67→64) |
| WASM 빌드 (Docker) | 4.8 MB, rhwp-studio/public 동기화 |

산출물: `output/poc/pr999/{before,after}/` (`project_output_folder_structure`, git 미추적)

## 5. 작업지시자 시각 판정

sample16-hwp5 페이지 수 64 (HWP3 reference 정합) + page 19/22/23 — **시각 판정 통과**.

## 6. 후속 (PR 본문 정합)

- **자동보정 (reflow_line_segs) path: 69 페이지** — line_segs 채워져 본 fix path 우회 → raw ParaShape → +5. 별도 task 분리 예정.
- **HWPX 변종: 71 페이지** — `<hp:linesegarray>` preset 으로 line_segs 채움 → 본 fix path 미통과. #942/#988 fundamental 한계 (PR #989 D6 영역).
- 임시 휴리스틱(CHARS_PER_LINE=45 + spacing_before=0 "실험") → 향후 reflow_line_segs 정식 호출 대체 (PR 주석 명시 기술 부채)
- 연속 시리즈: #999 → #1005(closes #1001) → #1009 → #1011 (@jangster77)

## 7. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 #997→#999→#1005~ 연속 시리즈 순차 처리
- `feedback_hancom_compat_specific_over_general` — 쟁점 A/B: spacing_before=0 무차별 + CHARS_PER_LINE 고정. 검토 우려 → sweep 8 fixture 회귀 0 입증으로 해소 (권위 사례)
- `feedback_self_verification_not_hancom` — 쟁점 C: HWP3 reference 정합 ≠ 한컴 권위. HWP5/HWPX 페이지 수 후속 분리
- `feedback_fix_scope_check_two_paths` — 자동보정 path 우회 (PR 인정 잔존), 본 fix는 composer fallback path 한정
- `feedback_visual_judgment_authority` — sample16 64페이지 + page 19/22/23 작업지시자 시각 판정 최종 게이트
- `project_output_folder_structure` — sweep 산출물 output/poc/pr999 배치
