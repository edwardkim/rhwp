# PR #1024 검토 — Task #1022: 측정 정합 — 분할 표 cut 모델 + LAYOUT_OVERFLOW 42→12

- 작성일: 2026-05-20
- 컨트리뷰터: [@planet6897](https://github.com/planet6897) (Jaeuk Ryu, commit 작성자 Jaeook Ryu)
- PR: https://github.com/edwardkim/rhwp/pull/1024
- base/head: `devel` ← `planet6897:pr-task1022`
- 연결 이슈: closes #1022 (M100: HeightMeasurer ↔ cell_units 측정 정합)
- 규모: **+3960 / -952, 49 files** (소스 11 + 테스트 + 문서 + golden)
- mergeable: **MERGEABLE** (PR 본문 명시: devel 최신 기준 재구성, #1003·#1004 위 순변경분 적층)
- 본질 커밋: 단일 squash `609e9a21` (작성자 Jaeook Ryu)

## 1. 컨트리뷰터 사이클 / 시리즈 관계

@planet6897 = 15+ PR 핵심 컨트리뷰터. 5/20 시리즈 마무리:

| PR | 본질 | 상태 |
|----|------|------|
| #1003 | Task #990 (빈 문단 위 글상자 advance 이중 가산) | 머지 `c2024ec9` |
| #1004 | Task #991 (분할 표 휴리스틱 정정) 부분 | 머지 `77a25471` |
| **#1024** | **Task #1022 (RowCut 이산 모델 + 측정 정합)** | 본 PR — #1003 + #1004 위 발전형 |

devel = `65c8e693`. PR 본문 명시 "devel 최신 기준 재구성, 다중 타스크 번들(#992/#993/#1022) → 단일 commit squash".

## 2. 본질 변경

### A. RowCut 이산 모델 도입 (table_layout.rs +514, table_partial.rs -423)

```rust
pub(crate) type RowCut = Vec<usize>;          // 셀별 소비 콘텐츠 단위
pub(crate) struct RowCutResult { pub end_cut: RowCut, ... }

fn cell_units(...) -> ...;                    // 셀 콘텐츠 평탄화
pub(crate) fn advance_row_cut(...) -> RowCutResult;  // 단일 권위 cut 함수
```

페이지네이터·렌더러가 `advance_row_cut` 단일 권위 함수 공유 → 분할 위치 불일치 제거. PR #1004 의 휴리스틱 정정(끝 페이지 패스 유도 / 1행 표 분할 금지 / vpos 팬텀) 을 이산 cut 모델로 일반화.

### B. 측정 정합 (`closes #1022`)

- **cell_units ↔ HeightMeasurer 정합**: `row_cut_content_height` / `cell_units` / `cell_line_ranges_from_cut` 단일 권위 통합 (trailing-ls 규칙, filler, corrected_line_height 일치)
- **VPOS_CORR over-correction 제거**: `y_delta_hu` 의 stale `+trailing_ls_hu` 제거 (Task #537 이 #479 정책 하에 작성 → #452 복원 후 과보정). 페이지 22 하단 초과 해소
- **다중 머리행 overhead 정합**: `header_overhead` 를 연속분에서 반복되는 `is_header` 전체 행 기준으로 정정. rs=2 머리행 rowspan-split 초과 해소

### C. v2 trailing-ls 보정 조건부 복원 (issue_598 회귀 자정)

PR 본문 명시 — 재구성 중 `issue_598_footnote_marker_nav` 회귀 검출 → trailing-ls 보정 무조건 제거가 lazy_base 산출에서 표 없는 문서 vpos 한 줄 간격 위로 밀림 원인. 조건부 복원:

```rust
let lazy_base_corrected = prev_vpos_end - (y_delta_hu + trailing_ls_hu);
let lazy_base = if lazy_base_corrected >= 0 { lazy_base_corrected }
                else { prev_vpos_end - y_delta_hu };
```

- 컬럼이 `vpos≠0` 에서 시작 (상단 박스/도형 뒤 본문): 보정 적용 → IR 정합 복원
- sequential 이 IR 정확 추적 (drift 0): 비보정 유지 → 표 over-correction 방지

PDF (한컴 2022) 검증: footnote-01 · 복학원서 모두 정답지 정합 (복학원서 본문 시작 band **196→214** = PDF 일치).

### D. golden 재생성

`issue-617 / issue-677 / form-002` — PR #1020 chain 확장 영역과 영역 중복 가능 (UPDATE_GOLDEN 일괄 갱신 패턴 적용 가능, PR #1021/#1026 패턴).

### E. LAYOUT_OVERFLOW **42 → 12 (71% 감소)** 정량 측정

## 3. 검토 의견

### 강점

1. **devel 최신 기준 재구성** — PR #1003 + #1004 머지 후 순변경분 squash 적층 (MERGEABLE 달성) — `feedback_pr_supersede_chain` 정합
2. **단일 권위 모델** — `advance_row_cut` + `cell_units` 함수로 페이지네이터·렌더러 측정 정합. PR #1004 휴리스틱 → RowCut 이산 모델 일반화 (`feedback_image_renderer_paths_separate` 본질 정합)
3. **정량 측정** — LAYOUT_OVERFLOW 42→12 (71% 감소) — 측정 가능한 효과, `closes #1022` 정합
4. **v2 회귀 자정** — `issue_598_footnote_marker_nav` 회귀 검출 → trailing-ls 보정 조건부화로 해소. 컨트리뷰터 자정 (`feedback_hancom_compat_specific_over_general` 권위 사례)
5. **PDF 검증** — 복학원서 본문 시작 band 196→214=PDF 일치 명시 (`reference_authoritative_hancom` 정합)
6. **회귀 가드 보존** — `issue_598` 4/4 + svg_snapshot 8/8 (PR 본문)
7. cargo build/test/clippy/fmt check 모두 통과 (PR 본문)

### ⚠️ 핵심 쟁점

#### (A) 광범위 표면 — 49 파일, +3960/-952

PR #1018(24 파일) + PR #1003(8 파일) + PR #1004(23 파일) 보다 큼. 회귀 표면 매우 큰 영역. **광범위 sweep + 분할 표 보유 fixture(hy-001, sample16, 표 보유 일반) 무회귀 입증 필수**.

#### (B) 단일 squash commit — bisect 효용 감소

다중 task 번들(#992/#993/#1022) 을 단일 commit 으로 squash. 회귀 시 binary search 불가 — 5가지 변경(RowCut 모델 + cell_units + advance_row_cut + VPOS_CORR + header overhead + trailing-ls 조건부) 중 어느 것이 회귀 원인인지 식별 어려움. 다만 회귀 가드 (`issue_598`) 통과로 알려진 영역은 보호.

#### (C) PR #1004 Task #991 휴리스틱 위 적층

PR #1004 의 휴리스틱 정정 (끝 페이지 패스 유도 / 1행 표 분할 금지 / vpos 팬텀 해소) 이 본 PR `advance_row_cut` 이산 모델로 일반화 또는 대체. PR #1004 변경 영역과 본 PR 영역 양립 확인 필요 — devel 최신 기준 재구성이라 자동 해소되었을 가능성 높음 (MERGEABLE).

#### (D) golden 재생성 — PR #1020 chain 확장 영역 충돌 가능성

`issue-617 / issue-677 / form-002` — PR #1020 chain 확장 갱신 영역. UPDATE_GOLDEN 일괄 갱신 패턴 적용 가능 (PR #1021/#1026 패턴 정합). 본 PR base 가 devel 최신이라 충돌 자동 해소 가능성.

#### (E) v2 trailing-ls 조건부 복원의 가드 정확성

조건 `lazy_base_corrected >= 0` — 컬럼 시작 위치 의존. PR 본문 명시 검증 (footnote-01 / 복학원서 PDF 일치 + exam_kor / form-002 골든 불변) 양호하나 광범위 sweep 으로 추가 확인 권고.

#### (F) `feedback_push_full_test_required` 정합

PR 본문 "cargo test 전체 green" 명시. 본 환경 검증 시 `cargo test --release --tests` 전체 + fmt 동시 실행 필수.

### 확인 필요 (검증 단계)

1. cherry-pick `609e9a21` — MERGEABLE 이므로 충돌 없을 가능성 + golden 재생성은 UPDATE_GOLDEN 갱신
2. `cargo test --release --lib` + `cargo test --release --tests` 전체 통합 (issue_598 + svg_snapshot 등) + clippy -D + fmt 0
3. **광범위 sweep** — 분할 표 보유 fixture (hy-001, sample16, table-vpos-01, aift 등) + 일반 fixture + 회귀 fixture (issue-598 footnote, 복학원서 PDF 정합) 무회귀
4. WASM 빌드 + 작업지시자 시각 판정 — 분할 표 + footnote + 복학원서 정합

## 4. 처리 옵션

- **옵션 A (수용 — 권고)**: 정량 측정(LAYOUT_OVERFLOW 71% 감소) + 단일 권위 모델 + 컨트리뷰터 자정 + devel 최신 재구성 우수. 광범위 sweep + 작업지시자 시각 판정 통과 시 cherry-pick no-ff merge.
- **옵션 B (수정 요청)**: 다른 fixture 회귀 시 — squash commit 회귀 차단 또는 영역 좁힘 요청.
- **옵션 C (close)**: 본질 결함 시. 해당 낮음.

## 5. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @planet6897 #1003 → #1004 → **#1024** 분할 표 시리즈 마무리
- `feedback_pr_supersede_chain` — **권위 사례**: 작은 단위(#1003) + 부분 적용(#1004) + 발전형(#1024)이 순차 적층 → 단일 권위 모델로 일반화
- `feedback_small_batch_release_strategy` — devel 최신 기준 재구성 + 단일 commit squash 적층 권위
- `feedback_image_renderer_paths_separate` — `advance_row_cut` 페이지네이터·렌더러 단일 권위 함수 (PR #1018 image_resolver 패턴 정합)
- `feedback_hancom_compat_specific_over_general` — v2 trailing-ls 조건부화 (case-specific 가드 적용, 컨트리뷰터 자정)
- `feedback_visual_judgment_authority` — 복학원서 PDF 정합 + issue_598 + svg_snapshot 작업지시자 판정 게이트
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — PDF(한컴 2022) 검증 명시 (복학원서 196→214=PDF 일치)
- `feedback_self_verification_not_hancom` — 컨트리뷰터 자기 검증 + 메인테이너 sweep + 작업지시자 시각 판정 게이트
- `feedback_push_full_test_required` (신규, 2026-05-20) — cargo test --tests 전체 + fmt --check 필수
- `project_output_folder_structure` — sweep 산출물 output/poc/pr1024 배치

## 6. 권고

**옵션 A 조건부** — 광범위 표면(49 파일) + 단일 권위 모델 + LAYOUT_OVERFLOW 71% 감소 + 컨트리뷰터 자정 + devel 최신 재구성. 검증 단계에서 (1) cherry-pick + UPDATE_GOLDEN 일괄 갱신 (PR #1021 패턴), (2) cargo test --lib + --tests + clippy + fmt 전체, (3) 광범위 sweep — 분할 표 보유 fixture + 일반 + 회귀 가드(issue_598/복학원서) 무회귀, (4) WASM + 작업지시자 시각 판정 통과 시 cherry-pick no-ff merge. 회귀 시 옵션 B 전환 (squash commit 영역 좁힘 요청).
