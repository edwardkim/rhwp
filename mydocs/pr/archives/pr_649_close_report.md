---
PR: #649
제목: fix: Layout 리팩터링 Phase 1 디버그 인프라 누락 회귀 정정 (Task #517 재적용, closes #647)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 13번째 사이클 PR
처리: CLOSE (PR #650 영역에서 함께 cherry-pick 영역으로 통합 처리)
처리일: 2026-05-08
---

# PR #649 Close 보고서

## 1. 결정

**옵션 A (CLOSE)** — devel 반영 부재. 본 PR 영역의 cherry-pick 영역은 후속 PR #650 머지 시 함께 처리.

## 2. Close 사유

PR #650 (Task #518 Phase 2 본질 정정) 이 이미 도착 + 본 PR `ffb32ff7` 영역의 commit 을 포함하고 있음.

```
$ gh pr view 650 --json commits
ffb32ff7 Task #517: Layout 리팩터링 Phase 1 — 디버그 인프라 + 회귀 검증 도구
e8dd3f0f Task #518: Layout 리팩터링 Phase 2 — line_break_char_idx 다중화
```

PR #650 본문 명시:
> 본 PR 은 PR #649 (Task #517) 선행 머지 필요. ... PR #649 머지 후에는 본 PR diff 가 `b395e8e6` 단일 commit 으로 자동 축약됩니다.

→ 본 PR 영역을 단독 머지 영역 대신 **PR #650 머지 시 두 commit 함께 cherry-pick** 영역으로 통합 처리. 작업지시자 결정.

## 3. 본 환경 검증 영역 (close 영역의 근거 영역 보존)

### 3.1 cherry-pick simulation
- `local/pr649-sim` 브랜치, 1 commit cherry-pick
- 충돌 0건

### 3.2 결정적 검증
- `cargo test --release --lib` → 1165 passed (회귀 0)
- `cargo clippy --release` → clean

### 3.3 env-var 동작 영역 정합 확인 (핵심)
```
$ md5sum /tmp/pr649-{default,debug}/exam_science_002.svg
842c9513bbbb833c5ba1ad27bac52694  /tmp/pr649-default/exam_science_002.svg
842c9513bbbb833c5ba1ad27bac52694  /tmp/pr649-debug/exam_science_002.svg
```

→ env-var 미설정 / 설정 영역의 SVG md5 동일 (기본 동작 무영향 확증).

## 4. PR #650 영역의 통합 처리

PR #650 cherry-pick 시:
- `ffb32ff7` (Task #517 Phase 1 인프라) 영역
- `e8dd3f0f` (Task #518 Phase 2 본질 정정) 영역

두 commit 함께 cherry-pick → 단계별 보존 머지 영역 정합. 본 PR 영역의 가치는 PR #650 영역에 그대로 보존.

## 5. 가치 보존

| 산출물 | 영역 |
|--------|------|
| Task #517 commit `ffb32ff7` | PR #650 영역에 포함 |
| `(cherry picked from commit 9c16a1b4)` 영역의 author 정합 | 보존 |
| 본 PR 검토 보고서 | `mydocs/pr/archives/pr_649_review.md` 영구 보존 |
| 본 환경 검증 자료 (md5 동일성, 1165 lib pass) | 본 보고서 영역 보존 |

## 6. Issue #647 영역

본 PR close 영역 후에도 Issue #647 은 OPEN 유지 — PR #650 머지 시 close 영역 정합. 본 close 보고서 영역 + PR #650 영역의 머지 commit 영역 정합으로 #647 close.

## 7. 컨트리뷰터 응대

@planet6897 (13번째 사이클) 안내:
- 본 PR 영역의 본질 정정 정확 (cherry-pick 영역의 단순 재적용 영역)
- 본 환경 검증 통과 (결정적 검증 + SVG md5 동일성)
- **PR #650 영역의 의존성 영역 안내 정합** — 본 PR 영역은 close 영역, PR #650 영역에서 두 commit 함께 머지 처리
- 가치 영역 보존 (작업 영역 무효화 영역 부재)

## 8. 산출물

| 산출물 | 경로 |
|--------|------|
| 검토 보고서 | `mydocs/pr/archives/pr_649_review.md` |
| Close 보고서 | `mydocs/pr/archives/pr_649_close_report.md` (본 문서) |

## 9. 메모리 룰 적용 결과

### `feedback_close_issue_verify_merged`
> 이슈 close 시 정정 commit devel 머지 검증 필수

→ 본 PR close 영역 후 Issue #647 영역의 close 영역은 PR #650 영역의 머지 영역에 정합 영역. 임의 close 영역 부재.

### `feedback_pr_comment_tone`
→ 13번째 사이클 PR 영역 컨트리뷰터 — 차분한 사실 중심 응대 영역 + supersede 영역 안내 영역.

작성: 2026-05-08
