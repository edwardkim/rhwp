# PR #8291 리뷰 — 종료 후 output과 review debug 자동 정리

## 최종 판정

**승인.** 지침 간 정리 범위를 일치시키고 실제 삭제·허용 규칙과 문서 전용 CI를 확인했다.
검증한 원 head는 `38a8e7768d2bae159f0b089911e59abc52426c02`다.
작업지시자가 요청한 review·오늘할일을 같은 브랜치의 trailing commit으로 추가한다.
해당 새 head의 CI·정책·병합 가능 상태 확인 후 일반 merge하고 후속 정리를 완료한다.

## 접수와 범위

- [PR #8291](https://github.com/edwardkim/rhwp/pull/8291), 작성자 `jangster77`, base `devel`.
- 브랜치: `review/integration-20261026-dependabot-cleanup-rules`.
- 고정 base: `f770ef5024e014284fead45e839264beada35802`.
- 원 commit: `1f71372cf`의 자동 정리 명확화와 `38a8e7768`의 debug 유휴 셸 판정 보완.
- collaborator self-review이며 별도 self reviewer는 지정하지 않았다. 연결된 종료 대상 이슈는 없다.
- 문서 5개만 변경했다. 제품 코드·Rust/JS 검사·fixture·workflow·원본·PDF 변경은 없다.

후속 처리에서 유휴 셸의 cwd 참조만으로 output 삭제를 보류하고 임시 산출물을 통째로
보존했던 실행을 바로잡았다. 작업지시자의 명시 지시를 다음 문서의 같은 계약으로 반영했다.

| 문서 | 반영한 내용 |
| --- | --- |
| [AGENTS.md](../../../AGENTS.md), [CLAUDE.md](../../../CLAUDE.md) | 별도 지시·재승인 없이 자동 정리, 백업 금지와 정본 진입점 |
| [post_merge.md](../../manual/pr_review/post_merge.md#77-branch-worktree-검토-전용-target-정리) | output 전체 삭제·유휴 셸 판정·정리 로그·종료 후 debug 전체 삭제의 정본 |
| [local_validation.md](../../manual/pr_review/local_validation.md) | 검증 중 target 재사용과 종료 후 debug 삭제의 시점 구분 |
| [dev_environment_guide.md](../../manual/dev_environment_guide.md#디스크-공간이-부족할-때) | 공유 target 보존 문구의 debug 정리 예외 |

정식 검증 증거는 작업 중 정식 경로에 반영한다. 종료 시 임시 내용의 추가 복사·압축·이관을
만들거나 “보존하지 말고 제거”라는 추가 지시를 기다리지 않는다. 실제로 진행 중인 다른 작업과
명시적인 보존 지시는 구체적인 근거로 판단한다. 유휴 셸의 cwd만으로 정리를 보류하지 않는다.

`target/pr-review/debug`는 공유 target을 사용하는 빌드·테스트·실행 파일의 사용 종료 후
디렉터리 전체를 정리한다. 공유 target 루트와 다른 profile의 재사용 범위를 명시했다.

## 검증과 실제 정리 결과

- 원 head의 `git diff --check`가 통과했다.
- 변경 Markdown 5개 상대 링크 검사에서 오류가 없었다.
- 고정 base/head의 `git merge-tree --write-tree`가 exit 0으로
  `c8be4b143ce92ca6b302ae82006659dee0c36f73`을 생성했다. 문서 기록을 위해 rebase하지 않았다.
- 로컬 `output/` 전체를 추가 백업 없이 삭제하고 디렉터리 부재를 확인했다.
- `target/pr-review/debug`는 빌드·테스트·열린 파일 사용 없이 유휴 셸 cwd만 확인됐다.
  디렉터리 전체를 백업 없이 삭제했고 여유 공간은 139,927,187,456바이트(약 130.3GiB) 증가했다.
  `release`, `release-test`, `wasm32-unknown-unknown`은 유지됐다.
- 삭제 경로·용량·완료 여부와 검사 로그는
  `/home/tsjang/rhwp-pr-evidence/dependabot-20261026`에 기록했다. 삭제한 임시 내용의 백업은 만들지 않았다.

조판 원칙·독립 Print 비교·새 Native/fresh WASM 출력은 비해당이다. 문서 정책 변경이며 렌더링
소비 값·원본·골든을 변경하지 않았다. 이 PR을 위해 Rust 회귀·빌드를 재실행하지 않았다.
별도 review_impl은 만들지 않으며 정책·검증·후속 계획을 이 문서에 함께 기록한다.

## 저장소 밖의 허용 규칙 운영

이 항목은 저장소 diff에 포함된 설정 파일이 아닌, 승인받아 수행한 호스트별 운영 기록이다.

- 로컬 `ubuntu-ted2`, `ubuntu-ted`, `ubuntu-ted3`의
  `~/.codex/rules/rhwp-cleanup.rules`에 output·review debug의 절대 경로 규칙을 생성했다.
- `win10-ted`의 `C:\Users\admin\.codex\rules\rhwp-cleanup.rules`에는
  `C:\Users\admin\rhwp` 아래 대응 경로와 PowerShell 호출 규칙을 생성했다.
- Windows 실제 runtime
  `C:\Users\admin\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe`를
  launcher에 추가하고 해당 절대 경로의 명령 16종과 전체 규칙 조합의 allow 판정을 확인했다.
- 저장소 루트·target 루트·release·samples·debug-copy 대조 명령은 새 규칙에 매칭되지 않았다.
  prefix 규칙의 명령 판정과 실제 파일시스템 접근 제어는 구분한다. 추가 후행 인자까지
  매칭될 수 있으므로 정리 실행 시 정확한 단일 대상 경로를 사용한다.
- 기존 규칙을 유지했고 Windows 원격 파일의 실제 삭제는 수행하지 않았다. 새 규칙은 Codex 재시작 후 로드된다.

## 정확한 원 head의 CI

[CI #40334749874](https://github.com/edwardkim/rhwp/actions/runs/40334749874)의 preflight 로그는
`fast_pass=true reason=all-review-only-no-code-impact`와 base candidate `f770ef5024e014284fead45e839264beada35802`를
명시했다. PR 전체가 허용된 문서 범위이므로 과거 Full candidate 조회를 요구하지 않았다.

- Build & Test 성공; A~E builder/worker, Lint, Native Skia, Frontend, WASM Build 생략을 확인했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40334750206),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40334750416),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/40334750481),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/40334749951)의 문서 경로도 성공했다.
- 총 33개 check가 완료됐고 success/skipped/neutral만 있었다.
- CI Impact Policy는 성공이며 `v=8;cv=8;mode=selective;rust=0;fe=none;render=0;skia=0;ql=none`과
  `b=f770ef5024e014284fead45e839264beada35802`를 확인했다.

## Merge 후 contributor PR comment 계획

trailing head의 fast-pass·최종 Build & Test·정책과 같은 최신 base/head의 충돌 부재를 재확인하고
head를 고정해 일반 merge한다. review·오늘할일은 이 PR에 포함되므로 별도 기록 PR은 만들지 않는다.

병합 SHA가 upstream/devel에 포함됨을 확인하고 로컬 devel을 fast-forward한다. 문서-only 병합 뒤
제품 검증 CI와 duration metadata가 새로 실행되지 않았는지 관찰하며 검증을 재실행하지 않는다.
연결된 종료 대상 이슈는 없으므로 과거 이슈를 다시 처리하지 않는다.

PR에는 한국어 존댓말로 감사·실제 merge SHA·원 head와 trailing head CI 링크·허용 파일 범위·
fast-pass와 heavy worker skip·최종 집계·이슈 비해당·자동 정리 결과를 남기고 API로 본문을 확인한다.
이후 이번 작업의 원본 저장소 임시 원격 head를 최종 SHA lease로 삭제하고 로컬 브랜치를 정리한다.
기본 작업공간과 다른 worktree는 유지한다. output·review debug의 부재를 재확인해 종료한다.
