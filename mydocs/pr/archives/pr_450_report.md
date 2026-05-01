# PR #450 처리 보고서 — Task #445 지문 박스/페이지 번호 박스 시각 결함 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#450](https://github.com/edwardkim/rhwp/pull/450) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#445](https://github.com/edwardkim/rhwp/issues/445) (closes) |
| 처리 결정 | **cherry-pick 머지** (Task #445 본질 3 commits 분리) |
| 처리 일자 | 2026-04-30 |

## 처리 절차

### Stage 0: 검토 (작업지시자 결정)

PR 의 17 commits 중 Task #435 (PR #442 흡수) + Task #439 (PR #443 흡수) + merge commits 제외하고 **Task #445 본질 3 commits** 분리 cherry-pick.

작업지시자 결정 정황 — cherry-pick vs merge 비교 후 cherry-pick 채택:
- 본 사이클 일관성 (PR #442/#443 와 같은 패턴)
- 작성자가 이미 누적 commit 분리 패턴 인지
- 본 PR 의 시각 증상 정정이 본질 정정과 충돌 안 함

### Stage 1: cherry-pick

`local/pr450` 브랜치 (`local/devel` 분기) — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `452a41b` (← `c151156`) | @planet6897 | Stage 1+2: paragraph border 가 col_bottom 너머로 그려지는 문제 수정 |
| `c7979ef` (← `10d8709`) | @planet6897 | Stage 3: 머리말/꼬리말 wrap=TopAndBottom 표 anchor 위치 보정 |
| `8ff0212` (← `717ca1f`) | @planet6897 | Stage 4: 최종 결과보고서 + 오늘할일 갱신 |

cherry-pick 결과:
- Stage 1+2, 3 자동 적용
- Stage 4 의 `mydocs/orders/20260429.md` add/add 충돌 → HEAD 유지 후 `--continue`

### Stage 2: 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 21s, 4,186,281 bytes |

### Stage 3: 광범위 byte 비교

10 샘플 / 305 페이지 SVG 비교: **67 / 305 차이** (의도된 정정 영향).

차이 분포:
- 2025년 기부·답례품 30 페이지 (wrap=TopAndBottom 표 anchor)
- exam_* 28 페이지 (정정 대상 + 머리말/꼬리말)
- synam-001 5, kps-ai 2, k-water-rfp 1, aift 1 (머리말/꼬리말)

### Stage 4: 시각 판정 (작업지시자 직접 — 4 페이지 debug-overlay)

| 페이지 | overflow | 정정 효과 | 시각 판정 |
|-------|---------|----------|----------|
| 2 | +30 px (col_bottom 대비) | border 클램프 | ✅ 분석 일치 |
| 5 | +84 px | border 클램프 | ✅ 분석 일치 |
| **8** | **+248 px (+84 px 페이지 바깥)** | **가장 심각 정정** | ✅ **분석 일치** |
| 15 | +173 px (+9 px 페이지 바깥, col 1) | border 클램프 | ✅ 분석 일치 |

작업지시자 통찰 (작성자 진단 모드 변천 인식):
> "이 컨트리뷰터는 자신의 가설을 미세하게 변하는 상태를 관찰하는 모드로 진입중이라고 봅니다."

→ 본 PR 은 본질 정정이 아닌 시각 증상 클램프 + PDF 정량 측정 정합. 작성자 본인 인정.

## 변경 요약

### 본질 — `layout.rs` 두 영역 정정

| 영역 | 변경 |
|------|------|
| `build_single_column` | paragraph border merge 후 col_area 바닥/꼭대기 클램프 |
| `layout_header_footer_paragraphs` | wrap=TopAndBottom + vert=Para 표 anchor 에 line_height/2 추가 |

snapshot 갱신 1건: `tests/golden_svg/issue-267/ktx-toc-page.svg` (invisible 구조 rect height 5.34px, 가시 변화 없음).

### 잔여 본질 작업 (작성자 명시, 별도 이슈)

`respect_vpos_reset` 정책: 페이지네이션의 vpos-reset 미존중이 col_bottom 너머 layout 의 본질 origin. 작성자 후속 PR 진행 예정.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 4 페이지 직접 판정 통과 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ wrap=TopAndBottom + vert=Para + i==0 명시 |
| 작은 단위 PATCH 회전 | ✅ 시각 증상 + 잔여 본질 분리 |
| `feedback_pdf_not_authoritative` | ⚠️ 작성자 PDF 200dpi 의존 (한컴독스만) — 잔여 정정 시 한컴 2010/2020 추가 검증 권장 |
| output 폴더 가이드라인 | ✅ `output/svg/pr450-{debug,debug-baseline,regression-baseline,regression-test}/` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr450` 에서 커밋 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr450` → `local/devel` → `devel` 머지 + push
3. PR #450 close + 작성자 댓글 (이슈 #445 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_450_review.md`
- PR: [#450](https://github.com/edwardkim/rhwp/pull/450)
- 이슈: [#445](https://github.com/edwardkim/rhwp/issues/445)
- 작성자 본 사이클 머지 PR (10건): [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415), [#424](https://github.com/edwardkim/rhwp/pull/424), [#434](https://github.com/edwardkim/rhwp/pull/434), [#442](https://github.com/edwardkim/rhwp/pull/442), [#443](https://github.com/edwardkim/rhwp/pull/443), **#450 (본 PR)**
