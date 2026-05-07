# PR #620 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ SVG/web 양쪽 통과

**PR**: [#620 fix: Picture flip/rotation 누락 회귀 정정 — Task #519 정정 재적용 (closes #618)](https://github.com/edwardkim/rhwp/pull/620)
**작성자**: @planet6897 (Jaeuk Ryu) / Jaeook Ryu (commit author, jaeook.ryu@gmail.com)
**관련**: closes #618, **회귀 origin = Issue #519 (CLOSED)** — PR #527 (CLOSED) 묶음 머지 영역의 누락
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR/Issue close + 옵션 B 후속 권유 댓글 등록**
**처리일**: 2026-05-07

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`7ac87770` 단독, 4 파일 +10/-4) + devel merge + push + PR/Issue close + 옵션 B 후속 권유 |
| 시각 판정 | ★ **SVG + web 양쪽 통과** (작업지시자 시각 판정) |
| Devel merge commit | `c80d227` |
| Cherry-pick commit (local/devel) | `95b228e` |
| Cherry-pick 충돌 | 0 (auto-merge layout.rs / paragraph_layout.rs 깨끗 통과) |
| Author 보존 | ✅ Jaeook Ryu (jaeook.ryu@gmail.com) 보존 |
| PR #620 close | ✅ 한글 댓글 등록 + close |
| Issue #618 close | ✅ 수동 close (closes #618 키워드는 cherry-pick merge 로 자동 처리 안 됨, 안내 댓글 등록) |
| 옵션 B 후속 권유 | ✅ Task #517/#518/#523 누락 영역 재적용 권유 댓글 등록 |
| 광범위 페이지네이션 sweep | 167 fixture / 1,687 페이지 / 회귀 0 |

## 2. 본질 결함 (Issue #618 + 본 환경 직접 검증)

**본 환경 (edwardkim/rhwp devel) 에서 직접 검증** — PR 본문 진단 100% 정합:

```bash
$ git merge-base --is-ancestor 7ead89d devel    # ❌ Task #519 fix commit 부재
$ git merge-base --is-ancestor a7e43f9 devel    # ❌ 묶음 머지 부재
$ git branch -a --contains 7ead89d              # pr-551-review (PR 검토 임시 브랜치만)
```

