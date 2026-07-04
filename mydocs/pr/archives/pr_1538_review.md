# PR #1538 검토 기록 — bench 단계별 처리 성능 계측 CLI

- PR: https://github.com/edwardkim/rhwp/pull/1538
- 제목: `Task #1537: bench — 단계별 처리 성능 계측 CLI 서브커맨드`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1537
- 작성일: 2026-06-26
- 처리 경로: collaborator-mediated 외부 PR 처리 경로. `maintainerCanModify=true`이므로 review 문서를
  PR head의 `mydocs/pr/archives/`에 직접 포함한다.
- base/head: `edwardkim/rhwp:devel` <- `planet6897/rhwp:pr-task1537`
- 문서 작성 시점 검증 head: `a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2`
- 규모: 5 files, +373 / -0 (문서 커밋 전 기준)

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 판단 전 최신 상태를 다시 확인한다.

## 1. 목적

PR #1538은 대용량 HWP/HWPX 처리 성능을 재현 가능한 수치로 계량하기 위해 `rhwp bench` 서브커맨드를
추가한다.

대상 명령:

```text
rhwp bench <파일...> | --batch <폴더> [-n <반복수>] [--tsv <출력.tsv>]
```

계측 단계:

- `parse`: 바이트 -> Document IR (`parse_document`)
- `layout`: `DocumentCore::from_bytes` 로드 비용에서 parse 비용을 뺀 근사값
- `render`: 전 페이지 `render_page_svg_native`
- `serialize`: `serialize_hwpx` HWPX 바이트 생성

이 문서는 코드 리뷰에서 확인한 축, Request changes와 반영 상태, collaborator 보정 커밋, 로컬 검증,
merge 전 조건을 기록한다.

## 2. 현재 PR 메타

| 항목 | 내용 |
|---|---|
| state | open |
| draft | false |
| mergeable | 문서 작성 전 확인 기준 `MERGEABLE` |
| merge state | 문서 작성 전 확인 기준 `CLEAN` |
| review decision | 기존 Request changes 때문에 `CHANGES_REQUESTED` |
| base | `devel` |
| head branch | `planet6897:pr-task1537` |
| labels | `enhancement`, `performance` |
| milestone | 없음 |
| PR assignee | `planet6897` |
| issue #1537 state | open |

GitHub Actions, contributor 수정 커밋 `2ca6a3463dad1a109286d44ac9ac7445bc23ad1c` 기준 확인:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| WASM Build | skipped |

주의:

- collaborator 보정 커밋 `a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2`와 review 문서 커밋 push 후에는
  GitHub Actions가 다시 실행될 수 있다.
- 기존 Request changes review decision은 자동으로 해소되지 않으므로, merge 전 새 GitHub review로 갱신해야 한다.

## 3. 커밋별 검토 범위

| 커밋 | 작성자 | 내용 | 주요 검토 축 |
|---|---|---|---|
| `75c7fd75` | Jaeook Ryu | `bench` CLI 본 구현, help/manual 동기화, 성능 보고서 추가 | CLI 인자 처리, 단계별 계측 경계, TSV 산출, 문서 정합 |
| `2ca6a346` | Jaeook Ryu | Request changes 반영: render 실패 전파, 파일 처리 실패 시 exit 1 | 기존 리뷰 지적 해소 여부, 자동화 신뢰성 |
| `a6469c8d` | postmelee | TSV 쓰기 실패도 exit 1로 처리하는 collaborator 보정 | 산출물 실패의 non-zero 전파, contributor commit 보존 |

원본 contributor 구현 커밋 `75c7fd75`에는 다음 trailer가 포함되어 있다.

```text
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```

collaborator 보정 커밋은 contributor 커밋을 rewrite하지 않고 후속 커밋으로만 추가했다.

## 4. 코드 리뷰 체크리스트

### 4.1 CLI 인자와 입력 수집

- `<파일...>`와 `--batch <폴더>`를 모두 지원하는지 확인한다.
- `--batch`는 `.hwp`, `.hwpx`를 재귀 수집하고 정렬하는지 확인한다.
- `-n 0`은 최소 1회 반복으로 보정되는지 확인한다.
- 입력이 없으면 사용법만 출력하고 종료한다. 현재 이 경로는 exit 0이며, 도움말성 호출로 볼 수 있어 blocker로 보지 않는다.

### 4.2 단계별 계측

- parse 단계는 `parse_document(&data)`를 별도 측정한다.
- load 단계는 `DocumentCore::from_bytes(&data)`를 측정하고, layout은 `load - parse` 근사값으로 계산한다.
- layout 근사가 음수일 때 0으로 clamp하는 한계를 문서와 보고서에 명시한다.
- render 단계는 전 페이지 `render_page_svg_native`를 호출한다.
- serialize 단계는 `serialize_hwpx(core.document())`의 HWPX 바이트 생성 비용을 측정한다.

### 4.3 실패 전파와 자동화 신뢰성

초기 review에서 다음 두 항목을 Request changes로 남겼다.

1. `render_page_svg_native(p)`의 `Result`를 버려 render 실패가 성공처럼 숨겨지는 문제.
2. 파일 처리 실패가 있어도 프로세스 exit code가 0으로 남는 문제.

