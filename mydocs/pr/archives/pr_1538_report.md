# PR #1538 사전 처리 판단 보고서 — bench 단계별 처리 성능 계측 CLI

- PR: https://github.com/edwardkim/rhwp/pull/1538
- 제목: `Task #1537: bench — 단계별 처리 성능 계측 CLI 서브커맨드`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1537
- 검토일: 2026-06-26
- 검증 head: `a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2`
- 처리 경로: collaborator-mediated 외부 PR 처리 경로
- 문서 경로: `mydocs/pr/archives/pr_1538_review.md`, `mydocs/pr/archives/pr_1538_report.md`

## 1. 사전 판단

**보정/문서 커밋 push 후 최신 CI 확인을 조건으로 수용 가능.**

PR #1538은 `rhwp bench` CLI를 추가해 HWP/HWPX 처리 성능을 parse/layout/render/serialize 단계별
median(ms)로 측정한다. 변경은 `src/diagnostics/bench.rs`, CLI dispatch/help, `cli_commands.md`,
contributor 성능 보고서에 한정된다.

초기 검토에서 자동화 신뢰성 문제가 2건 발견되어 Request changes를 제출했다.

- render 단계 실패가 성공처럼 숨겨질 수 있음.
- 하나 이상의 파일 처리 실패가 있어도 프로세스 exit code가 0으로 남음.

contributor 수정 커밋 `2ca6a3463dad1a109286d44ac9ac7445bc23ad1c`에서 두 항목은 반영됐다. 이후 추가
검토에서 TSV 쓰기 실패도 exit 0으로 남는 같은 계열 문제가 발견되어, collaborator 보정 커밋
`a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2`로 `write_tsv` 실패를 failure count에 포함했다.

현재 로컬 검증 기준 blocking finding은 없다. 다만 보정/문서 커밋 push 후 최신 PR head 기준 GitHub Actions와
review decision 갱신이 필요하다.

## 2. PR 상태

문서 작성 전 GitHub 확인 기준:

| 항목 | 값 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| merge state | `CLEAN` |
| review decision | `CHANGES_REQUESTED` |
| labels | `enhancement`, `performance` |
| milestone | 없음 |
| assignee | `planet6897` |
| head branch | `planet6897:pr-task1537` |
| maintainerCanModify | true |
| issue #1537 | open, milestone 없음 |

GitHub Actions, contributor 수정 커밋 `2ca6a3463dad1a109286d44ac9ac7445bc23ad1c` 기준:

| 체크 | 결과 |
|---|---|
| Build & Test | success |
| Analyze (rust) | success |
| Analyze (javascript-typescript) | success |
| Analyze (python) | success |
| Canvas visual diff | success |
| CodeQL | success |
| WASM Build | skipped |

주의:

- 이 보고서는 pre-merge 판단 보고서다. merge SHA, 실제 merge 시각, issue close 완료 여부를 단정하지 않는다.
- 보정/문서 커밋 push 후 위 상태는 최신 head 기준으로 다시 확인해야 한다.

## 3. 변경 검토

| 파일 | 변경 | 판단 |
|---|---|---|
| `src/diagnostics/bench.rs` | `bench` 구현. 파일/배치 입력, 반복 median, TSV 산출, 실패 exit 처리 | 보정 후 타당 |
| `src/diagnostics/mod.rs` | `bench` 모듈 export | 타당 |
| `src/main.rs` | `bench` dispatch/help 추가 | 타당 |
| `mydocs/manual/cli_commands.md` | CLI 매뉴얼에 `bench` 추가 | 타당 |
| `mydocs/report/task_m100_1537_report.md` | contributor 성능 측정 보고서 | 한계 명시되어 수용 가능 |

핵심 판단:

