# PR #507 처리 보고서

**PR**: [#507 fix: CASES+EQALIGN 중첩 토폴로지 분수 분실 정정 (#505 / #175 후속)](https://github.com/edwardkim/rhwp/pull/507)
**작성자**: @cskwork (Agentic-Worker) — 외부 컨트리뷰터 첫 PR
**처리 결정**: ✅ **머지** (cherry-pick @cskwork 2 commits)
**처리일**: 2026-05-03

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | cherry-pick 머지 |
| cherry-pick 대상 | 2 commits (12037a43 + 4b1feeac) |
| author 보존 | ✅ @cskwork (cherry-pick default) |
| 충돌 | 0 (auto-merge `mydocs/orders/20260501.md` 정합) |
| 결정적 검증 | 모두 통과 |
| 시각 판정 1차 (SVG) | ✅ 4/4 통과 |
| 시각 판정 2차 (rhwp-studio + 한컴) | ✅ 4/4 통과 |
| WASM 빌드 | ✅ 성공 (4,461,235 bytes) |

## 2. cherry-pick 결과

### 2.1 적용된 commits (local/devel 기준)

| 신 commit | 원본 PR commit | 설명 |
|----------|--------------|------|
| `7bcbe2c` | `12037a43` | Task #505: CASES+EQALIGN+MATRIX 중첩 토폴로지 분수 분실 정정 |
| `1f65919` | `4b1feeac` | Task #505: 시각 판정용 fixture HWP 추가 (PR #507 코멘트 응답) |

cherry-pick 의 default 동작으로 author = @cskwork 유지, committer = edward (메인테이너).

### 2.2 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | +51 / -38 (헬퍼 추출 + 6 호출지점 통합) |
| `src/renderer/equation/tokenizer.rs` | +3 / -1 (`skip_spaces` 에 `\n`/`\r` 추가) |
| `tests/issue_505.rs` (신규) | +292 (회귀 테스트 9건) |
| `samples/issue-505-equations.hwp` (신규) | 12,800 bytes (시각 판정용 fixture) |
| `examples/build_issue_505_fixture.rs` (신규) | +134 (재현 가능 빌더) |
| `mydocs/plans/task_m100_505{,_impl}.md` (신규) | 컨트리뷰터의 절차 문서 |
| `mydocs/working/task_m100_505_stage{1-4}.md` (신규) | 단계별 보고서 |
| `mydocs/report/task_m100_505_report.md` (신규) | 최종 보고서 |
| `mydocs/tech/all_in_one_parser_fidelity_strategy.md` (신규) | 1:1 정합화 전략 |
| `mydocs/orders/20260501.md` | +12 (auto-merge 정합) |

## 3. 검증 결과

### 3.1 결정적 검증 (모두 통과)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | ✅ **1110 passed** (PR 본문 1102 +8 — 본 사이클 PR #506/#509/#510 머지 누적) |
| `cargo test --test issue_505` | ✅ **9/9 passed** (회귀 0) |
| `cargo test --test issue_418` | ✅ 1 passed (회귀 0) |
| `cargo test --test issue_501` | ✅ 1 passed (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** (이전 검토 시 5/6 → cherry-pick 후 6/6) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo clippy --test issue_505 -- -D warnings` | ✅ 0 건 |
| `cargo build --release` | ✅ Finished |

### 3.2 WASM 빌드

| 산출물 | 크기 |
|--------|------|
| `pkg/rhwp_bg.wasm` | 4,461,235 bytes (이전 4,456,858 +4,377 — parser/tokenizer 정정 반영) |
| `pkg/rhwp.js` | 231,609 bytes (변동 없음) |
| `rhwp-studio/public/rhwp_bg.wasm` | ✅ 동기화 |
| `rhwp-studio/public/rhwp.js` | ✅ 동기화 |

### 3.3 시각 판정 결과 (작업지시자 직접 판정)

#### 1차 (SVG, CLI export-svg)

| 페이지 | fixture | 결과 |
|--------|---------|------|
| 1 | pi=151 | ✅ 통과 |
| 2 | pi=165 (본 이슈 핵심) | ✅ 통과 — `{1} over {2} x^2` 분수 인식 (squashing 해소) |
| 3 | pi=196 | ✅ 통과 |
| 4 | pi=227 | ✅ 통과 |

#### 2차 (rhwp-studio web Canvas + 한컴 2010/2020)

WASM 재빌드 + studio 동기화 후, 작업지시자가 rhwp-studio (web Canvas) 에서 동일 fixture 4 페이지를 표시 + 한컴 2010/2020 출력과 비교:

| 페이지 | 결과 |
|--------|------|
| 1 (pi=151) | ✅ 통과 |
| 2 (pi=165) | ✅ 통과 |
| 3 (pi=196) | ✅ 통과 |
| 4 (pi=227) | ✅ 통과 |

작업지시자 인용:
> 웹 데이터에서도 시각 판정 통과입니다.

→ web Canvas 와 SVG 의 시각 정합. equation 영역의 renderer 별 시각 차이 없음 (메모리 `feedback_image_renderer_paths_separate` 의 image 영역 외, equation 영역도 정합 입증).

## 4. 본 PR 의 본질 정리

### 4.1 결함

PR #396 (Task #175/#174, EQALIGN 영역) 이 다루지 못한 **CASES+EQALIGN+MATRIX 중첩 토폴로지의 분수 분실 결함**.

### 4.2 근본 원인

`src/renderer/equation/parser.rs` 의 `parse_command` 가 OVER/ATOP 단독 호출 시 `EqNode::Empty` 로 폐기. 중위 연산자 처리는 `parse_expression` / `parse_group` 에만 존재하나, `parse_cases` / `parse_pile` / `parse_eqalign` / `parse_matrix` 는 `parse_element` 직접 호출 → OVER 분실.

### 4.3 정정

- `try_consume_infix_over_atop()` 헬퍼 추출 → 6 호출지점 통합 (DRY)
- `tokenizer.skip_spaces` 에 `\n`/`\r` 추가 (HWP 수식 스크립트는 `#`/`&` 으로 명시적 행/탭 구분)

## 5. 컨트리뷰터 정합

본 PR 은 외부 컨트리뷰터 @cskwork 의 첫 PR. 다음 정합한 절차를 수행:

1. **본질 정확 진단** — PR #396 이 다루지 못한 토폴로지를 정밀 식별 + Self-review 로 `parse_matrix` 까지 확장 (동일 결함 클래스).
2. **DRY 헬퍼 추출** — 6 호출지점 통합으로 변경 영역 본질화.
3. **풍부한 회귀 테스트** — PR 본문 4건 → 9건 (matrix bare/braced OVER+ATOP, pile bare OVER+ATOP, cases bare ATOP, chained OVER 좌결합, orphan OVER 안전성 추가).
4. **수정 요청 정확 대응** — 메인테이너의 시각 판정 fixture 부재 지적에 저작권 회피 + 직접 작성 fixture (`samples/issue-505-equations.hwp` + `examples/build_issue_505_fixture.rs`) 로 대응. baseline 메타데이터 보존 + Equation script 만 교체로 한컴 호환성 위험 최소.
5. **재현 가능 빌더** — `examples/build_issue_505_fixture.rs` 로 fixture 재현성 확보.
6. **내부 워크플로우 정합** — 수행 계획서 / 구현 계획서 / 단계별 보고서 / 최종 보고서 / 1:1 정합화 전략 5종 작성.

## 6. 머지 절차

### 6.1 cherry-pick + 검토 문서 commit

```bash
git checkout local/devel
git stash push -u -m "PR #507 review docs" mydocs/pr/pr_507_review.md mydocs/pr/pr_507_review_impl.md
git cherry-pick 12037a43   # → 7bcbe2c
git cherry-pick 4b1feeac   # → 1f65919
git stash pop
# 검토 문서 + report 함께 commit (다음 단계)
```

### 6.2 devel 머지 + push

```bash
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: PR #507 cherry-pick (Task #505 CASES+EQALIGN+MATRIX 분수 분실 정정 — cherry-pick @cskwork 2 commits) — closes #505"
git push origin devel
```

### 6.3 PR / 이슈 close + 컨트리뷰터 인사

```bash
gh pr close 507 --repo edwardkim/rhwp --comment "..."
gh issue close 505 --repo edwardkim/rhwp --comment "..."
```

## 7. 사후 처리

- [ ] PR #507 close (수동, cherry-pick 머지로 GitHub 자동 close 미동작)
- [ ] 이슈 #505 close (PR 본문에 `closes #505` 명시)
- [ ] 이슈 #505 milestone v1.0.0 추가 (PR 의 milestone 정합)
- [ ] README 기여자 목록 갱신 (@cskwork 첫 PR — 본 사이클 일괄 또는 별도)
- [ ] 컨트리뷰터 인사 댓글 (작업지시자 직접 작성 권장)

## 8. 메모리 정합

- ✅ `feedback_check_open_prs_first` — 본 PR 처리 정합 (이슈 #505 → PR #507 연결 확인 완료)
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심
- ✅ `feedback_release_sync_check` — cherry-pick 시점 main 동기화 점검 (`0fb3e675` 정합)
- ✅ `feedback_v076_regression_origin` — fixture HWP 추가 후 작업지시자 직접 시각 판정 통과 (외부 환경 자료 정답지 사용 회피)
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 통과 (1차 SVG + 2차 rhwp-studio web Canvas 둘 다)
- ✅ `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 작업지시자 한컴 2010/2020 직접 판정으로 정답지 확정
- ✅ `feedback_image_renderer_paths_separate` 정합성 입증 — equation 영역의 web Canvas / SVG 시각 정합 (별도 image 함수 위험이 equation 에는 적용 안 됨)