contributor 수정 커밋 `2ca6a346`에서 두 항목은 반영됐다.

추가 검토 중 다음 항목을 발견했고 collaborator 보정 커밋으로 반영했다.

3. `--tsv` 쓰기 실패가 있어도 exit code가 0으로 남는 문제.

보정 후 `write_tsv` 실패도 `failures += 1`로 처리되어 마지막에 exit 1로 이어진다.

### 4.4 문서와 보고서

- `src/main.rs`의 help 문자열에 `bench`가 추가됐다.
- `mydocs/manual/cli_commands.md`에 `bench` 명령이 추가됐다.
- contributor 보고서 `mydocs/report/task_m100_1537_report.md`는 측정 머신/빌드 의존성과 상대 비교 지표라는 한계를 명시한다.
- `layout = load - parse` 근사, 메모리 미측정, 단일 머신이라는 한계를 기록했다.

## 5. 로컬 검증 기록

검증 head:

```text
a6469c8d67e72bd7c83b1fea298ccc6cf11f50d2
```

실행한 로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo build --release --bin rhwp` | 통과 |
| `./target/release/rhwp bench samples/hwpx_sample2.hwpx samples/task-001.hwp -n 1 --tsv output/poc/pr1538/bench-collab.tsv` | 통과, exit 0 |
| `./target/release/rhwp bench /no/such/file -n 1` | 실패 입력을 보고하고 exit 1 |
| `./target/release/rhwp bench samples/task-001.hwp -n 1 --tsv /dev/null/foo.tsv` | TSV 쓰기 실패를 보고하고 exit 1 |

이전 head `6f9bf7ffa885ceb4afbfb972c8ca44e880e8876a` 기준 추가 검증:

| 명령 | 결과 |
|---|---|
| `cargo test --release --lib` | 통과, 1937 passed / 0 failed / 6 ignored |

`cargo test --release --lib`는 contributor 수정 전 기본 구현 검증에서 통과했다. 이후 변경은 `bench.rs`의
실패 전파와 exit code 처리에 한정되며, 최신 head에서는 `fmt`, `clippy`, release build, 세 가지 CLI 스모크로
재확인했다.

## 6. visual/rendering 영향

이 PR은 renderer 결과의 시각 정합성 변경이 아니라 CLI 진단 도구 추가다. `render` 단계에서 기존
`DocumentCore::render_page_svg_native`를 호출해 시간을 측정하지만, 렌더러 출력 로직 자체를 변경하지 않는다.

따라서 별도 before/after 시각 산출물은 만들지 않았다. GitHub의 Canvas visual diff는 contributor 수정 커밋
기준 pass였다. 보정 커밋 후에는 GitHub Actions 최신 상태를 다시 확인해야 한다.

## 7. 위험 분류

### Blocking 후보

- render 실패 또는 serialize 실패가 성공처럼 숨겨지는 경우.
- 파일 처리 실패 또는 TSV 산출 실패가 자동화에서 exit 0으로 처리되는 경우.
- `bench` 측정값이 절대 성능 비교값처럼 문서화되는 경우.

### 검토 결과

- render 실패는 `?`로 파일 처리 실패에 전파된다.
- 파일 처리 실패와 TSV 쓰기 실패는 모두 failure count에 반영되어 exit 1로 종료된다.
- 문서와 보고서에는 측정값이 머신/빌드 의존이며 같은 환경의 상대 비교·회귀 추적 지표임을 명시했다.

### Non-blocking 후속

- layout 단계를 정확히 분리하려면 `DocumentCore` 또는 layout engine 내부 계측 훅이 필요하다.
- peak RSS 등 메모리 계측은 후속 범위다.
- CI 회귀 게이트로 고정할지는 별도 정책 판단이 필요하다.

## 8. 리뷰 문서 push 계획

Route A, original PR merge 후보로 유지한다.

이번 collaborator push는 두 커밋으로 분리한다.

1. code 보정 커밋: `bench: fail on TSV write errors`
2. review 문서 커밋: `docs(pr): record PR #1538 review`

문서 커밋에는 다음 파일만 포함한다.

```text
mydocs/pr/archives/pr_1538_review.md
mydocs/pr/archives/pr_1538_report.md
```

push 대상:

```text
planet6897/rhwp:pr-task1537
```

push 후 확인:

1. PR head SHA가 보정 커밋과 문서 커밋으로 갱신된다.
2. PR diff에 `mydocs/pr/archives/pr_1538_review.md`와 `mydocs/pr/archives/pr_1538_report.md`가 포함된다.
3. contributor 원 커밋 2건은 rewrite되지 않는다.
4. 최신 GitHub Actions 상태를 확인한다.
5. 작업지시자 승인 후 기존 `CHANGES_REQUESTED`를 새 review로 갱신한다.

## 9. 현재 결론

현재 로컬 검증 기준으로 blocking finding은 없다.

권고: **보정/문서 커밋 push 후 최신 CI 확인, 이후 작업지시자 승인에 따라 Approve 가능.**

merge, issue close, final merge comment는 별도 작업지시자 승인 전에는 수행하지 않는다.