**`a7e43f9` 의 출처 분석**:
- 컨트리뷰터 (Jaeook Ryu) 의 fork 에서 만든 묶음 머지 (Task #517/#518/#519/#520/#521/#523/#528)
- **PR #527 (CLOSED 상태)** 으로 본 환경에 올라온 영역 — close 처리됨 (즉, 묶음 머지가 본 환경 devel 에 도달 못 함)
- 이후 개별 task 들 중 **PR #564 (Task #521)** + **PR #592 (Task #528)** 만 cherry-pick 으로 본 환경 devel 에 적용됨
- **Task #517 / #518 / #519 / #520 / #523 은 PR #527 close 후 본 환경 미머지 상태**

증상: `samples/exam_eng.hwp` 4페이지 28번 박스 종이-말림(curl) 데코레이션 그림이 `horz_flip+vert_flip` 속성을 가지고 있으나 SVG 출력에 `<g transform>` 래퍼가 누락되어 잘못된 위치(우하단)로 렌더되어 본문과 겹쳐 보이지 않음.

해당 그림 속성:
```
[s0 p189 c1] bin_id=2 flip=(h=true,v=true) rot=0
M=[-0.976,0.000,30614; 0.000,-0.881,30190]
```

## 3. 본질 정정 (4 파일 6 ImageNode 생성 지점에 `transform:` 재적용)

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/renderer/layout.rs` | 2851 | `transform:` 추가 (TAC Picture / col_node) |
| `src/renderer/layout/picture_footnote.rs` | 117, 328 | `transform:` 추가 + import |
| `src/renderer/layout/paragraph_layout.rs` | 1834, 2108, 2218 | `transform:` 추가 + import (TAC 인라인 3 사이트) |
| `src/renderer/layout/table_cell_content.rs` | 644 | **명시적 `ShapeTransform::default()` → `extract_shape_transform(&pic.shape_attr)`** + import |

**`extract_shape_transform` 헬퍼**: `src/renderer/layout/utils.rs:109` 에 그대로 살아있음 (devel 에서 `shape_layout.rs:530` 1곳에서만 사용 중) → **신규 단위 변환 코드 0줄**.

## 4. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ issue_546 1 + issue_554 12 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,590,537 bytes** (1m 30s, PR #611 baseline 4,590,307 +230 — 4 파일 transform: extract_shape_transform 호출 추가 정합) |
| rhwp-studio `npm run build` (web 시각 판정용) | ✅ TypeScript 타입 체크 통과 + dist 빌드 (`rhwp_bg-DaB-SIXg.wasm` 4,590,537 bytes 정합) |

## 5. 정량 측정 (PR 본문 100% 재현)

### 5.1 `samples/exam_eng.hwp` SVG export 8 페이지

| 페이지 | BEFORE `<g transform=>` | AFTER `<g transform=>` | byte 차이 |
|---|---|---|---|
| page 1 | 0 | 0 | identical |
| page 2 | 0 | 0 | identical |
| page 3 | 0 | 0 | identical |
| **page 4** | **0** | **1** | **differ (2,519,257 → 2,519,364, +107)** |
| page 5 | 0 | 0 | identical |
| page 6 | 0 | 0 | identical |
| page 7 | 0 | 0 | identical |
| page 8 | 0 | 0 | identical |

### 5.2 page 4 transform 영역 (PR 본문 명시값 100% 일치)

```
<g transform="translate(1602.426666666667,0) scale(-1,1) translate(0,2217.373333333333) scale(1,-1)">
```

→ Q28 박스 종이-말림 그림 horz_flip+vert_flip 정상 적용.

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **167** (161 hwp + 6 hwpx, PR #611 의 3개 권위 샘플 포함) |
| 총 페이지 (BEFORE) | **1,687** |
| 총 페이지 (AFTER) | **1,687** |
| **fixture 별 페이지 수 차이** | **0** |

→ Picture transform 추가가 페이지네이션에 영향 없음.

## 7. 시각 판정 (★ 게이트 — SVG + web 양쪽)

작업지시자 시각 검증 결과:
- "svg 는 시각 판정 통과!"
- "웹 에디터도 시각 판정 통과입니다."

→ ★ **SVG + web 양쪽 통과**.

권위 영역: **page 4 — Q28 박스 종이-말림(curl) 데코레이션 그림** (`bin_id=2`, `flip=(h=true,v=true)`, `rot=0`) 박스 좌상단 정상 위치 출력 (PDF 정합).

## 8. PR / Issue close 처리

### 8.1 PR #620 close
- 댓글 등록 (한글, cherry-pick 결과 + 결정적 검증 + 본 환경 직접 진단 검증 + 광범위 sweep + PR 본문 100% 재현 + 시각 판정 ★ SVG+web 양쪽 통과 + 본질 평가)
- close 처리

### 8.2 Issue #618
- closes #618 키워드는 cherry-pick merge 로 자동 처리 안 됨 (PR #570/#629/#611 등 동일 패턴) → 수동 close + 안내 댓글

### 8.3 옵션 B 후속 권유 댓글 등록
- PR #620 에 별도 댓글 등록 — Task #517 / #518 / #523 누락 영역 재적용 권유
- 각 task 의 본 환경 정합 상태 표 (정합 / 누락 가능성 / PR #627 별도 영역) 명시
- 컨트리뷰터에게 후속 PR 등록 권유 + 본 환경 직접 점검도 가능 안내 (작업 부담 옵션 분리)

## 9. 회귀 방지 후속 영역 — 묶음 머지 a7e43f9 의 다른 task 누락 가능성

| Task | Issue 상태 | 본 환경 PR 영역 | 정합 상태 |
|------|----------|--------------|---------|
| **#517** | (별도 등록 영역 불명) | PR #527 (CLOSED) | ❓ **누락 가능성** (별도 후속 task 영역) |
| **#518** | (별도 등록 영역 불명) | PR #527 (CLOSED) | ❓ **누락 가능성** (별도 후속 task 영역) |
| #519 | CLOSED | PR #527 (CLOSED) → **본 PR #620 재적용 완료** | ✅ 본 PR 영역 |
| #520 | CLOSED | PR #527 + **PR #627 (revert restore)** | 🔄 PR #627 별도 영역 (다음 검토 단계) |
| #521 | CLOSED | **PR #564 cherry-pick 적용 완료** | ✅ 정합 |
| **#523** | CLOSED | PR #527 (CLOSED) | ❓ **누락 가능성** (별도 후속 task 영역) |
| #528 | CLOSED | **PR #592 cherry-pick 적용 완료** | ✅ 정합 |

→ **별도 후속 영역**: Task #517 / #518 / #523 누락 영역 + Task #520 (PR #627 의 별도 검토 영역).

## 10. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (167 fixture / 1,687 페이지) + 1140 passed 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 4 파일 6 ImageNode 생성 지점만 명시 정정, 다른 영역 무영향 (case-specific)
- ✅ `feedback_rule_not_heuristic` — `extract_shape_transform` 헬퍼 재사용, 신규 단위 변환 코드 0줄, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ SVG + web 양쪽 통과)
- ✅ `feedback_pdf_not_authoritative` — PDF 는 권위 기준 미입증이지만, 본 PR 은 IR 표준 (HWP 의 horz_flip/vert_flip 속성) 직접 검증 영역
- ✅ `feedback_per_task_pr_branch` — Task #618 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 한글 답변
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 + PR #527 close 영역 점검 후 진행
- ✅ **`feedback_close_issue_verify_merged` — 본 PR 의 회귀 origin 자체가 이 메모리 영역의 권위 케이스**: Task #519 close 시 정정 commit 의 devel 머지 검증 미수행 → 동일 결함 재발 (Issue #618). 본 PR 은 이 패턴을 정확히 진단하고 재적용. **메모리 권위 영역 강화**.
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/6 v0.7.10 후 → 5/7 처리분) 영역 정합

## 11. 본 PR 의 본질 — v0.7.10 후 네 번째 처리 PR

본 PR 의 처리 본질에서 가장 우수한 점:

1. **회귀 origin 정확 진단** — `git merge-base --is-ancestor` 로 정확한 부재 영역 입증 + PR #527 close 영역의 누락 메커니즘 정밀 분석. 본 환경 직접 재현 100% 정합.
2. **`extract_shape_transform` 헬퍼 재사용** — 신규 단위 변환 코드 0줄 (`utils.rs:109` 헬퍼 그대로 활용)
3. **케이스별 명시 가드** — 4 파일 6 ImageNode 생성 지점만 정확 정정 (`feedback_hancom_compat_specific_over_general` 정합)
4. **회귀 위험 영역 좁힘** — page 1/2/3/5/6/7/8 byte-identical, page 4 만 의도된 정정 (+107 bytes), 광범위 sweep 167 fixture / 1,687 페이지 차이 0
5. **`feedback_close_issue_verify_merged` 메모리 권위 영역 강화** — Task #519 close 후속 검증 미수행이 동일 결함 재발 (Issue #618) 의 권위 케이스로, 본 PR 의 정확한 진단 + 재적용
6. **회귀 방지 후속 권고** — PR 본문 + 옵션 B 후속 권유 댓글에 묶음 머지 `a7e43f9` 의 다른 task (#517/#518/#523) 누락 가능성 점검 권고 명시
7. **시각 판정 ★ SVG + web 양쪽 통과** — Picture transform 영역의 SVG/Canvas 양쪽 일관 정합 (`feedback_image_renderer_paths_separate` 영역에서도 정합)

## 12. 본 사이클 사후 처리

- [x] PR #620 close (cherry-pick 머지 + push + 한글 댓글)
- [x] Issue #618 close (수동 close + 안내 댓글)
- [x] 옵션 B 후속 권유 댓글 등록 (Task #517/#518/#523 누락 영역)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_620_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_620_review.md` → `mydocs/pr/archives/pr_620_review.md`)
- [ ] 5/7 orders 신규 작성 (PR #620 첫 항목)

## 13. 다음 영역

- **PR #627** (Task #520 partial revert restore) — 본 PR 처리 후 별도 검토 단계 (회귀 영역 동일 패턴, PR #527 close 영역의 다른 task 후속 PR)
- **별도 후속 task** — Task #517 / #518 / #523 누락 영역 (컨트리뷰터 후속 PR 또는 본 환경 직접 점검 영역)
