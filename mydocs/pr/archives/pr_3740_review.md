---
kind: review
status: code-ci-success-review-only-fast-pass-pending
canonical: mydocs/pr/archives/pr_3740_review.md
last_verified: 2026-08-03
---

# PR #3740 검토 기록 — TAC sibling-line·표/각주 흐름 보정

## 결론

[PR #3740](https://github.com/edwardkim/rhwp/pull/3740)은 텍스트 없는 host 문단에서 두 번째
자리차지 개체가 첫 번째 `LINE_SEG` 높이를 사용해 cursor를 과전진시키고, 조판의 TAC cap은 형제
개체를 과소계상해 뒤 문단을 쪽 밖으로 밀던 [#3738](https://github.com/edwardkim/rhwp/issues/3738)의
핵심 축을 고친다. contributor의 원인 분석은 정확했고, 후속 시각 검토 중 발견한 표 앵커·각주
fragment 경계는 collaborator 보정으로 별도 commit에 한정했다.

code candidate `fd069734f8289e91bf04a1c72933a19fbc0aad50`의 full CI와 로컬 전체
release-test는 통과했다. 이 문서·구현 기록·오늘할일만 담는 trailing commit의 preflight와
`Build & Test` aggregate가 success이면 **merge를 권고한다.** PDF 충실도의 모든 잔여 결함이
해결됐다는 뜻은 아니며, 분리한 후속 범위는 [#3820](https://github.com/edwardkim/rhwp/issues/3820)과
[#3821](https://github.com/edwardkim/rhwp/issues/3821)에서 계속 추적한다.

## PR metadata와 라우팅

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#3740](https://github.com/edwardkim/rhwp/pull/3740) / `@planet6897` |
| 제목 | `fix(render): 한 문단의 두 번째 자리차지 개체가 자기 줄만큼 전진하게 — 뒤 문단이 쪽 밖으로 밀리던 축 (#3738)` |
| base / source | `devel` / `planet6897/rhwp:fix/3738-tac-sibling-line-advance` |
| contributor 원 code commit | `5be9b6ec2c8a801363fe1778976e5f3de3a8b48c` |
| 현재 code candidate | `fd069734f8289e91bf04a1c72933a19fbc0aad50` |
| 변경 규모 | 658 files, +53,558 / -908 (작성 시점 GitHub 참고값) |
| source 수정 권한 | `maintainerCanModify=true` |
| code candidate 원격 상태 | `MERGEABLE`, `CLEAN` (문서 trailing commit 전 재확인값) |
| 관련 issue | [#3738](https://github.com/edwardkim/rhwp/issues/3738), 잔여 [#3820](https://github.com/edwardkim/rhwp/issues/3820), [#3821](https://github.com/edwardkim/rhwp/issues/3821) |

base route: `collaborator_external_pr.md`

modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
`rework_and_exceptions.md`, `review_only_fast_pass.md`, `post_merge.md`

## 변경 검토

원 contributor 변경은 같은 host 문단의 선행 자리차지 개체 수를 `LINE_SEG` index로 사용하도록
조판과 layout의 반대 방향 오류를 함께 맞춘다. 따라서 표 뒤의 글상자가 표의 높이만큼 전진하는
오류와, TAC cap이 형제 글상자 높이를 다시 잘라내는 오류가 서로 상쇄돼 page count만으로는 놓치던
본문 소실을 직접 다룬다.

후속 collaborator commit들은 원 변경을 rewrite하지 않고 source head 뒤에 쌓였다. 시각 검토에서
확인된 HWP/HWPX 그림·caption·RowBreak 표·각주 owner 경계의 회귀를 focused fixture와 evidence로
고정했고, 최종 `fd069734f`는 과도한 raw stored anchor 예외를 실제 그림 구조로 좁혔다. 또한 빈
각주 가상 fragment의 line range를 compose 결과 길이로 clamp하여 CI의 nested split-row 회귀가
panic 없이 원래 계약을 검증하게 했다. 상세 reasoning과 commit 분리는
[구현·보정 기록](pr_3740_review_impl.md)에 남긴다.

## 검증

| 게이트 | 결과 |
| --- | --- |
| full integration | `CARGO_TARGET_DIR=target/review-pr3740-full-regression CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과 (최종 exit 0) |
| CI 실패 회귀 | `issue_3637_split_cell_nested_table_vpos::nested_table_snap_stays_inside_the_split_cell` 통과 |
| #3738 focused / IR sweep | `issue_3738_rowbreak_table_footnote_fragment` 18건 및 IR field sweep 2건 통과 |
| Native Skia | library 58건, missing-picture placeholder 2건, direct PDF export 4건 통과 |
| 정적 검사 | `cargo fmt --all -- --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings` 통과 |
| GitHub code candidate CI | [CI run 30791602165](https://github.com/edwardkim/rhwp/actions/runs/30791602165) — `Build & Test`, default-feature 8 shards, Lint, Native Skia, Canvas visual diff, CodeQL 통과 |
| 사용자 수동 확인 | production `wasm-pack build --target web --out-dir pkg` 통과 (사용자 수행) |

원작자가 남긴 r29 10k oracle 수치는 `8cd4901dd` 시점의 세 PR 조합에만 적용된다. 후속 보정이
누적된 현재 head의 결과로 인용하지 않았으며, contributor가 범위를 명시한 두 코멘트와 현재의
full integration/CI는 별도 근거로 취급했다.

## 시각·fixture 증적

| 자료 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp` | 개인정보 제거 HWP 입력 | `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113` |
| 같은 이름의 `.hwpx` | 동등 HWPX 입력 | `8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c` |
| `pdf/pr3740/hwp/...-2020.pdf` | 한컴 2020 기준 PDF | `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a` |

대표 증적은 p78–80 표 25 URL 각주 owner의 [Stage 17 visual sweep](../../working/task_m100_3738_stage17_visual_sweep.md)과
`mydocs/pr/assets/pr_3740_issue3738_stage17/`의 review PNG, p90 표 27 row owner의
`mydocs/pr/assets/pr_3740_issue3738_stage31_p90_table_owner/`에 보존했다. p23–24, p25–27,
p30–32, p37, p43, p44–45, p52–54, p58–59, p66–67, p68–70, p76–80, p83–85, p90, p127은
최신 evidence에서 해결 또는 current exact revision에서 비재현으로 판정했다. 이 판정은 전체
PDF page-map의 완전 정합을 주장하지 않는다.

## 잔여 범위와 merge 후속 처리

현재 기준 PDF 215쪽과 native SVG/render tree 219쪽의 page-map 차이, p94 표 28, p106 표 29,
p107–108 owner 경계 및 p156 Square-wrap은 이 PR의 merge 차단 조건으로 확장하지 않고
[#3820](https://github.com/edwardkim/rhwp/issues/3820)·[#3821](https://github.com/edwardkim/rhwp/issues/3821)에
분리했다. 각각 PDF 직접 대조와 dedicated regression이 준비된 뒤에만 해결로 전환한다.

이 trailing commit은 `mydocs/`만 바꾸는 single-parent review-only commit이다. push 전 source SHA와
LFS 대상 여부를 재확인한 뒤 `GIT_LFS_SKIP_PUSH=1` dry-run 및 실제 push를 수행한다. code candidate
CI 30791602165의 `Build & Test`가 success인 것을 A 경로의 candidate로 사용하되, **latest docs head의
preflight와 `Build & Test` aggregate가 success일 때만** merge한다. merge 뒤에는 devel fast-forward,
#3738 상태 확인, contributor fork branch를 삭제하지 않는 branch/worktree/target cleanup, 그리고
사용자 승인 뒤의 원작자 답글·merge comment를 수행한다.
