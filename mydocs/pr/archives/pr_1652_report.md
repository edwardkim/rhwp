# PR #1652 사전 처리 판단 보고서 — 페이지↔PI 검증 도구 + TAC 표 직후 빈 문단 누락 수정

## 1. 결정

**merge 수용 권고** — 최신 코드 head `071e9771` 기준 로컬 검증과 GitHub Actions가 통과했다.

이 보고서는 merge 전 사전 판단 보고서다. merge 완료, merge SHA, 이슈 close 완료 여부를 확정 사실로 기록하지
않는다. 문서 커밋 push 후 최신 PR head 기준 CI와 merge state를 다시 확인해야 한다.

## 2. 변경 본질

이번 PR은 #1643의 페이지↔PI 비교 도구와 #1648의 TAC 표 직후 빈 문단 누락 수정을 함께 제출한다.

- `tools/verify_pi_page_vs_hangul.py`: rhwp `dump-pages` 결과와 한글 OLE 자동화 결과를 PI 단위로 비교한다.
- `src/renderer/typeset.rs`: `next_will_vpos_reset`에서 빈 문단을 fit 검사 없이 skip하던 경로를 보정한다.
- `tests/issue_1116.rs`: 한컴 기준으로 trailing 빈 문단이 보존되는 sample16 기대값을 갱신한다.
- `mydocs/report/task_m100_1648_report.md`: #1648 수정과 #1659 회귀 보정 내용을 기록한다.

초기 수정은 height-only fit으로 충분하지 않았고, #1659에서 `synam-001` 페이지수 35→36 회귀가 확인됐다.
최신 head는 placement와 맞춘 vpos fit을 AND로 추가하여 이 회귀를 해소했다.

## 3. 판단 근거

- PR base는 `devel`이고, 문서 작성 시점 merge state는 `CLEAN`이다.
- contributor 원 commit은 `071e9771` 1개이며, collaborator가 원 커밋을 rewrite하지 않는다.
- 이전 review에서 요청한 3건은 모두 반영됐다.
  - report가 #1659 회귀 보정까지 포함하도록 갱신됨
  - `MATCH` 샘플도 TSV 행으로 기록됨
  - `PartialParagraph` continuation의 top vpos 기준이 `start_line`으로 보정됨
- 최신 코드 head 기준 GitHub Actions는 통과했다.
  - Build & Test: success
  - CodeQL: success
  - Canvas visual diff: success
- `closingIssuesReferences`가 비어 있으므로 issue close는 merge 후 별도 확인이 필요하다.

## 4. 검증 결과

| 항목 | 결과 |
|------|------|
| GitHub CI (`071e9771`) | Build & Test / CodeQL / Canvas visual diff 성공 |
| `python3 -B -m py_compile tools/verify_pi_page_vs_hangul.py` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_1156_rowbreak_fragment_fit` | 통과 (3 passed) |
| `cargo test --test issue_1116` | 통과 (13 passed) |
| `cargo test --test issue_676_trailing_empty_para` | 통과 (3 passed) |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --lib` | 통과 (1988 passed, 0 failed, 7 ignored) |

PR 작성자가 보고서에 기록한 한컴 환경 검증:

- 재현 36399821: 빈 문단 `pi=2`가 1쪽에 배치되고 페이지수 5 유지
- `synam-001`: #1659 회귀 보정 후 35쪽 유지
- sample16 p3: trailing 빈 문단 `pi=87` emit 보존
- 400-HWPX 통제 비교: PAGE_INFLATE 0 / PAGE_DEFLATE 0 / ADD_ONLY 9 / REMOVED 0 / CROSS_MOVED 0

## 5. PR head 문서 push 계획

Route A, collaborator-mediated 외부 PR 경로로 처리한다.

- 문서 추가 경로:
  - `mydocs/pr/archives/pr_1652_review.md`
  - `mydocs/pr/archives/pr_1652_report.md`
- 문서 commit은 contributor 원 commit 뒤에 별도 commit으로 추가한다.
- push 대상은 `planet6897:task/1643-1648-pi-page-verify`이다.
- push 후 PR diff에 위 두 문서만 추가됐는지 확인한다.
- push 후 최신 PR head 기준 GitHub Actions를 다시 확인한다.

## 6. 기여자 credit

- 원 PR: https://github.com/edwardkim/rhwp/pull/1652
- 원 작성자: @planet6897 (Jaeuk Ryu)
- 원 소스 커밋:
  - `071e9771a3038e93964531a41b0ec2194ce2f277`
    `Task #1643/#1648: 페이지↔PI 검증 도구 + TAC 표 직후 빈 문단 누락 수정`
- collaborator 문서 커밋은 검토 기록만 추가하며 contributor 코드 credit을 변경하지 않는다.

## 7. merge 전 조건

- 문서 커밋이 포함된 최신 PR head 기준 GitHub Actions 통과
- PR diff에 `pr_1652_review.md`, `pr_1652_report.md` 포함 확인
- 해소된 inline review thread resolve 처리
- approval review 제출
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 merge 승인

## 8. merge 후 확인 계획

- `#1643`, `#1648`, `#1659` state 확인
- 자동 close가 되지 않으면 작업지시자 승인 후 수동 close 및 comment 처리
- PR에 merge 완료 comment 작성
- 필요 시 로컬 `devel` sync 및 작업 worktree 정리
