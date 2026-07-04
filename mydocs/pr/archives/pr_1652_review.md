# PR #1652 검토 — 페이지↔PI 검증 도구 + TAC 표 직후 빈 문단 누락 수정

- 작성일: 2026-06-29 18:02 KST
- 컨트리뷰터: [@planet6897](https://github.com/planet6897) (Jaeuk Ryu)
- PR: https://github.com/edwardkim/rhwp/pull/1652
- base/head: `devel` `180e394d` ← `planet6897:task/1643-1648-pi-page-verify` `071e9771`
- 관련 이슈: `Closes #1643`, `Closes #1648`, `Closes #1659`
- 규모: +388 / -6, 4 files
- 상태 참고값: open, draft 아님, `MERGEABLE` / `CLEAN`, 기존 review decision은 `CHANGES_REQUESTED`

위 상태값은 문서 작성 시점 참고값이다. merge 전에는 최신 PR head 기준 `mergeable`, `mergeStateStatus`,
GitHub Actions, review decision을 다시 확인해야 한다.

## 1. PR 정보

변경 파일:

| 파일 | 변경 |
|------|------|
| `tools/verify_pi_page_vs_hangul.py` | 한글 OLE와 rhwp `dump-pages`의 PI별 페이지 배치를 대조하는 검증 도구 추가 |
| `src/renderer/typeset.rs` | TAC 표 직후 빈 문단 누락 방지 fit 검사 추가 및 #1659 vpos 보정 |
| `tests/issue_1116.rs` | sample16 trailing 빈 문단 배치 기대값 갱신 |
| `mydocs/report/task_m100_1648_report.md` | #1648 수정 및 #1659 회귀 보정 검증 보고서 추가 |

커밋:

| SHA | 내용 | 작성자 |
|-----|------|--------|
| `071e9771` | `Task #1643/#1648: 페이지↔PI 검증 도구 + TAC 표 직후 빈 문단 누락 수정` | @planet6897 |

## 2. 변경 내용 분석

이번 PR은 두 가지 작업을 함께 담는다.

1. `tools/verify_pi_page_vs_hangul.py`를 추가해 rhwp의 PI별 시작 페이지와 한글 OLE 자동화의
   `SetPos(0, para, 0)` + `current_page` 결과를 TSV로 대조한다. 다구역 문서는 섹션별 문단 수 누적값으로
   본문 연속 PI에 매핑한다.
2. `src/renderer/typeset.rs`의 `next_will_vpos_reset` 분기에서 페이지를 거의 채운 TAC 표 직후 빈 문단이
   fit 검사 없이 skip되던 문제를 수정한다. 최종 구현은 height fit만 보지 않고 placement와 맞는 vpos fit도
   함께 확인한다.

초기 force-push 전 검토에서 요청했던 세 가지는 최신 head에서 모두 반영됐다.

| 요청 항목 | 확인 결과 |
|-----------|-----------|
| 보고서가 #1659 이전 height-only 설명에 머물러 있음 | `task_m100_1648_report.md`가 #1659 회귀와 height+vpos AND 최종 구현 기준으로 갱신됨 |
| 검증 도구가 `MATCH` 행을 TSV에 쓰지 않음 | `MATCH`도 `sample / MATCH / rhwp_pages / hwp_pages / 0 / ""` 형태로 기록함 |
| `PartialParagraph { start_line > 0 }`에서 원 문단 첫 LINE_SEG를 사용함 | `PartialParagraph`는 `line_segs[start_line]` 기준 vpos를 사용하고, `PartialTable`은 vpos 판정을 보류함 |

## 3. 검토 의견

### 수용 근거

- #1643의 검증 도구는 성공/불일치/오류 샘플을 TSV에서 구분할 수 있고, Windows+한컴+pyhwpx 환경 전제를
  도구 상단과 PR 설명에 명확히 남겼다.
- #1648 수정은 원래 문제였던 "현재 페이지에 들어가는 빈 문단까지 무조건 skip"을 좁게 고친다.
- #1659 회귀 보정으로 음수 줄간격이 있는 페이지에서 height 누적값만 믿고 단독 빈 페이지를 만들던 경로를 막았다.
- `PartialParagraph` continuation의 top vpos 기준이 `start_line`으로 보정되어, 페이지 fragment 시작 줄과
  fit 판정 기준이 어긋날 가능성을 줄였다.
- 최신 PR head 기준 GitHub Actions가 통과했다.
  - CI / Build & Test: success
  - CodeQL: rust, javascript-typescript, python success
  - Render Diff / Canvas visual diff: success
  - WASM Build: skipped (CI 조건상 skip)
- 로컬 검증에서도 대상 테스트, 전체 lib 테스트, fmt, clippy가 통과했다.

### 리스크

- `verify_pi_page_vs_hangul.py`는 Windows + 한컴 OLE 자동화 + pyhwpx 의존 도구라 일반 CI에서 직접 실행하기 어렵다.
  대신 도구 자체의 Python 문법 검사와 PR 작성자의 한컴 환경 검증 결과를 함께 기록한다.
- `PartialTable`은 줄 기준 `vpos`를 직접 산출하지 못해 vpos fit을 보류하고 height fit에 위임한다. 현 구현의
  보수적 fallback으로 수용 가능하지만, 향후 table fragment의 vpos 기준을 별도로 모델링하면 재검토할 수 있다.
- `closingIssuesReferences`가 GitHub API에서 비어 있으므로, merge 후 `#1643`, `#1648`, `#1659` 자동 close 여부를
  반드시 확인해야 한다.

## 4. 로컬 검증

검증 기준 worktree: `/private/tmp/rhwp-pr1652-rereview`

| 항목 | 결과 |
|------|------|
| `python3 -B -m py_compile tools/verify_pi_page_vs_hangul.py` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_1156_rowbreak_fragment_fit` | 통과 (3 passed) |
| `cargo test --test issue_1116` | 통과 (13 passed) |
| `cargo test --test issue_676_trailing_empty_para` | 통과 (3 passed) |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --lib` | 통과 (1988 passed, 0 failed, 7 ignored) |

비고: worktree checkout 중 `pdf-large/hwpx/2026_oss_rst.pdf` LFS pointer 경고가 있었으나, 기존 대용량 파일
상태에서 재현되는 경고이며 이번 PR diff에는 포함되지 않는다.

## 5. 경로 결정

**Route A: original PR merge 후보.**

- PR 작성자는 외부 contributor이고 `maintainerCanModify=true`이다.
- 최신 PR head는 `upstream/devel` 위 단일 contributor commit으로 정리되어 있다.
- 변경 범위가 목적과 일치하고, cherry-pick integration PR로 분리할 사유가 없다.
- collaborator-mediated 외부 PR 경로에 따라 review 문서를 `mydocs/pr/archives/`에 포함하고 contributor PR head에
  문서 전용 커밋으로 push한다.

## 6. 권고

**수용 / approval review 후 merge 권고.**

단, 문서 전용 커밋을 PR head에 push한 뒤 최신 head 기준 GitHub Actions 재실행 결과를 확인해야 한다. 이후
approval review를 제출하고, merge 전 `mergeable` / `mergeStateStatus` / review decision을 다시 확인한다.
