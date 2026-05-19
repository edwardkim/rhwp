# PR #1005 처리 보고서 — fix: HWP5/HWP3 한컴 정합 종합 fix — 격차 A/B/C/D

- 처리일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- 결정: **옵션 A (수용)** — 작업지시자 승인 + 시각 판정 통과
- 머지: (no-ff, local/devel → devel)
- closes #1001

## 1. 결정 사유

@jangster77 24+ 사이클. #997→#999 의 직접 연장(HWP3 변환본 전반 한컴 정합 종합). 격차 A/B/C 실효 + 격차 D 는 컨트리뷰터 자정 revert. 검토 쟁점(variant 오판 / HWP3 baseline 변동) sweep + 작업지시자 시각 판정으로 회귀 부재 + 정합 개선 확정.

## 2. 처리 내역 (2 본질 커밋, 모두 작성자 Taesup Jang)

| 커밋 (cherry-pick 후) | 내용 |
|------|------|
| `8b7fba3b` | Task #1001 종합 fix (격차 A/B/C/D), 14파일 |
| `4e3ad587` | variant 식별 FP 차단 강화 + **격차 D revert**, 2파일 |

- 충돌: `mydocs/orders/20260518.md` 1건 (메인테이너 PR 처리 일지 vs 컨트리뷰터 작업 일지) → `--ours` 메인테이너 일지 보존 (이전 #968 등 동일 정책). **소스 .rs 10파일 충돌 없음** (auto-merge).

## 3. 격차별 실효 + 검증

| 격차 | 내용 | 최종 실효 |
|------|------|-----------|
| **A** 페이지번호 외곽선 | HWP5 spec 표136 bit 1/2 → body_area clip | ✅ 실효 (HWP5 변환본 + HWP3 원본 외곽선 spec 정합 위치) |
| **B** PUA 글머리표 | `0xF03C5 → "□"` 매핑 | ✅ 실효 (저위험 1줄) |
| **C** 변환본 식별 + ParaShape /4 | 4중 AND 가드 + variant 시 `/4` | ✅ 실효 (sample16-hwp5만 variant=true) |
| **D** y_offset double count | line_spacing×3 휴리스틱 | ❌ **후속 `4e3ad587`에서 revert (table-vpos-01 회귀 발견) — 최종 no-op. 실효 = A/B/C** |

## 4. 자기 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | 1307 passed / 0 failed / 2 ignored |
| `cargo clippy --release --lib -D warnings` | 통과 |
| `cargo fmt --check` | exit 0 |
| WASM 빌드 (Docker) | 4.83 MB, rhwp-studio/public 동기화 |

## 5. 검토 쟁점 → sweep 검증 (8 fixture, BEFORE devel `c58c198f` ↔ AFTER)

| Fixture | 결과 | 쟁점 | 판정 |
|---------|------|------|------|
| exam_kor / exam_math / exam_eng / aift / biz_plan / 복학원서 (일반 HWP5) | 전부 **diff=0** | A | ✅ **variant=false 유지, ParaShape /4 미적용 — 오판 0** |
| sample16-hwp5 (타깃, variant=true) | diff=64 | — | 격차 A/B/C 의도 변경 |
| sample16.hwp (HWP3 원본) | diff=64 | B | **page border 외곽선만 이동, 텍스트 무변동** → 작업지시자 시각 판정 **정합 개선** 확정 |

- **쟁점 A (`feedback_hancom_compat_specific_over_general` / `feedback_v076_regression_origin`)**: variant ParaShape /4 광범위 영향 우려 → 일반 HWP5 6종 sweep diff=0 으로 variant 오판 없음 입증. 4중 AND 가드 견고성 검증.
- **쟁점 B (`feedback_visual_judgment_authority`)**: 격차 A body_area clip 이 HWP3 원본 sample16.hwp 외곽선 위치 변경 (paper 기준 y≈56/1065 → body clip y≈75/1046). 정밀 분석 — **텍스트 내용·좌표 완전 무변동, 외곽선 line 요소만 이동** (비-도형 diff 0). 검토 주석 자인("HWP3 baseline 변동 가능")이 현실화 → 작업지시자 시각 판정으로 **정합 개선(HWP5 spec 표136 정합)** 확정.
- **쟁점 C (`feedback_self_verification_not_hancom`)**: 격차 D revert로 PR 제목 "A/B/C/D" 중 D 실효 없음 — 본 보고서 명확화.

## 6. 작업지시자 시각 판정

sample16-hwp5 page 1/3 격차 A/B/C 개선 + HWP3 원본 sample16.hwp 외곽선 spec 정합 이동 + 일반 HWP5 무회귀 — **시각 판정 통과**.

## 7. 잔존 격차 (PR 본문 명시, 후속 issue 권고)

- 머릿말 inline image overlap (cover page) — selective outline push-down 필요
- 페이지 강제 나눔 정밀화 — 한컴 page-fill 휴리스틱 미상
- Shape control gradient simplify — drawing_to_shape_style 전체 callers variant context (broad refactor)
- rhwp-studio WASM 렌더링 path 진단 — Canvas2D/native SVG 차이 (`feedback_image_renderer_paths_separate`)
- **격차 D 시각 breathing room** — variant flag 를 layout 까지 thread 후 variant-only 분기 (PR #1005 scope 외, 컨트리뷰터 명시)

## 8. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 #997~#1011 연속, #1005 순차
- `feedback_hancom_compat_specific_over_general` — **권위 사례 강화**: 격차 C 4중 AND 구조 가드 + 격차 D 일반화 휴리스틱 회귀→컨트리뷰터 자정 revert
- `feedback_fix_scope_check_two_paths` — 격차 D non-variant(table-vpos-01) 경로 회귀 발견 후 revert
- `feedback_v076_regression_origin` — variant 오판 시 작업지시자 환경 회귀 위험 → 일반 HWP5 6종 sweep diff=0 입증
- `feedback_visual_judgment_authority` — **권위 사례**: HWP3 원본 baseline 변동(diff=64)을 sweep 정량 + 작업지시자 시각 판정으로 회귀 아닌 정합 개선 확정
- `feedback_self_verification_not_hancom` — 격차 D revert로 PR 제목과 실효 괴리 보고서 명확화
- `project_output_folder_structure` — sweep 산출물 output/poc/pr1005 배치
