# PR #461 검토 — Task #459 + #462 + #463 + #468 + #469 (5 Tasks 통합)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#461](https://github.com/edwardkim/rhwp/pull/461) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 본 사이클 16번째 PR |
| 이슈 | [#459](https://github.com/edwardkim/rhwp/issues/459) (closes, 본 PR 메인) + Task #462/#463/#468/#469 |
| base / head | `devel` ← `planet6897:local/devel` |
| 변경 규모 | +4,538 / -656, 57 files (53 commits → 본질 15 commits) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-30 |

## 본 PR 의 특수 정황 — 5 Tasks 누적

PR 제목은 Task #459 만 명시되었으나 실제로는 **5 Tasks 누적**:

작성자가 PR #454 / #457 미머지 상태에서 자기 fork 의 `local/devel` 위에 후속 Tasks 누적 진행한 정황. 작업지시자 결정 (옵션 A) 으로 **5 Tasks 모두 분리 cherry-pick** 진행.

## 5 Tasks 본질 commits (15 commits)

### Task #459 — 다단 후속 페이지 LINE_SEG vpos-reset 단 경계 (1 commit)

`829cb0b` — `on_first_multicolumn_page` 가드 제거. 다단 구역이 페이지를 넘어갈 때 후속 페이지에서도 LINE_SEG vpos-reset 으로 인코딩된 단 경계 인식.

**효과**: exam_kor 페이지 2 좌측 단 pi=39 4줄 중 마지막 2줄이 col_bottom 39px 초과 → HWP LINE_SEG vpos 리셋 위치 (ls[2]) 대로 좌측 2줄 / 우측 5줄 정확 분할. **PR #450 의 잔여 본질 작업** (`respect_vpos_reset` 정책) 정정.

### Task #462 — TAC Picture 인라인 line advance 누락 (1 commit)

`427db1f` — TAC Picture 다음 줄 line advance 누락 정정.

### Task #463 — exam_kor 14p 본문 외곽선 셀 leakage (8 commits Stage 1~8 + snapshot)

```
2a074be — Stage 1: 수행/구현 계획서
6a6bbf8 — Stage 2: 셀 단락 본문 외곽선 큐 leakage 게이팅
b82e422 — Stage 3: 시각 검증 + 회귀 보고서
7c46be0 — Stage 4: 최종 결과 보고서
58270f8 — Stage 5: 박스 geometry max-extent + wrap host override + floating 표 둘러싸기
0e81119 — Stage 6: wrap host 텍스트 들여쓰기 이중 적용 수정
eedc395 — Stage 7: 인라인 TAC 그림 crop 누락 수정
c500a07 — Stage 8: 확장 바탕쪽 헤더 중복 렌더링 수정
a5d3f7a — 보고서 갱신 (Stage 5~8 추가)
48fcf13 — snapshot golden 갱신 (Stage 2 cell_ctx 게이팅 부작용)
```

5 영역 (셀 leakage / 박스 geometry / 들여쓰기 / TAC crop / 바탕쪽 중복) 의 누적 정정. Stage 8 까지의 깊은 정정이라 **메인테이너 시각 검증 핵심 영역**.

### Task #468 — cross-column 박스 partial 플래그 (2 commits)

`154b4e8` + `5598fc1` — cross-column 박스 연속 partial 플래그 보정.

### Task #469 — cross-column partial 박스 col_top/col_bot 침범 (1 commit)

`ca31fb7` — cross-column partial 박스 inset 으로 인한 col_top/col_bot 경계 침범 수정 (Task #468 후속).

## 처리 방향

**옵션 A — 5 Tasks 모두 분리 cherry-pick** (작업지시자 결정).

본 사이클 일관 패턴 + 본 PR 의 누적 정황 인지 + 작은 단위 회전 정책 부합.

## dry-run cherry-pick 결과

`local/pr461` 브랜치 (`local/devel` 분기 — PR #454 + #457 머지 후) 에서 시간 순 15 commits cherry-pick — 작성자 attribution 보존:

| 영역 | commits | 충돌 |
|------|---------|------|
| Task #459 | `72d8840` (← `829cb0b`) | 없음 |
| Task #462 | `ca5b084` (← `427db1f`) | 없음 |
| Task #463 (10 commits) | `7cf8dd7`~`526c104` | 없음 |
| Task #468 (2 commits) | `313eb8b`, `3f76680` (← `154b4e8`, `5598fc1`) | `mydocs/orders/20260430.md` add/add — HEAD 유지 |
| Task #469 | `458b051` (← `ca31fb7`) | `mydocs/orders/20260430.md` add/add — HEAD 유지 |

## 검증 게이트 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1070 passed** (1069 → +1, 신규 테스트 추가) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed (golden 4건 갱신: form-002, issue-157, issue-267, table-text) |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

### golden snapshot 갱신 4건

본 5 Tasks 의 광범위 영향 (특히 Task #463 Stage 2 cell_ctx 게이팅 부작용):
- `tests/golden_svg/form-002/page-0.svg`
- `tests/golden_svg/issue-157/page-1.svg`
- `tests/golden_svg/issue-267/ktx-toc-page.svg`
- `tests/golden_svg/table-text/page-0.svg`

## 광범위 byte 단위 비교

10 샘플 / 305 페이지 SVG 비교 (PR #457 머지 후 devel ↔ PR #461):

| 결과 | 카운트 |
|------|------|
| byte 동일 | **42 / 305 (13.8%)** |
| 차이 발생 | **263 / 305 (86.2%)** |

차이 분포 (모든 샘플에 광범위 영향):

| 샘플 | 차이 페이지 |
|------|----------|
| kps-ai | 66 |
| aift | 61 |
| exam_* | 45 |
| synam-001 | 33 |
| 2025년 기부·답례품 | 30 |
| k-water-rfp | 23 |
| biz_plan | 5 |

→ Task #463 의 본문 외곽선 / 박스 geometry / wrap host 들여쓰기 / 바탕쪽 정정이 매우 광범위 영향 (PR #454 87% 와 비슷).

## 시각 판정 정황 (작업지시자 결정)

작업지시자 결정 (PR #454 / #457 와 동일 정책):
> "메인테이너는 이 PR 처리를 끝 낸 후 시각적 검증을 하겠습니다."

본 PR 단독 시각 판정 보류 — **PR #454 + #457 + #461 모든 머지 후 통합 시각 검증** 진행.

이는 누적 정정 (paragraph_layout 통일 + 글상자 외부 본문 + 다단 vpos-reset + TAC Picture line advance + 셀 leakage + 박스 geometry + 들여쓰기 + 바탕쪽 + cross-column) 모두 적용된 상태에서 한 번에 한컴 정답지 비교 진행 — 작업 효율 + 위험 분산.

## 본 PR 의 좋은 점

1. **5 Tasks 누적 본질 정정**: Task #459 (PR #450 의 잔여 본질) + Task #463 (Stage 8 까지 깊은 누적 정정)
2. **PR #450 의 잔여 작업 직접 처리**: `respect_vpos_reset` 정책 본질 정정 (`on_first_multicolumn_page` 가드 제거) — 작성자가 PR #450 본문에 명시한 잔여 본질을 즉시 처리
3. **광범위 회귀 검증** (작성자 본문 명시): exam_kor LAYOUT_OVERFLOW 36→16 (-20 건 해소), 다른 샘플 SVG byte 동일

## 본 PR 의 위험 정황

- **광범위 영향 (86% 페이지)**: 작성자 명시 외 모든 샘플에 변화. 통합 시각 검증에서 회귀 점검 필수
- **Task #463 의 깊은 누적**: Stage 8 까지의 정정 — 본질 영역 5개 (셀 leakage / 박스 geometry / 들여쓰기 / TAC crop / 바탕쪽) 가 한 Task 에 누적
- **PR 제목 ↔ 실제 누적 불일치**: 제목 (#459) 만 명시, 본문도 #459 만 다룸 — 다른 4 Tasks 검증 자료 메인테이너 통합 검증 의존

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1070 + svg_snapshot 6/6 + clippy 0 |
| 시각 판정 게이트 (push 전 필수) | ⏸️ 후속 통합 검증 (작업지시자 결정) |
| `feedback_v076_regression_origin` | ⚠️ 광범위 변화 — 통합 시각 검증 필수 |
| 작은 단위 PATCH 회전 | ✅ 5 Tasks 분리 cherry-pick (PR 자체는 누적이지만 메인테이너 분리 처리) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr461` 에서 커밋 |

## 작성자 후속 PR 안내 권장

본 사이클의 누적 정황 인식 — 향후 PR 작성 시:
1. **자기 fork base 정기 rebase**: origin/devel (cherry-pick 후) 로 rebase 하면 다음 PR 의 commits 누적 부담 감소
2. **PR 제목 ↔ 실제 commits 정합**: 5 Tasks 누적 시 PR 본문에 모든 Tasks 명시 권장
3. **Task #463 같은 깊은 누적**: Stage 별 분리 PR 권장 — 8 stages 가 한 Task 에 묶이면 검증 자료 부담 큼

## 다음 단계

1. 본 보고서 commit
2. `local/pr461` → `local/devel` → `devel` 머지 + push
3. PR #461 close + 작성자 댓글 (5 Tasks 모두 close, 후속 PR 권장 사항 안내)
4. **PR #454 + #457 + #461 통합 시각 검증** 진행 (작업지시자 직접)

## 참고

- PR: [#461](https://github.com/edwardkim/rhwp/pull/461)
- 이슈: [#459](https://github.com/edwardkim/rhwp/issues/459) + Task #462/#463/#468/#469 본질 정정
- 직전 PR (같은 영역 누적): [#454](https://github.com/edwardkim/rhwp/pull/454) (Task #452), [#457](https://github.com/edwardkim/rhwp/pull/457) (Task #455)
- PR #450 의 잔여 본질 정정: Task #459 (`respect_vpos_reset` 정책)
- 통합 시각 검증: PR #454 + #457 + #461 모두 머지 후 진행
