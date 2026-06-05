# PR #1125 처리 보고 — #1122 #1124 3-11월 실전 통합 p4 수식 및 HWPX 다단 구분선 개선

## 1. 결정

**MERGE 수용** — 메인테이너 측 직접 cherry-pick + devel 머지 + PR close (base=main 영역 영역 영역 영역).

| 항목 | 값 |
|------|-----|
| 번호 | #1125 |
| 작성자 | jangster77 (Taesup Jang) — HWP3 핵심 컨트리뷰터 (16번째 PR) |
| 연결 이슈 | closes #1122 + closes #1124 (양쪽 close 완료) |
| 선행 PR (CLOSED) | #1123 (base=main, #1122 단독 영역 영역 시도) |
| 처리일 | 2026-05-26 |
| Merge commit | `9b99eadebc4a..` (devel) |
| Merge 방식 | 메인테이너 측 직접 머지 (`git merge --no-ff pr-1125-check` + 충돌 5 파일 해결 + push) |
| PR close 영역 | 수동 (base=main 영역 영역 영역 영역 devel push 영역 영역 자동 close 안 됨) |
| 이슈 #1122 #1124 close | 수동 (closes 자동 영역 영역 안 됨) |

## 2. 검증 결과

### 자동 검증 (통과)

| 항목 | 결과 |
|------|------|
| 충돌 해결 (5 파일) | ✅ orders + section.rs + layout.rs + picture_footnote.rs + utils.rs |
| `cargo build --release` | ✅ 통과 |
| `cargo fmt --all` (자동 정정) | ✅ 1 파일 정정 (equation/parser.rs — 신규 테스트 영역 영역) |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (26.36s) |
| `cargo test --release --test svg_snapshot` | ✅ 8/8 passed |
| GitHub CI | 컨트리뷰터 보고 영역 영역 통과 |

### 시각 검증

- 작업지시자 제보 영역 영역 (`samples/3-11월_실전_통합_2022.hwp` p4 + `.hwpx` p4) — 컨트리뷰터 자체 보고 영역 영역 한컴오피스 2022 정합
- 직접 시각 검증 미수행 — svg_snapshot 회귀 가드 + unit test 회귀 가드

## 3. 변경 영역 요약

16 파일, +969/-14 (4 commits):

| 영역 | 영역 |
|------|------|
| `src/parser/hwpx/section.rs` (+163/-4) | `parse_col_pr_with_children` — HWPX 자식 hp:colLine (type/width/color) 파싱 + line type/width 매핑 |
| `src/renderer/equation/tokenizer.rs` (+58/-1) | `matches_at_ascii_ci` + over/atop + 숫자 prefix-split 분기 |
| `src/renderer/equation/parser.rs` (+38) | over/atop 영역 영역 분수 토큰 처리 |
| `src/renderer/layout/utils.rs` (+47/-1) | `picture_display_size_hu()` — current vs common 영역 영역 더 큰 영역 영역 채택 |
| `src/renderer/layout/picture_footnote.rs` (+7/-5) | picture_display_size_hu 적용 |
| `src/renderer/layout.rs` (+5/-3) | use re-export |
| `mydocs/plans/` (4) + `mydocs/report/` (3) + `mydocs/working/` (2) + `mydocs/orders/20260526.md` | 컨트리뷰터 보고서 |

## 4. 충돌 해결

| 파일 | 해결 |
|------|------|
| mydocs/orders/20260526.md | 메인테이너 일지 + 컨트리뷰터 Task #1122/#1124 일지 양쪽 보존 |
| src/parser/hwpx/section.rs | 같은 위치 영역 영역 영역 영역 새 unit test (HEAD: 8개, PR: 2개) — 양쪽 모두 추가 |
| src/renderer/layout.rs | HEAD 영역 영역 정리된 use re-export + picture_display_size_hu 영역 영역 영역 영역 |
| src/renderer/layout/picture_footnote.rs | HEAD 영역 영역 정리된 use + picture_display_size_hu import |
| src/renderer/layout/utils.rs | HEAD 영역 영역 자동 영역 영역 영역 영역 (Picture import 이미 적용됨) |

## 5. base=main 영역 영역

본 PR 영역 영역 base=main 영역 영역 설정 — 본 프로젝트 영역 영역 정상 워크플로 영역 영역 base=devel. 선행 #1123 영역 영역 동일 base 영역 영역 close 후 #1125 영역 영역 통합 영역 영역 시도 (base 영역 영역 그대로). 메인테이너 측 영역 영역 직접 머지 (PR #1120 영역 영역 영역 영역 패턴) + PR 자체 close.

## 6. 위험·관찰

| 항목 | 등급 | 영역 |
|------|------|------|
| base=main 영역 영역 PR 워크플로 영역 영역 잘못 설정 | 중 | 컨트리뷰터 영역 영역 영역 영역 신규 룰 권장 — `feedback_pr_base_devel_required.md` 영역 영역 |
| 이슈 #1122 #1124 영역 영역 assignee 비어있음 | 저 | 메모리 룰 `feedback_assign_issue_before_work` 위반 (#1120 영역 영역 동일 패턴) |
| 별도 통합 테스트 파일 없음 | 저 | unit test (section.rs + tokenizer.rs + utils.rs) 영역 영역 회귀 가드 |
| 직접 시각 검증 미수행 | 중 | 작업지시자 fixture 시각 검증 권장 (3-11월_실전_통합_2022) |
| `picture_display_size_hu` 영역 영역 영역 영역 — 더 큰 축 채택 | 저 | 합리적 가설 + unit test 영역 영역 일부 확보 |
| over/atop + 숫자 분기 | 저 | case-specific — alpha/sqrt 영역 영역 영역 영역 영역 회귀 0 |

## 7. 후속 권장 영역

| 항목 | 우선순위 | 영역 |
|------|---------|------|
| 작업지시자 3-11월_실전_통합_2022 fixture 시각 검증 | 즉시 | p4 수식 + 다단 구분선 + 문26 그림 |
| `feedback_pr_base_devel_required.md` 메모리 룰 추가 | 본 PATCH 후속 | 컨트리뷰터 영역 영역 영역 영역 base=devel 영역 영역 |
| 이슈 assignee 정책 강화 | M100 후속 | `feedback_assign_issue_before_work` 영역 영역 영역 영역 |

## 8. 메모리 룰 정합

- ✅ `feedback_hancom_compat_specific_over_general` — over/atop 영역 영역 case-specific, picture_display_size_hu 영역 영역 conservative
- ✅ `feedback_pr_supersede_chain` — #1123 → #1125 supersede (close+통합)
- ✅ `feedback_visual_judgment_authority` — 컨트리뷰터 자체 시각 검증 + 회귀 가드
- ⚠️ `feedback_assign_issue_before_work` — 이슈 #1122/#1124 assignee 비어있음
- ✅ `feedback_v076_regression_origin` — fmt + clippy + svg_snapshot 영역 영역 회귀 검출
