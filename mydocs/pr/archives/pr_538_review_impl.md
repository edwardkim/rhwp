# PR #538 cherry-pick 구현 계획서

**대상 PR**: [#538 fix: 21_언어_기출_편집가능본.hwp 줄간격 716 HU drift 정정 (Task #534 v2 + #537 + #539)](https://github.com/edwardkim/rhwp/pull/538)
**작성자**: @planet6897 (Jaeuk Ryu)
**처리 결정**: ✅ 본질 12 commits 만 cherry-pick (작업지시자 의도)
**작성일**: 2026-05-03

## 1. cherry-pick 전략

### 1.1 대상 commits (시간 순서)

| 순서 | commit | task | 변경 영역 |
|------|--------|------|---------|
| 1 | `fbcb5c5` | #534 계획서 | mydocs/plans/task_m100_534.md |
| 2 | `b669ab5` | #534 Stage 1 진단 | mydocs/working/task_m100_534_stage1.md |
| 3 | `4abee04` | #534 Stage 3 정정 | **`src/renderer/layout.rs` (+25 / -2)** |
| 4 | `5357223` | #534 Stage 3-5 보고서 | mydocs |
| 5 | `9dfc56a` | #534 v2 정정 | **`src/renderer/layout.rs` (+12 / -1)** |
| 6 | `47d1aac` | #534 v2 보고서 | mydocs |
| 7 | `226b644` | #537 Stage 1 + TDD | mydocs + `tests/integration_tests.rs` |
| 8 | `1803bc6` | #537 Stage 2 정정 | **`src/renderer/layout.rs` (+14 / -1)** |
| 9 | `a39085a` | #537 Stage 3 보고서 | mydocs |
| 10 | `e8a0a8c` | #539 Stage 1 + TDD | mydocs + `tests/integration_tests.rs` |
| 11 | `0db709b` | #539 Stage 2 정정 | **`src/renderer/layout.rs` (+9 / -0)** |
| 12 | `eb0ddc2` | #539 Stage 3 보고서 | mydocs |

**제외 commits**: PR #538 의 나머지 170 commits (40 여 task) — 별도 PR 분리 또는 컨트리뷰터의 다음 PR 시 포함.

### 1.2 충돌 처리 정책

| 파일 | 충돌 가능성 | 정책 |
|------|----------|------|
| `mydocs/orders/20260502.md` | 🟧 중간 | auto-merge 정합 — PR #507/#531 사례 처럼 처리 |
| `mydocs/orders/20260503.md` | 🟥 거의 확실 | **수동 해소 필수** — 본 환경의 오늘 작업 (PR #507 + PR #531 + 이슈 #543) 보존 + 컨트리뷰터의 #537/#539 일지 통합 |
| `src/renderer/layout.rs` | 🟢 매우 작음 | 다른 영역 변경 — 정합 |
| `tests/integration_tests.rs` | 🟧 중간 | 같은 파일 끝에 새 테스트 추가 — 정합 |

## 2. cherry-pick 절차

### 2.1 사전 점검

```bash
# main 동기화 (메모리 feedback_release_sync_check)
git fetch origin
git pull --ff-only origin main 2>&1 || echo "main divergence — manual check"

# local/devel 스위치 + 검토 문서 stash
git checkout local/devel
git stash push -u -m "PR #538 review docs" \
    mydocs/pr/pr_538_review.md \
    mydocs/pr/pr_538_review_impl.md
```

### 2.2 cherry-pick 12 commits

```bash
# Task #534 v1 (4 commits)
git cherry-pick fbcb5c5 b669ab5 4abee04 5357223

# Task #534 v2 (2 commits)
git cherry-pick 9dfc56a 47d1aac

# Task #537 (3 commits)
git cherry-pick 226b644 1803bc6 a39085a

# Task #539 (3 commits)
git cherry-pick e8a0a8c 0db709b eb0ddc2

# 충돌 발생 시 수동 해소 (--continue / --abort)
```

### 2.3 stash 복원 + 결정적 검증

```bash
git stash pop

cargo test --release --lib                    # 1119 passed 기준
cargo test --test issue_530                   # 1 passed (PR #531 회귀 0)
cargo test --test issue_505                   # 9/9 (PR #507 회귀 0)
cargo test --test issue_418                   # 회귀 0
cargo test --test issue_501                   # 회귀 0
cargo test --test svg_snapshot                # 6/6
cargo clippy --lib -- -D warnings             # 0 건
cargo build --release                         # 정상
```

### 2.4 WASM 빌드 + studio 동기화

```bash
docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp_bg.wasm rhwp-studio/public/rhwp_bg.wasm
cp pkg/rhwp.js     rhwp-studio/public/rhwp.js
```

### 2.5 ※ 작업지시자 시각 판정

**1차 (SVG, 이미 출력됨)**: `output/svg/pr538/21_언어_기출_편집가능본_001~015.svg`

판정 항목:
- Task #537: P2 q3 / P3 q6 / P5 q9 / P6 q12 / P8 q15 / P9 q17/18 / P12 q23/24 / P13 q27 / P14 q29 (11곳) — TAC 표 직후 첫 답안 줄간격
- Task #539: P7 (pi=145→146) / P9 (pi=181→182) — 글박스 호스트 직후 paragraph 줄간격

한컴 2010 / 한컴 2020 / 한컴 PDF 와 비교.

**2차 (rhwp-studio web Canvas)**: WASM 동기화 후 같은 fixture 시각 판정.

### 2.6 검토 문서 + 보고서 commit + 머지

```bash
git add mydocs/pr/pr_538_review.md \
        mydocs/pr/pr_538_review_impl.md \
        mydocs/pr/pr_538_report.md
git commit -m "PR #538 처리 보고서 + 검토 문서 (cherry-pick @planet6897 12 commits)"

git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: PR #538 cherry-pick (Task #534 v1+v2 + #537 + #539 — cherry-pick @planet6897 12 commits) — closes #534 #537 #539"
git push origin devel
```

### 2.7 PR / 이슈 close + 컨트리뷰터 안내

```bash
gh pr close 538 --repo edwardkim/rhwp --comment "..."
# 이슈 #537/#539/#534 는 이미 closed 상태 — 정정 적용으로 close 유지 정합
# milestone v1.0.0 추가 (사후 처리)
```

**컨트리뷰터 안내 핵심**: 다음 PR 전 devel 동기화 부탁 (현 PR 의 fork devel 분기로 PR 가 비정상 상태였음).

## 3. 위험 정리

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| `mydocs/orders/20260503.md` 충돌 | 🟥 거의 확실 | 수동 해소 (양쪽 보존) |
| Task #534 v1 의 코드 변경 (`layout.rs` +25/-2) 후 v2 (`+12/-1`) 의 누적 정합 | 🟧 중간 | 결정적 검증으로 검출 |
| 2 task 의 통합 테스트가 본 환경의 사전 테스트와 충돌 | 🟧 중간 | cargo test 로 검출 |
| 시각 판정 미통과 가능성 | 🟧 중간 | 작업지시자 직접 판정 |
| WASM 영향 | 🟧 중간 | 머지 전 재빌드 + 시각 판정 2차 |

## 4. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 안내
- `feedback_release_sync_check` — 사전 main 동기화 점검
- `feedback_v076_regression_origin` — 작업지시자 직접 시각 판정
- `feedback_visual_regression_grows` — 시각 판정 1차 + 2차
- `feedback_image_renderer_paths_separate` — `layout.rs` 의 정정이 SVG / web Canvas 양쪽 영향. 시각 판정 2차 필수
- `feedback_assign_issue_before_work` — 이슈 #537/#539/#534 assignee 부재 사례

## 5. 다음 단계

작업지시자 본 구현 계획서 승인 (또는 검토 문서 승인 시 함께) 후:

1. cherry-pick 12 commits + 충돌 해소
2. 결정적 검증 + WASM 빌드 + studio 동기화
3. ※ 작업지시자 시각 판정 1차 + 2차
4. `pr_538_report.md` 작성 + commit + devel 머지 + push
5. PR/이슈 close + 컨트리뷰터 안내 + archives + 오늘할일 갱신
