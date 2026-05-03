# PR #507 cherry-pick 구현 계획서

**대상 PR**: [#507 fix: CASES+EQALIGN 중첩 토폴로지 분수 분실 정정 (#505 / #175 후속)](https://github.com/edwardkim/rhwp/pull/507)
**작성자**: @cskwork (Agentic-Worker)
**시각 판정 결과**: ✅ **4/4 통과** (작업지시자 한컴 2010/2020 직접 비교, 2026-05-03)
**작성일**: 2026-05-03

## 1. cherry-pick 전략

### 1.1 대상 commits

PR #507 의 `feature/issue-505-cases-eqalign-fraction` 브랜치에서 두 commit 을 `local/devel` 로 cherry-pick:

| 순서 | commit | 설명 |
|------|--------|------|
| 1 | `12037a43` | Task #505: CASES+EQALIGN+MATRIX 중첩 토폴로지 분수 분실 정정 (parser + tests + 문서) |
| 2 | `4b1feeac` | Task #505: 시각 판정용 fixture HWP 추가 (`samples/issue-505-equations.hwp` + `examples/build_issue_505_fixture.rs`) |

merge commit `7909e6d5` 는 cherry-pick 대상 **아님** (devel 변동만 흡수하는 머지 commit, 본 PR 의 본질 변경 아님).

### 1.2 충돌 가능성 점검

| 영역 | 영향 | 충돌 위험 |
|------|------|----------|
| `src/renderer/equation/parser.rs` (+51 / -38) | parser 의 row-collecting 루프 (`try_consume_infix_over_atop` 헬퍼 추출) | 🟢 작음 — 본 영역 main 변경 없음 |
| `src/renderer/equation/tokenizer.rs` (+3 / -1) | `skip_spaces` 에 `\n`/`\r` 추가 | 🟢 작음 |
| `tests/issue_505.rs` (신규 +292) | 신규 외부 통합 테스트 9건 | 🟢 충돌 0 (신규 파일) |
| `samples/issue-505-equations.hwp` (신규) | 시각 판정 fixture | 🟢 충돌 0 (신규 파일) |
| `examples/build_issue_505_fixture.rs` (신규) | 재현 가능 빌더 | 🟢 충돌 0 (신규 파일) |
| `mydocs/plans/task_m100_505.md` 등 9 파일 (신규) | 컨트리뷰터의 절차 문서 | 🟢 충돌 0 (신규 파일) |
| `mydocs/orders/20260501.md` (+12) | 컨트리뷰터의 일지 | 🟧 작음 — main 의 동일 파일 점검 필요 |

### 1.3 충돌 점검 결과 — main 의 mydocs/orders/20260501.md

`local/devel` 의 동일 파일 존재 여부 + 충돌 가능성 확인 (cherry-pick 진행 중 git 자체 검출).

## 2. cherry-pick 절차

### 2.1 메인테이너 워크플로우 (CLAUDE.md)

```bash
# 0. 사전 점검 (메모리 feedback_release_sync_check)
git fetch origin
git pull --ff-only origin main 2>&1 || echo "main divergence detected — manual check"

# 1. local/devel 로 스위치
git checkout local/devel

# 2. cherry-pick (시간 순서대로)
git cherry-pick 12037a43
git cherry-pick 4b1feeac

# 3. PR 검토 문서 갱신 commit (재검토 + impl + report 함께)
git add mydocs/pr/pr_507_review.md mydocs/pr/pr_507_review_impl.md mydocs/pr/pr_507_report.md
git commit -m "PR #507 처리 보고서 + 검토 문서 갱신 (cherry-pick @cskwork 2 commits)"

# 4. 결정적 검증
cargo test --lib                      # 기준: 1102+ 통과 (PR #506/#509/#510 머지 후 누적)
cargo test --test issue_505           # 기준: 9/9 통과
cargo test --test issue_418           # 회귀 0
cargo test --test issue_501           # 회귀 0
cargo test --test svg_snapshot        # 6/6 통과 (PR #506 사전 회귀 정정 후)
cargo clippy --lib -- -D warnings     # 0 건
cargo build --release                 # 정상

# 5. WASM 빌드 + studio 동기화 (※ 머지 전 필수 게이트)
docker compose --env-file .env.docker run --rm wasm
cp pkg/rhwp_bg.wasm rhwp-studio/public/rhwp_bg.wasm
cp pkg/rhwp.js     rhwp-studio/public/rhwp.js

# 6. ※ 작업지시자 최종 시각 판정 게이트 (rhwp-studio + 한컴 2010/2020)
#    samples/issue-505-equations.hwp 4 페이지를 rhwp-studio (web Canvas) 로 표시 +
#    한컴 2010/2020 비교. 시각 판정 통과 후에만 다음 단계 진행.

# 7. 작업지시자 최종 승인 후 devel 머지 + push
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: PR #507 cherry-pick (Task #505 CASES+EQALIGN+MATRIX 분수 분실 정정 — cherry-pick @cskwork 2 commits)"
git push origin devel

# 8. PR close + 이슈 close + 컨트리뷰터 인사
gh pr close 507 --repo edwardkim/rhwp --comment "..."
gh issue close 505 --repo edwardkim/rhwp --comment "..."
```

### 2.2 cherry-pick conflict 발생 시 대응

- `mydocs/orders/20260501.md` 충돌 가능성 — main 의 동일 파일이 다른 내용을 가질 수 있음
- 대응: 양쪽 내용 보존 (작업지시자 메인 일지 + 컨트리뷰터 추가) 또는 작업지시자 main 일지 우선
- 충돌 발생 시 작업지시자 직접 결정 후 진행

### 2.3 cherry-pick author 보존

- `git cherry-pick` 의 default 동작이 author 보존 → @cskwork 의 attribution 유지
- 메인테이너는 committer 로 기록 (정합)

## 3. 검증 게이트

### 3.1 결정적 검증

| 게이트 | 기준 | 비고 |
|--------|------|------|
| `cargo test --lib` | 1102+ 통과 | 본 사이클 PR #506/#509/#510 머지 후 누적치는 더 클 수 있음 |
| `cargo test --test issue_505` | 9/9 통과 | 본 PR 신규 |
| `cargo test --test issue_418/501` | 회귀 0 | PR #396 회귀 영역 |
| `cargo test --test svg_snapshot` | 6/6 통과 | PR #506 사전 CRLF/LF 회귀 정정 후 |
| `cargo clippy --lib -- -D warnings` | 0 건 | 본 PR 변경 영역 + 기존 영역 |
| `cargo build --release` | 정상 | bin rhwp + lib |
| `cargo test --test issue_535` (있는 경우) | 부재 | Task #535 폐기로 본 task 와 무관 |

### 3.2 시각 판정 1차 (작업지시자, 통과 — SVG 출력 기준)

`output/svg/pr507/issue-505-equations_{001..004}.svg` (CLI export-svg 출력) 의 시각 판정. 머지 차단 사유 해소 게이트.

| 페이지 | fixture | 결과 |
|--------|---------|------|
| 1 | pi=151 | ✅ 통과 |
| 2 | pi=165 (본 이슈 핵심) | ✅ 통과 — `{1} over {2} x^2` 분수 인식 (squashing 해소) |
| 3 | pi=196 | ✅ 통과 |
| 4 | pi=227 | ✅ 통과 |

### 3.3 시각 판정 2차 (작업지시자, 머지 전 최종 게이트) — WASM + rhwp-studio

cherry-pick + 결정적 검증 + WASM 빌드 후, 작업지시자가 **rhwp-studio (web Canvas)** 에서 동일 fixture 4 페이지를 표시 + 한컴 2010/2020 비교.

**판정 절차:**
1. cherry-pick 머지 commit 후 본 환경에서 WASM 재빌드 + studio 동기화
2. `cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700` 으로 dev server 기동
3. 브라우저에서 `samples/issue-505-equations.hwp` 로드 → 4 페이지 시각 확인
4. 한컴 2010 + 한컴 2020 의 동일 fixture 와 비교

**판정 결과 기준:**
- 4 페이지 모두 SVG 출력과 동일한 시각 (web Canvas + SVG 정합)
- 한컴 출력 대비 squashing 해소 + 분수 정상 인식 (pi=165 핵심)
- web Canvas 만의 회귀 결함 (서체 / lineSeg / 폰트 / 좌표 변환 등) 없음

**미통과 시 대응:**
- web Canvas 와 SVG 의 시각 차이 발견 시 — `src/renderer/web_canvas.rs` 영역 별도 점검 (PR #507 본질 외 영역, 별도 task 분리)
- 작업지시자 결정으로 머지 보류 또는 추가 조정 후 재판정

### 3.3 fixture HWP 한컴 호환성

- baseline `samples/equation-lim.hwp` 의 메타데이터 (char_count, char_shapes, line_segs, para_shape_id 등) 보존
- `Equation.script` 만 fixture (pi=151/165/196/227) 로 교체
- `Section.raw_stream = None` + `Equation.raw_ctrl_data = Vec::new()` 로 재직렬화 유도
- round-trip parse 4/4 일치 (PR 본문 보고)
- 한컴 2010/2020 시각 판정 통과 (작업지시자 검증)

## 4. PR / 이슈 close 정합

### 4.1 PR #507 close 절차

cherry-pick 머지 후 PR 본문에 `closes #505` 가 명시되어 있으나, cherry-pick 머지는 GitHub 의 자동 close 트리거가 동작하지 않으므로 (devel 으로 cherry-pick → PR 의 base/head 와 다름) **수동 close 필요**.

cherry-pick 머지 commit 메시지에 PR 참조 + 컨트리뷰터 attribution 명시:

```
Merge local/devel: PR #507 cherry-pick (Task #505 CASES+EQALIGN+MATRIX 중첩 토폴로지 분수 분실 정정 — cherry-pick @cskwork 2 commits) — closes #505
```

### 4.2 이슈 #505 close

PR #507 의 본문에 `closes #505` 명시. cherry-pick 머지 후 수동 close + 컨트리뷰터 인사.

### 4.3 README 기여자 목록 갱신

@cskwork 의 첫 PR. README.md 의 기여자 목록에 추가 (메모리 정합 — 외부 컨트리뷰터 기여 인정).

별도 commit 또는 본 사이클 README 갱신 시 일괄 반영.

### 4.4 milestone / assignee 사후 처리

이슈 #505:
- milestone 미지정 → v1.0.0 추가 (PR #507 milestone 정합)
- assignees 없음 → close 시 별도 처리 불필요

## 5. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합 (이슈 #505 → PR #507 연결 확인 완료)
- `feedback_pr_comment_tone` — 차분하고 사실 중심 댓글 (close 시 인사 + 머지 알림)
- `feedback_release_sync_check` — cherry-pick 시점 git pull --ff-only origin main 점검 (필수)
- `feedback_v076_regression_origin` — fixture HWP 추가 후 작업지시자 직접 시각 판정 통과 → 본 PR 의 차단 사유 해소
- `feedback_visual_regression_grows` — 시각 판정 게이트 통과 정합
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정

## 6. 위험 정리

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| cherry-pick 시 `mydocs/orders/20260501.md` 충돌 | 🟧 중간 | 충돌 발생 시 양쪽 보존 or 작업지시자 결정 |
| svg_snapshot 6/6 회귀 (사전 영역 변경) | 🟢 작음 | PR #506 머지 후 정정 완료 가정. cherry-pick 후 재실행 게이트 |
| Task #535 폐기 후 환경 (working tree clean) 영향 | 🟢 없음 | local/devel 시점이 task535 폐기 후 정합 |
| WASM 빌드 영향 | 🟧 중간 | 본 PR 은 native lib (parser/tokenizer) 변경. WASM 의 equation 렌더링 동일 본질 — **재빌드 + 작업지시자 시각 판정 게이트** 필수 |
| web Canvas 와 SVG 시각 차이 | 🟧 중간 | renderer 별 별도 image 함수 메모리 정합 (`feedback_image_renderer_paths_separate`) — equation 영역도 동일 위험 가능성. 시각 판정 2차에서 검출 |
| 한컴 2010/2020 호환성 (fixture HWP) | 🟢 매우 작음 | 작업지시자 시각 판정 1차 4/4 통과 (한컴 호환성 정합 입증) |

## 7. 단계 구분 (PR 검토 절차 정합 — 단계별 보고서 없음)

CLAUDE.md `pr/` 절차에 따라 PR 검토는 단계별 보고서 (stage) 가 불필요. 본 구현 계획서 승인 후 cherry-pick + 검증 + 보고서 작성을 한 절차로 진행.

## 8. 다음 단계

작업지시자 본 구현 계획서 승인 후:

1. cherry-pick 절차 진행 (위 §2.1 절차 1~3 단계)
2. 결정적 검증 게이트 통과 확인 (위 §3.1)
3. WASM 빌드 + studio 동기화 (위 §2.1 절차 5단계)
4. **※ 작업지시자 최종 시각 판정 게이트** (위 §3.3, rhwp-studio + 한컴 2010/2020 비교)
5. **시각 판정 통과 후에만** devel 머지 + push 진행
6. `pr_507_report.md` 작성 (cherry-pick 결과 + 결정적 검증 + WASM + 시각 판정 결과 + close 절차)
7. PR #507 close + 이슈 #505 close + 컨트리뷰터 인사
8. README 기여자 목록 갱신 (별도 또는 본 사이클 일괄)
