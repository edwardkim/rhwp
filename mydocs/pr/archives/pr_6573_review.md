---
kind: pr-review
status: remote-validation-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
pr: 6573
issue: 5949
author: edwardkim
---

# PR #6573 self-review — Linux AArch64 릴리즈 바이너리

## 결론

**승인.** code candidate `eaf1bcfc9fb4ef030e5951d560ee8db364c92987`은
`Release Binary` matrix에 native `ubuntu-24.04-arm` 기반
`aarch64-unknown-linux-gnu` target 한 개를 추가한다. 기존 네 target, tag trigger,
`workflow_dispatch`, `contents: write`, release job 조건과 action pin은 바꾸지 않았다.

로컬 workflow 계약 48건, YAML 구조, 문서 링크와 diff 검사가 통과했고 blocking finding은 없다.
다만 runner label·toolchain·linker·바이너리 실행·artifact는 로컬 x86_64 환경에서 증명할 수 없는
O3 실행 계약이다. 따라서 이 승인은 기술 검토 판정이며, 병합 전에는 최신 PR head의 Full CI와
별도 승인된 `Release Binary(tag=test)` 다섯 job, Linux AArch64 archive의 독립 검증이 모두
성공해야 한다.

이 PR은 collaborator 본인 self-review이므로 reviewer와 GitHub approve review를 지정하지 않는다.
이 문서의 작성은 workflow dispatch, merge 또는 #5949 close 승인이 아니다.

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `github_operations.md`, `publish_guide.md`,
  `codex/docs_and_git_workflow.md`
