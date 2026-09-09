# Stage 3 완료 보고 — Task M100 #6791 공개 검증 명령 실증

- Issue: [#6791](https://github.com/edwardkim/rhwp/issues/6791)
- 출처: baba9811의 [PR #6786](https://github.com/edwardkim/rhwp/pull/6786) 본문 「외부 기여자 검증 절차 확인」
- 계획: [수행계획](../plans/task_m100_6791.md), [구현계획](../plans/task_m100_6791_impl.md)
- 승인: 사용자 “진행해줘”, 기록 commit `16c41bc87`
- 검증 문서 commit: `b5a33bfe3b85b9b0f4ebe119a1d1addeb8c1e43b` (Stage 2 완료)
- 상태: 실검증·로컬 PR 준비 완료. 원격 조치 승인 대기

## 검증 구성

검증 전용 source worktree를 `/private/tmp/rhwp-6791-final-source`에 위 commit으로 새로 만들었다.
시작할 때 `git status --porcelain`은 비어 있었고 generated suite·manifest가 없었다. 이전 기준선
worktree와 #6786 검토 worktree는 재사용하거나 수정하지 않았다.

해당 commit의 `CONTRIBUTING.md`에서 다음 두 bash 블록을 문자열 변경 없이 추출하고 한 Bash 세션에서
순차 실행했다. 실행 시 stdout/stderr를 로그 파일로 보내는 외부 wrapper만 덧붙였다.

1. `1. 검증 worktree 준비와 포맷 검사`: source SHA 보존, clean 상태 확인, 새 detached worktree 생성,
   `node scripts/rust-test-suite-manifest.mjs --prepare`, `cargo fmt --all -- --check`.
2. `4. manifest 확인과 검증한 원본 제출` 중 **manifest 확인 블록**: 검증 SHA 일치,
   `node scripts/rust-test-suite-manifest.mjs --check`, diff·clean 상태 확인.

공개 명령이 직접 만든 review worktree는 `/private/tmp/rhwp-6791-final-source-rust-review`다.
source·review의 디렉터리, 변수와 SHA 처리 로직을 수정하지 않았다. push 블록과 전체 Rust lint·build·회귀
블록은 실행하지 않았다. 이 작업은 문서 절차 검증이며, 그 제품 검증까지 통과한 것으로 집계하지 않는다.

환경은 macOS, Bash, Node `v24.15.0`, Cargo/Rust toolchain `1.93.1`, Python `3.14.4`다.

## 실측 결과

| 검사 | 결과 |
| --- | --- |
| 공개 worktree 준비·prepare·fmt 블록 | exit 0 |
| 공개 manifest·SHA·diff·clean 확인 블록 | exit 0 |
| manifest | 1169 sources / 4925 static test attrs / 28 suites + 20 exceptions / 48 of 48 integration targets |
| tracked `.rs` 전체 + root Cargo.toml·Cargo.lock | 2,206개 파일 전후 SHA-256 동일 |
| source worktree 원본 | 2,206개 파일 불변, generated suite·manifest 없음 |
| 생성물 | review worktree의 harness 28개와 manifest 모두 `git check-ignore`로 확인 |
| source·review 상태 | 둘 다 porcelain 출력 없음, review staged diff 없음 |

파일 경로를 정렬해 `경로<TAB>SHA-256<LF>`로 직렬화한 입력 원장의 전후 SHA-256은 모두 다음과 같다.

```text
61f54676ba2bb1b128c055fefab1dcc1e3bdd469367c298290abd629b39fb541
```

로그·입력별 hash·집계는 `/private/tmp/rhwp-6791-final-evidence/`의 `prepare-and-fmt.log`,
`manifest-and-clean.log`, `inputs-before.json`, `summary.json`에 보존했다. 이 임시 경로는 세션 증적 위치이며
공개 기여자의 필수 환경이 아니다. 검증 중 보조 집계의 첫 `git check-ignore --quiet` 호출은 여러 경로를
함께 전달해 exit 128이었다. `--quiet`를 제거하고 반환 경로 29개를 예상 집합과 비교해 검증했다. 공개
준비·fmt·manifest 명령의 실패가 아니므로 그 실행을 반복하거나 PASS로 덮어쓰지 않았다.

## 문서와 최신 base 정합

- 공개 2파일과 기존 계획·단계 기록 4개를 지정한 상대 링크 검사: exit 0.
- Stage 2에서 내부·교차 anchor 21개와 Rust 절 bash 블록 10개의 존재·구문을 확인했고, 이후 공개 2파일은
  변경하지 않았다. 최종 문서·보고 링크도 commit 전에 검사한다.
- `git diff --check upstream/devel...HEAD`: exit 0.
- 현재 upstream/devel은 `016fe3ceed904633e74e70127a4cceaa1f18a756`으로 진행됐지만 작업 시작 base
  `ff1ce007b428547da74e0d6b7e9a196592c60ff6` 이후 두 공개 파일의 upstream 변경은 없었다.
- `git merge-tree --write-tree upstream/devel HEAD`: exit 0, 충돌 없음. 검사 당시 로컬 HEAD는 `16c41bc87`이며
  통합 tree는 `28ac31f421a7b72e47db15d0d52df08f8be7ecd7`이다. 실제 checkout/branch merge는 하지 않았다.
- 통합 tree와 검증된 로컬 HEAD 사이의 두 공개 파일 diff도 없었다. 최신 devel 전체 제품을 로컬에서
  다시 빌드·회귀 검증했다는 뜻은 아니다. PR 생성 뒤 최신 required checks는 별도 확인해야 한다.

## 완료 범위와 다음 조치

문서·템플릿 수정, 공개 준비·fmt·manifest 실증, 원본 불변·생성물 미제출 증명과 로컬 PR 초안을 완료했다.
[최종 보고서](../report/task_m100_6791_report.md)에 PR 본문과 push/생성 명령을 준비했다.

현재 classifier v7은 PR 템플릿의 `.github/` 경로 때문에 이 변경을 `full / fail-closed:workflow-contract`로
분류한다. 정책은 변경하지 않았으며 실제 GitHub CI는 PR 생성 뒤 실행·판정할 대상이다.
원격 push·PR 생성·merge·이슈 close와 #6786에 대한 원격 변경은 수행하지 않았다.
