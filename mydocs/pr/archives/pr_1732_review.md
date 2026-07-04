# PR #1732 리뷰 문서

## 개요

| 항목 | 내용 |
|------|------|
| PR | #1732 `Task #1664: 최종 measurement 기록을 mydocs에 반영` |
| 작성자 | @postmelee |
| 경로 | collaborator self-merge 후보 |
| base | `devel` |
| head | `postmelee:task1664-docs-final-measurement` |
| 검토 대상 head | `70c1e8b42406e7c792b5873ec35cc8392261f7cf` (문서 작성 시점 참고값) |
| 관련 이슈 | #1664, #1668 |
| labels | `documentation`, `ci`, `performance` |
| milestone | `v1.0.0` |

이 PR은 외부 contributor PR이 아니라 collaborator 본인 PR이다. 따라서
`mydocs/manual/pr_review_workflow.md` 8장 collaborator self-merge 후보 예외 경로를 적용한다.

## 변경 범위

PR #1732는 #1702 merge 이후 GitHub issue comment에 남긴 최종 measurement를 `mydocs/` 장기 기록으로
이관하는 문서 전용 PR이다.

변경 파일:

- `mydocs/orders/20260701.md`
- `mydocs/plans/task_m100_1664_v2.md`
- `mydocs/report/task_m100_1664_measurement.md`
- `mydocs/report/task_m100_1664_report.md`
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`

범위 밖 항목:

- `.github/workflows/ci.yml` 변경 없음
- `Cargo.toml` profile 변경 없음
- `tests/`, `tests/golden_svg/`, sample fixture 변경 없음
- GitHub issue close 없음

## 원천 기록 대조

문서가 참조하는 전제와 측정값을 GitHub 원천 기록과 대조했다.

- PR #1701: merge 완료, merge commit `d7f2afe7297e699904771cdf6c026c281cc189d7`
- PR #1702: merge 완료, merge commit `a555d318f097de5cb2ce568bdcd043824c11906d`
- #1664 상세 후속 관측: <https://github.com/edwardkim/rhwp/issues/1664#issuecomment-4853263686>
- #1668 부모 롤업: <https://github.com/edwardkim/rhwp/issues/1668#issuecomment-4853268643>

대조한 핵심 값:

- closed/merged `refs/pull/*` cache 총 21개 삭제 기록
- 최종 cache 총량 `7,154,189,707` bytes, 약 6.66 GiB / 7.15 GB
- cleanup 직후 `devel` rerun `28505355210` attempt 2:
  - head `5e3b1ec652fda14a74af7cf9afd77962e3bb7903`
  - `Build & Test` 22m53s
  - fallback restore 후 `Linux-cargo-6a1af...` save success
  - save size `1,637,296,893` B
- 후속 `devel` run `28507949075`:
  - head `150ca316ee557d6bf95928302166e037d7467b03`
  - `Build & Test` 18m02s
  - exact-hit restore
  - exact hit으로 save skipped
- `Cache reservation failed`, `Failed to save`, `##[error]` 재현 없음

## 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| PR metadata | 통과 | `documentation`, `ci`, `performance`, `v1.0.0`, assignee @postmelee |
| PR head fetch | 통과 | `upstream pull/1732/head:local/pr1732` |
| 변경 범위 | 통과 | `mydocs/**` 5개 파일, +198/-27 |
| `git diff --check upstream/devel...local/pr1732` | 통과 | whitespace 문제 없음 |
| 단일 부모 커밋 | 통과 | 원 변경 커밋 `70c1e8b4`는 single-parent commit |
| 원천 issue comment 대조 | 통과 | #1664/#1668 후속 관측값과 문서값 일치 |
| Actions run 대조 | 통과 | run `28505355210` attempt 2, run `28507949075` job 시간과 head 일치 |
| 시각/렌더 검증 | 해당 없음 | 문서 전용 PR이며 렌더 출력 변경 없음 |

문서 작성 시점 참고 GitHub Actions:

- `CI preflight`: success
- `CodeQL preflight`: success
- `Render Diff preflight`: success
- `Build & Test`: skipped
- `Analyze`: skipped
- `Canvas visual diff`: skipped
- `WASM Build`: skipped

후속 커밋도 `mydocs/**` 문서만 추가하면 문서 전용 fast-pass 조건에 부합한다. merge 전에는 최신 head 기준
check 상태를 다시 확인해야 한다.

## 리뷰 결과

Blocking finding 없음.

PR #1732는 #1702 merge 후 관측값을 이슈 코멘트에만 두지 않고 장기 보관 문서에 반영한다는 목적에 맞게
`mydocs/**` 범위를 유지하고 있다. #1664 원천 측정 문서, #1668 부모 추적 문서, #1664 최종 보고서의 역할도
구분되어 있으며, `devel` push run과 PR checks 표본을 분리해 P50/P90 보류 사유를 남긴 점도 적절하다.

## 리스크 / 후속 확인

- #1664와 #1668은 현재 open 상태다. PR #1732는 `Refs` 성격이며 issue close를 포함하지 않는다.
- #1668은 tracking/RFC 이슈이므로 이 PR merge만으로 close하지 않는다.
- #1664 close 여부도 작업지시자 승인 후 별도로 판단해야 한다.
- 리뷰 문서 커밋이 push되면 PR head SHA가 바뀐다. merge 전 최신 head, mergeability, checks 상태를 다시
  확인해야 한다.

## 판단

수용 가능.

조건:

- review 문서 커밋 후 PR diff에 `mydocs/pr/archives/pr_1732_review.md`와
  `mydocs/pr/archives/pr_1732_report.md` 포함 확인
- 최신 PR head 기준 GitHub Actions fast-pass 또는 required checks 통과 확인
- merge 직전 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 merge 승인