- 변경은 O3 job matrix·runner·artifact 실행 계약이다. renderer, fixture, HWP/HWPX/PDF와
  사용자 화면을 바꾸지 않아 visual fixture 경로는 적용하지 않았다.

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6573](https://github.com/edwardkim/rhwp/pull/6573) / `edwardkim` |
| 관련 이슈 | [#5949](https://github.com/edwardkim/rhwp/issues/5949) (`Relates to #5949`) |
| base / head | `devel@cd1fe7181c657b0cedeb6d7bd8e7e7980cb1e9ef` / `task_m100_5949` |
| code candidate | `eaf1bcfc9fb4ef030e5951d560ee8db364c92987` |
| 변경 규모 | 8 files, `+589/-1`, 4 commits |
| 작성 시점 상태 | Open, non-draft, `MERGEABLE`, Full CI 진행 중이라 `BLOCKED` |
| reviewer | self PR이므로 지정하지 않음 |

mergeability와 CI는 작성 시점 참고값이다. 이 review와 오늘할일을 추가한 trailing head가 생기면
그 SHA의 상태를 다시 확인한다. workflow 자체를 바꾼 PR이므로 trusted controller의 exact-head 증명이
성립하지 않는 한 review-only tail을 임의로 fast-pass로 간주하지 않는다.

## 목적과 변경 범위 정합성

- Linux AArch64 target은 `aarch64-unknown-linux-gnu`, runner는 GitHub 표준 native
  `ubuntu-24.04-arm`으로 고정했다.
- archive는 기존 Linux/macOS 경로를 재사용하며 suffix만 `linux-aarch64`로 분리했다.
- native runner가 빌드 직후 `rhwp --version`을 실행하므로 컴파일 성공만으로 artifact를 게시하지 않는다.
- `tag=test`는 `v`로 시작하지 않아 build와 artifact upload만 수행하고 release job은 skip한다.
- 정식 `v*` 실행은 기존 `needs: build`에 따라 다섯 build가 성공해야 archive와
  `SHA256SUMS.txt`를 GitHub Release에 첨부한다.
- Linux musl·ARM32·Windows ARM64·installer·container·package manager와 `native-skia` 배포는
  범위에 포함하지 않았다.

제품 Rust source, Cargo feature, dependency, lockfile, package version과 Studio/WASM 경로는
바뀌지 않는다. 변경으로 추가되는 운영 비용은 release workflow가 실행될 때의 ARM64 build job
한 개와 7일 보존 artifact 한 개다.

## self-review findings

### blocking finding 없음

- 신규 matrix 값은 #5949 제안과 GitHub-hosted runner 정본의 현재 label에 일치한다.
- 기존 target의 target·runner·archive·suffix·binary 값은 diff에서 바뀌지 않았다.
- 권한 확대, privileged event, secret, self-hosted runner와 외부 action을 추가하지 않았다.
- test는 exact target 집합, Linux AArch64 행의 전 필드, suffix 누락·중복을 실패시키므로
  단순 행 존재 검사보다 강한 회귀 계약이다.
- publish guide는 dry-run, archive 구성, ELF architecture, version과 실제 v0.8.5 release
  종료 조건을 분리해 운영자가 구현 merge를 배포 완료로 오인하지 않게 한다.

### 원격에서만 판정 가능한 항목

- `ubuntu-24.04-arm` job queue와 image 제공
- stable Rust target 설치와 native linker 성공
- `rhwp --version` 실행
- `rhwp-test-linux-aarch64.tar.gz` upload·download와 ELF AArch64 판독

이 항목 중 하나라도 실패하면 runner 권한 확대나 self-hosted runner로 우회하지 않고 merge를 보류한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| release channel policy | 6 tests OK |
| nextest archive workflow | 15 tests OK |
| gym release gate workflow | 24 tests OK |
| workflow contract wiring | 3 tests OK |
| PyYAML 6.0.1 `BaseLoader` | 구문 parse, matrix 5행과 exact ARM64 행 확인 |
| `git diff --check` | 통과 |
| 변경 문서 Markdown 링크 | 통과 |

합계 48개 workflow 계약이 통과했다. 로컬에는 `actionlint`와 `yamllint`가 없어 설치되지 않은
검증 도구를 즉석에서 추가하지 않았고, 저장소에 이미 있는 Python 계약과 설치된 PyYAML을 사용했다.

제품 source와 build feature가 동일하므로 로컬 x86_64 Rust·WASM·Studio 전체 회귀는 수행하지 않았다.
그 검증은 ARM64 runner 호환성을 증명하지 못한다. 대신 workflow 변경의 최신 Full CI와 native ARM64
dry-run을 병합 필수 gate로 남겼다.

## 잔여 위험과 rollback

- GitHub-hosted runner image나 action의 ARM64 지원이 예상과 다르면 실제 job log에서 드러난다.
- native dependency 또는 linker가 실패하면 필요한 system dependency를 먼저 규명하고 구현 범위 변경을
  다시 승인받는다.
- suffix가 충돌하거나 archive가 누락되면 contract와 packaging glob을 함께 수정하고 재실행한다.
- `tag=test`에서 release job이 실행되면 즉시 중단하고 조건을 수정한다.

merge 전 rollback은 작업 branch와 PR을 폐기하는 것으로 끝난다. merge 뒤에는 #6573 merge commit을
revert해 matrix·계약·문서를 함께 제거한다. 게시된 release asset은 같은 버전으로 조용히 교체하지 않고
별도 patch release 절차를 사용한다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `eaf1bcfc9fb4ef030e5951d560ee8db364c92987`
- 원격 필수 gate: 최신 PR head Full CI, `Release Binary(tag=test)` 다섯 build job,
  Linux AArch64 `--version`과 artifact upload 성공
- artifact gate: archive의 `rhwp/rhwp`, `LICENSE`, `README.md`, `README_EN.md`와
  ELF 64-bit AArch64 확인
- merge 방식: 작업지시자 별도 승인 뒤 `--admin` 없이 정상 2-parent merge commit
- 이슈 종료: 구현 merge 뒤에도 #5949를 열어 두고, v0.8.5 실제 archive와 checksum 검증 뒤 close

## 후속 순서

1. 이 review와 오늘할일 trailing commit을 별도 승인 뒤 원격 branch에 push한다.
2. 최신 PR head의 Full CI와 `MERGEABLE/CLEAN`을 확인한다.
3. 별도 승인 뒤 exact branch에서 `Release Binary`를 `tag=test`로 dispatch한다.
4. 다섯 matrix job과 Linux AArch64 archive를 검증해 Stage O3-4 기록을 완성한다.
5. 최종 merge 승인을 받은 뒤 정상 merge commit 방식으로 `devel`에 병합한다.
6. v0.8.5 release에서 실제 Linux AArch64 asset과 checksum을 검증한 뒤 #5949를 종료한다.
