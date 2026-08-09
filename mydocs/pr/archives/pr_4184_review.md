---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4184 검토 - PII 읽기 전용 검사 발견 경로

## 절차와 대상

~~~text
base route: maintainer_general
modifiers: intake_and_review, local_validation, multi_pr_update_branch,
  review_only_fast_pass
loaded documents: pr_review_workflow.md, pr_review/README.md,
  maintainer_general.md, intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, review_only_fast_pass.md
~~~

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4184](https://github.com/edwardkim/rhwp/pull/4184) / @kevin9327 |
| 관련 이슈 | [#4112](https://github.com/edwardkim/rhwp/issues/4112) |
| contributor 원 head | `65db7bc088a465eb8c887150974d87212a722bcd` |
| base | `devel` / 검토 시점 `e64c85312` |
| 규모 | 1개 파일, +1/-0, 1 commit |
| 가시성 검토 브랜치 | `review/kevin9327-4184-20260808` |
| 시각 검증 | 비대상. 지식 지도 Markdown 한 행만 변경한다. |

이슈 #4112는 처음에 `rhwp-agent pii-scan`을 본 CLI `inspect pii`로 승격하려 했으나, 작성자가
`edit redact --dry-run --no-raw --json`이 같은 읽기 전용 PII 검사 계약을 이미 제공한다고 실측해
방향을 바꿨다. 이슈 댓글에는 새 표면이 봉투·플래그·문서를 이중화한다는 판단과 남은 공백이
발견 가능성이라는 근거가 기록돼 있다. PR은 지식 지도의 보안 결정표에 기존 명령과 판정 필드,
레시피 3·보안 소비자 가이드 링크를 한 행으로 추가한다.

## 발견 사항과 메인터너 보정

CLI의 `--dry-run --no-raw`는 MCP `hwp_redact`에서 자동 기본값이 아니다. MCP schema의 `dryRun`과
`noRaw`는 선택 boolean이며 지정하지 않으면 false다. 따라서 도구 이름만 병기하면 읽기 전용 호출이
사용법 오류로 끝나거나 원문 개인정보가 `findings[].raw`에 포함될 수 있다. 같은 표 행에
`dryRun:true`, `noRaw:true`를 명시해 CLI와 MCP의 안전한 호출 조건을 정확히 대응시켰다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| reviewer 지정 | GitHub requested reviewer에 `edwardkim`이 반영됐다. |
| 재판정 근거 | #4112 댓글에서 승격 불필요·발견 경로 명문화로 범위를 바꾼 사실을 확인했다. |
| CLI·MCP 계약 | `src/main.rs`와 v0.8.2 `capabilities --mcp`의 `hwp_redact` schema에서 `dryRun`, `noRaw`가 별도 boolean 인자이며 required는 `path`뿐임을 확인했다. |
| 실제 dry-run | #4183 검토의 v0.8.2 실행에서 `findingCount=3`, `noRaw=true`, 파일 무변경을 확인했다. |
| 링크 | 레시피 3과 `agent_security/consumer_guide.md` 대상 파일이 존재한다. |
| current-base 병합 | `git merge-tree --write-tree upstream/devel local/pr4184`가 충돌 없이 tree `8372d3e9c5bb5a96c5cda3557729506124f717ad`를 계산했다. |
| Cargo·시각 검증 | 문서 한 행 변경이므로 Cargo, WASM, Skia, 시각 검증을 생략했다. |

## GitHub Actions와 수용 판단

원 head `65db7bc08`의 [CI 31220567156](https://github.com/edwardkim/rhwp/actions/runs/31220567156)과
[CodeQL 31220566931](https://github.com/edwardkim/rhwp/actions/runs/31220566931)은 동일 PR·branch·head의
`pull_request` event에 귀속되어 성공했다. PR 전체가 `mydocs/` 범위여서 preflight와 Build & Test
aggregate가 성공하고 heavy worker가 skip된 것은 review-only fast-pass B 경로의 정상 결과다.

**메인터너 보정 포함 수용.** 한 줄 보정과 review 기록을 한 번에 push한 뒤 최신 head의 preflight,
Build & Test aggregate, mergeable 상태를 다시 확인하고 작업지시자 승인에 따라 병합한다. current-base
Update branch와 broad CI는 수행하지 않는다.

## 병합 결과

메인터너 보정과 검토 기록을 포함한 최종 head
`5d7c5ea9ce1da42a650a4e86cd12af4faeefcb9a`에서
[CI 31242060882](https://github.com/edwardkim/rhwp/actions/runs/31242060882)와
[CodeQL 31242060805](https://github.com/edwardkim/rhwp/actions/runs/31242060805)가 성공했다.
PR은 `mergeable=true`, `mergeable_state=clean`을 재확인한 뒤 작업지시자 승인에 따라 admin merge했다.
merge commit은 `9831159eeba63ff2d0661ef9ebf6ca23e07aabc6`이며, 이슈 #4112는 closing keyword
처리로 자동 종료됐다.
