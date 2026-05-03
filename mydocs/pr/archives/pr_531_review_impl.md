# PR #531 cherry-pick 구현 계획서

**대상 PR**: [#531 fix(layout): TAC inline 경로 Top caption 오프셋 누락 정정 (Task #530)](https://github.com/edwardkim/rhwp/pull/531)
**작성자**: @postmelee (Taegyu Lee)
**시각 판정 1차 결과**: ✅ **통과** (작업지시자 한컴 2010/2020 직접 비교, 2026-05-03)
**부수 발견**: 이슈 [#543](https://github.com/edwardkim/rhwp/issues/543) 등록 — TAC 그림 (s0:pi=56) 의 동일 본질 캡션 겹침 (Shape 경로, 별도 task)
**작성일**: 2026-05-03

## 1. cherry-pick 전략

### 1.1 대상 commits

PR #531 의 `task530-tac-top-caption` 브랜치에서 3 commit 을 `local/devel` 로 cherry-pick:

| 순서 | commit | 설명 |
|------|--------|------|
| 1 | `ac298d4` | Task #530 Stage 3: TAC Top caption 표 본문 y 오프셋 정정 |
| 2 | `104b4f7` | Task #530 Stage 4: 회귀 검증 |
| 3 | `b3d848b` | Task #530 Stage 5: 최종 보고 |

3 commits 의 단계별 분리가 깔끔하므로 그대로 cherry-pick.

### 1.2 충돌 가능성 점검

| 영역 | 영향 | 충돌 위험 |
|------|------|----------|
| `src/renderer/layout/table_layout.rs` (+18 / -2) | inline_top_caption_offset 변수 추가 + table_y 보정 | 🟢 작음 — 본 영역 main 변경 없음 |
| `tests/issue_530.rs` (신규 +99) | 회귀 테스트 1건 | 🟢 충돌 0 (신규 파일) |
| `mydocs/plans/task_m100_530{,_impl}.md` (신규) | 컨트리뷰터의 절차 문서 | 🟢 충돌 0 (신규 파일) |
| `mydocs/working/task_m100_530_stage{1,3,4}.md` (신규) | 단계별 보고서 | 🟢 충돌 0 (신규 파일) |
| `mydocs/report/task_m100_530_report.md` (신규) | 최종 보고서 | 🟢 충돌 0 (신규 파일) |
| `mydocs/orders/20260502.md` (+2 / -1) | 컨트리뷰터 일지 | 🟧 작음 — main 의 동일 파일 점검 필요 |

PR #507 의 cherry-pick 에서 동일한 `mydocs/orders/2026YYMM.md` 파일이 auto-merge 정합으로 처리된 사례 있음 — 동일 패턴 적용 가능.

## 2. cherry-pick 절차

### 2.1 메인테이너 워크플로우

```bash
# 0. 사전 점검 (메모리 feedback_release_sync_check)
git fetch origin
git pull --ff-only origin main 2>&1 || echo "main divergence detected — manual check"

# 1. local/devel 로 스위치 + 검토 문서 stash
git checkout local/devel
git stash push -u -m "PR #531 review docs" \
    mydocs/pr/pr_531_review.md \
    mydocs/pr/pr_531_review_impl.md

# 2. cherry-pick (시간 순서대로)
git cherry-pick ac298d4
git cherry-pick 104b4f7
git cherry-pick b3d848b

# 3. 검토 문서 복원
git stash pop

# 4. 결정적 검증
cargo test --lib                        # 기준: 1110+ 통과
cargo test --test issue_530             # 1 passed
cargo test --test issue_418             # 회귀 0
cargo test --test issue_501             # 회귀 0
cargo test --test svg_snapshot          # 6/6 통과
cargo clippy --lib -- -D warnings       # 0 건
cargo clippy --test issue_530 -- -D warnings  # 0 건
cargo build --release                   # 정상

# 5. WASM 빌드 + studio 동기화 (※ 머지 전 필수 게이트)
docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp_bg.wasm rhwp-studio/public/rhwp_bg.wasm
cp pkg/rhwp.js     rhwp-studio/public/rhwp.js

# 6. ※ 작업지시자 최종 시각 판정 게이트 (rhwp-studio + 한컴 2010/2020)
#    samples/basic/treatise sample.hwp page=5 (global_idx=4) 우측 단 표 시각 확인
#    (PR #531 본질: TAC 표 + Top caption 분리)

# 7. 작업지시자 최종 승인 후 PR 검토 문서 + 최종 보고서 commit
git add mydocs/pr/pr_531_review.md \
        mydocs/pr/pr_531_review_impl.md \
        mydocs/pr/pr_531_report.md
git commit -m "PR #531 처리 보고서 + 검토 문서 갱신 (cherry-pick @postmelee 3 commits)"

# 8. devel 머지 + push
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: PR #531 cherry-pick (Task #530 TAC Top caption 표 본문 y 오프셋 정정 — cherry-pick @postmelee 3 commits)"
git push origin devel

# 9. PR close + 컨트리뷰터 인사 + 이슈 #530 처리
gh pr close 531 --repo edwardkim/rhwp --comment "..."
# 이슈 #530 close 결정은 작업지시자 (PR 본문 Refs #530, not closes — 이슈 본질 잔여 가능성)
```

### 2.2 cherry-pick conflict 발생 시 대응

`mydocs/orders/20260502.md` 충돌 가능성:
- main 의 동일 파일에 본 사이클 다른 작업 (PR #506/#507/#510 등) 일지 추가됨
- 대응: 양쪽 보존 (auto-merge 가능 시) 또는 작업지시자 결정

PR #507 의 사례에서 동일 패턴이 auto-merge 정합으로 처리됨 → 위험 작음.

### 2.3 cherry-pick author 보존

- `git cherry-pick` 의 default 동작이 author 보존 → @postmelee 의 attribution 유지
- 메인테이너는 committer 로 기록 (정합)

## 3. 검증 게이트

### 3.1 결정적 검증 (PR 검토 단계 통과 ✅)

PR 검토 단계에서 본 환경에서 직접 실행 + 통과 확인:

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ 1110 passed |
| `cargo test --test issue_530` | ✅ 1 passed |
| `cargo test --test issue_418/501` | ✅ 회귀 0 |
| `cargo test --test svg_snapshot` | ✅ 6/6 통과 |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo clippy --test issue_530 -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

cherry-pick 후 동일 게이트 재실행하여 최종 통과 확인.

### 3.2 시각 판정 1차 (SVG, 작업지시자, 통과 ✅)

`output/svg/pr531/treatise sample_005.svg` 의 시각 판정:

| 항목 | 결과 |
|------|------|
| 우측 단의 TAC 표 (pi=60 ci=0) | ✅ 통과 — Top caption 두 줄 / 머리행 / 본문 정상 분리 |
| caption baseline y | 551.32 (유지) |
| 머리행 top y | 525.49 → 560.83 (caption 아래로 이동) |
| 부수 발견: s0:pi=56 그림 캡션 겹침 | 별도 이슈 [#543](https://github.com/edwardkim/rhwp/issues/543) 등록 |

### 3.3 시각 판정 2차 (rhwp-studio, 머지 전 최종 게이트)

cherry-pick + 결정적 검증 + WASM 빌드 후, 작업지시자가 **rhwp-studio (web Canvas)** 에서 동일 fixture 시각 확인 + 한컴 2010/2020 비교.

**판정 절차:**
1. cherry-pick + 결정적 검증 통과 후 본 환경에서 WASM 재빌드 + studio 동기화
2. `cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700` 으로 dev server 기동
3. 브라우저에서 `samples/basic/treatise sample.hwp` 로드 → 페이지 5 시각 확인
4. 한컴 2010 + 한컴 2020 의 동일 페이지와 비교

**판정 결과 기준:**
- 페이지 5 우측 단 TAC 표가 SVG 와 동일한 시각 (web Canvas + SVG 정합)
- caption / 머리행 / 본문 정상 분리
- web Canvas 만의 회귀 결함 없음

**미통과 시 대응:**
- web Canvas 와 SVG 의 시각 차이 발견 시 — 별도 task 분리
- 작업지시자 결정으로 머지 보류 또는 추가 조정 후 재판정

## 4. PR / 이슈 close 정합

### 4.1 PR #531 close 절차

cherry-pick 머지 후 PR 본문에 `Refs #530` (closes 아님) 명시. 작업지시자가 #530 close 결정.

cherry-pick 머지 commit 메시지:

```
Merge local/devel: PR #531 cherry-pick (Task #530 TAC Top caption 표 본문 y 오프셋 정정 — cherry-pick @postmelee 3 commits) — refs #530
```

### 4.2 이슈 #530 close 결정 (작업지시자)

PR #531 의 정정이 #530 의 본질 (TAC + Top caption 표 겹침) 을 완전히 해결. 그러나 PR 본문이 `Refs #530` (not closes) 으로 명시됨 — 컨트리뷰터의 신중한 표현. 작업지시자가 다음 중 결정:

- **A**. 이슈 #530 close (정정 충분, 다른 결함 잠재 없음)
- **B**. 이슈 #530 open 유지 (다른 결함 잠재 또는 추가 검증 필요)

### 4.3 부수 이슈 (#543) 처리

본 PR 의 시각 판정 중 발견된 s0:pi=56 그림의 동일 본질 캡션 겹침 결함은 [이슈 #543](https://github.com/edwardkim/rhwp/issues/543) 으로 별도 등록. 본 PR 의 처리 보고서에 부수 발견으로 기록.

### 4.4 README 기여자 목록 갱신

@postmelee 의 PR 카운트 누적. 본 사이클 일괄 갱신 시점에 반영.

### 4.5 milestone 사후 처리

이슈 #530:
- milestone 미지정 → v1.0.0 추가 (PR #531 의 milestone 정합)

## 5. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심
- `feedback_release_sync_check` — cherry-pick 시점 main 동기화 점검 필수
- `feedback_release_manual_required` — 본 PR 은 사이클 내 PATCH 단위 머지 (릴리즈 트리거 아님), 매뉴얼 정독 의무 해당 없음
- `feedback_v076_regression_origin` — 작업지시자 직접 시각 판정 1차 통과
- `feedback_visual_regression_grows` — 시각 판정 게이트 (1차 SVG + 2차 web Canvas) 모두 진행
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정
- `feedback_image_renderer_paths_separate` — 이슈 #543 의 본질 (Shape 경로의 동일 결함 클래스) 정합
- `feedback_hancom_compat_specific_over_general` — case-specific 정정 (TAC + Top caption + depth=0 한정), 정합

## 6. 위험 정리

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| cherry-pick 시 `mydocs/orders/20260502.md` 충돌 | 🟧 작음 | PR #507 사례 처럼 auto-merge 정합 가능. 충돌 시 양쪽 보존 |
| svg_snapshot 6/6 회귀 (사전 영역 변경) | 🟢 매우 작음 | PR 검토 단계에서 6/6 통과 확인 ✅ |
| WASM 빌드 영향 | 🟧 중간 | 본 PR 은 native lib (table_layout.rs) 변경. WASM 의 동일 영역 영향 받음 — 재빌드 + 작업지시자 시각 판정 2차 게이트 필수 |
| web Canvas 와 SVG 시각 차이 | 🟧 중간 | 메모리 `feedback_image_renderer_paths_separate` — equation 영역 정합 입증 (PR #507 사례). 표 영역도 정합 가능성 높음. 시각 판정 2차에서 검출 |
| 한컴 2010/2020 호환성 | 🟢 매우 작음 | 시각 판정 1차 통과 (한컴 호환성 정합 입증) |
| 부수 이슈 (#543) 의 본 PR 영향 | 🟢 없음 | 이슈 #543 은 별도 영역 (Shape 경로). 본 PR 은 표 영역만 정정 |

## 7. 단계 구분 (PR 검토 절차 정합)

CLAUDE.md `pr/` 절차에 따라 단계별 보고서 (stage) 불필요. 본 구현 계획서 승인 후 cherry-pick + 검증 + WASM + 시각 판정 2차 + 머지 + 보고서 작성을 한 절차로 진행.

## 8. 다음 단계

작업지시자 본 구현 계획서 승인 후:

1. cherry-pick 절차 진행 (위 §2.1 절차 1~4 단계)
2. 결정적 검증 게이트 통과 확인 (위 §3.1)
3. WASM 빌드 + studio 동기화 (위 §2.1 절차 5단계)
4. **※ 작업지시자 최종 시각 판정 게이트 2차** (위 §3.3, rhwp-studio + 한컴 2010/2020 비교)
5. **시각 판정 통과 후에만** PR 검토 문서 + 보고서 commit + devel 머지 + push
6. `pr_531_report.md` 작성
7. PR #531 close + 이슈 #530 처리 (작업지시자 결정) + 컨트리뷰터 인사
8. README 기여자 목록 갱신 (별도 또는 본 사이클 일괄)
