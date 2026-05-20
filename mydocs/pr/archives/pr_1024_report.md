# PR #1024 처리 보고서 — Task #1022: 측정 정합 — 분할 표 cut 모델 + LAYOUT_OVERFLOW 42→12

- 처리일: 2026-05-20
- 컨트리뷰터: [@planet6897](https://github.com/planet6897) (commit 작성자 Jaeook Ryu)
- 결정: **옵션 A (수용)** — 작업지시자 승인 + 시각 판정 생략 (작업지시자 명시 "그대로 A로")
- 머지: (no-ff, local/devel → devel)
- closes #1022

## 1. 결정 사유

@planet6897 분할 표 시리즈 마무리 (#1003 + #1004 머지 후 발전형). devel 최신 기준 재구성 (MERGEABLE) + RowCut 이산 모델 단일 권위 통합 + LAYOUT_OVERFLOW 42→12 정량 측정. 광범위 표면(49 파일) 이나 CI 검증 항목 모두 본 환경에서 통과 확인.

## 2. 처리 내역 (단일 squash commit cherry-pick)

| 커밋 (cherry-pick 후) | 내용 |
|------|------|
| `bc1cd4db` | Task #1022 (#992/#993/#1022 번들 squash, 49 파일 +3960/-952, 작성자 Jaeook Ryu) |

- **충돌 없음** (devel 최신 기준 재구성으로 MERGEABLE)

## 3. 변경 본질

### A. RowCut 이산 모델 (table_layout.rs +514, table_partial.rs -423)

`cell_units` + `advance_row_cut` 단일 권위 함수 — 페이지네이터·렌더러 측정 공유. PR #1004 휴리스틱 정정을 RowCut 이산 모델로 일반화.

### B. 측정 정합 (`closes #1022`)

- cell_units ↔ HeightMeasurer 단일 권위 통합
- VPOS_CORR over-correction 제거 (`y_delta_hu` stale `+trailing_ls_hu`)
- 다중 머리행 overhead 정합

### C. v2 trailing-ls 조건부 복원 (issue_598 회귀 자정)

`lazy_base_corrected >= 0` 조건부 가드 — vpos≠0 시작 컬럼 보정 / IR 정확 추적 케이스 비보정.

### D. golden 재생성

`issue-617/exam-kor-page5.svg` + `form-002/page-0.svg` (PR commit 포함). PR 본문 명시 `issue-677/bokhakwonseo-page1.svg`는 commit에 미포함이나 본 환경 검증 결과 PR 적용 후 출력 = 현재 golden binary identical → 갱신 불필요.

### E. LAYOUT_OVERFLOW **42 → 12 (71% 감소)** 정량 측정

## 4. 자기 검증 (`feedback_push_full_test_required` 정합)

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | **1308 passed** (devel 1307 + 신규 1) |
| `cargo test --release --tests` | 전체 통합 통과 (FAILED 0) |
| `cargo test --lib pagination` | 15/0 |
| `cargo test --lib wasm_api` | 160/0 |
| `cargo clippy --release --lib -D warnings` | 통과 |
| **`cargo clippy -- -D warnings` (CI 패턴, --lib 미한정 전체)** | 통과 |
| `cargo fmt --all -- --check` (CI 패턴) | exit 0 |
| WASM 빌드 (Docker) | 4.84 MB, rhwp-studio/public 동기화 |

**CI 통과 예상** — 모든 CI 검증 항목 (fmt --all / build / test / clippy 전체) 본 환경 통과 확인.

## 5. sweep 검증 (10 fixture, BEFORE devel `65c8e693` ↔ AFTER)

| Fixture | 결과 | 판정 |
|---------|------|------|
| hy-001 HWPX/HWP5, table-vpos-01 HWPX/HWP, 복학원서 | **diff=0** | 영향 없음 |
| sample16-hwp5 | 51 same / 13 diff | 분할 표 영역 변경 |
| sample16-hwp3 | 62 same / 2 diff | 변동 |
| aift | 47 same / 27 diff | 광범위 (분할 표 보유) |
| exam_kor | 4 same / 16 diff | 광범위 |
| biz_plan | 3 same / 3 diff | 절반 |

광범위 표면(49 파일)의 영향이 sweep에서 다수 fixture 변동으로 나타남. 작업지시자 결정 "그대로 A로" — 시각 판정 생략하고 cargo test/clippy/fmt 통과 + CI 검증 항목 통과 + 정량 측정 (LAYOUT_OVERFLOW 71% 감소) 근거로 머지.

## 6. 작업지시자 결정 — 시각 판정 생략 옵션 A 수용

작업지시자 명시 "그대로 A로". 광범위 변동 + 정량 측정 효과 우선 + 회귀 가드 (issue_598/svg_snapshot/cargo test 전체) 통과로 머지 결정.

## 7. 후속

- **bokhakwonseo-page1.svg PDF 정합 효과 본 환경 미발동** — PR 본문 "band 196→214=PDF 일치" 효과가 본 환경 변동 없음. 가능 원인: 컨트리뷰터 환경 특정 페이지 / commit 누락 / v2 trailing-ls 조건 미충족. 후속 관찰 영역
- **sweep 광범위 변동 fixture (sample16/aift/exam_kor/biz_plan)** — 시각 판정 생략 머지로 CI / 후속 sweep 회귀 발견 시 hotfix 또는 후속 PR 처리
- PR #1004 후속 — 본 PR 머지로 분할 표 시리즈 (#1003 + #1004 + #1024) 마무리

## 8. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @planet6897 #1003 → #1004 → **#1024** 분할 표 시리즈 마무리
- `feedback_pr_supersede_chain` — **권위 사례**: 작은 단위(#1003) + 부분(#1004) + 발전형(#1024) 순차 적층 → RowCut 단일 권위 모델로 일반화
- `feedback_image_renderer_paths_separate` — `advance_row_cut` 페이지네이터·렌더러 단일 권위 (PR #1018 image_resolver 패턴 정합)
- `feedback_hancom_compat_specific_over_general` — v2 trailing-ls 조건부화 (case-specific 가드, 컨트리뷰터 자정)
- `feedback_visual_judgment_authority` — 시각 판정 생략 (작업지시자 결정), 정량 측정 (LAYOUT_OVERFLOW 71%) + 회귀 가드 통과로 게이트 대체
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — PR 본문 PDF 정합 명시 (복학원서 196→214) 본 환경 미발동 (후속 관찰)
- `feedback_push_full_test_required` (신규) — cargo test --tests 전체 + fmt --check 필수 정합
- `project_output_folder_structure` — sweep 산출물 output/poc/pr1024 배치