- `bench`는 새로운 진단 CLI이며 renderer/serializer 동작 자체를 바꾸지 않는다.
- 측정값은 머신/빌드 의존값으로 문서화되어 있고, 같은 환경에서의 상대 비교·회귀 추적 지표로 설명된다.
- render 실패, 파일 실패, TSV 쓰기 실패가 모두 non-zero exit로 이어지도록 보정됐다.

## 4. Contributor credit와 커밋 출처

원본 contributor 커밋:

| source commit | author | 내용 |
|---|---|---|
| `75c7fd75a67aac99ea1547a34129c984b1819fa3` | Jaeook Ryu `<jaeook.ryu@gmail.com>` | `bench` CLI 본 구현, help/manual/report |
| `2ca6a3463dad1a109286d44ac9ac7445bc23ad1c` | Jaeook Ryu `<jaeook.ryu@gmail.com>` | Request changes 반영: render 실패 전파, 파일 실패 시 exit 1 |

`75c7fd75`에는 다음 co-author trailer가 있다.

```text
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

collaborator 보정 커밋:

| commit | author | 내용 |
|---|---|---|
| `a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2` | postmelee `<meleeisdeveloping@gmail.com>` | TSV 쓰기 실패도 exit 1로 처리 |

contributor 커밋은 rewrite하지 않았고, collaborator 변경은 후속 커밋으로 분리했다.

## 5. 로컬 검증

검증 head:

```text
a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2
```

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo build --release --bin rhwp` | 통과 |
| `./target/release/rhwp bench samples/hwpx_sample2.hwpx samples/task-001.hwp -n 1 --tsv output/poc/pr1538/bench-collab.tsv` | 통과, exit 0 |
| `./target/release/rhwp bench /no/such/file -n 1` | 실패 입력을 보고하고 exit 1 |
| `./target/release/rhwp bench samples/task-001.hwp -n 1 --tsv /dev/null/foo.tsv` | TSV 쓰기 실패를 보고하고 exit 1 |

이전 기본 구현 검증:

| 명령 | 결과 |
|---|---|
| `cargo test --release --lib` | 통과, 1937 passed / 0 failed / 6 ignored |

## 6. 시각 검증

별도 시각 검증 산출물은 만들지 않았다. 이 PR은 진단 CLI 추가이며, 렌더러 출력이나 layout 알고리즘을 직접
수정하지 않는다. render 단계는 기존 `DocumentCore::render_page_svg_native` 호출 시간을 측정할 뿐이다.

GitHub의 Canvas visual diff는 contributor 수정 커밋 기준 success였다. 보정/문서 커밋 push 후 최신 check
상태를 다시 확인한다.

## 7. merge 전 조건

1. 보정 커밋과 문서 커밋을 `planet6897/rhwp:pr-task1537`에 push한다.
2. PR diff에 `mydocs/pr/archives/pr_1538_review.md`와 `mydocs/pr/archives/pr_1538_report.md`가 포함되는지 확인한다.
3. 최신 PR head 기준 GitHub Actions를 확인한다. 코드 보정 커밋이 있으므로 최신 relevant checks가 필요하다.
4. 기존 `CHANGES_REQUESTED` review decision은 작업지시자 승인 후 새 review로 갱신한다.
5. merge 전 최신 `mergeable`, `mergeStateStatus`, head SHA를 다시 확인한다.
6. merge 후 #1537 상태를 확인한다. GitHub API의 `closingIssuesReferences`가 비어 있었으므로 자동 close 실패 가능성을 고려한다.
7. issue close 또는 수동 close comment는 작업지시자 승인 전에는 수행하지 않는다.

## 8. 권장 처리

권고: **최신 CI 통과와 review decision 갱신을 조건으로 Approve 가능.**

현재 로컬 검증 기준 blocking finding은 없다. 보정 커밋은 자동화에서 TSV 산출 실패를 성공으로 오인하는 문제를
해소한다. 문서 커밋 push 후에는 최신 CI 상태를 확인하고, 작업지시자 승인에 따라 GitHub review를
`Approve`로 갱신하면 된다.
