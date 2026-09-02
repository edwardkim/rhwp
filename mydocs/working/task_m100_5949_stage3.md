# Task M100 #5949 Stage O3-3 — 로컬 제출 후보 검증

- **Issue**: [#5949](https://github.com/edwardkim/rhwp/issues/5949)
- **브랜치**: `task_m100_5949`
- **제품·workflow candidate**: `34488064ddff77347a4cbde22cddfb51b6cd18fb`
- **최신 base**: `upstream/devel` `cd1fe7181c657b0cedeb6d7bd8e7e7980cb1e9ef`
- **수행일**: 2026-09-01 KST
- **Stage 판정**: 로컬 검증 통과 — 원격 ARM64 실행 대기

## 1. 기준선 재확인

검증 직후 `upstream/devel`을 fetch했다. branch는 base보다 3커밋 앞서고 뒤처진 commit은 0개다.
merge base와 최신 `upstream/devel`은 모두 `cd1fe7181`로 일치한다.

GitHub live metadata도 다음과 같다.

- repository: public `edwardkim/rhwp`
- 인증 계정 권한: `ADMIN`
- 기본 브랜치: `main`
- `devel`: protected
- required status: `Build & Test`

작업공간은 clean이며 기존 PR review worktree는 변경하지 않았다.

## 2. workflow 계약 검증

다음을 순차 실행했다.

```text
python3 -m unittest scripts/tests/test_release_channel_policy_workflow.py
# 6 tests OK

python3 -m unittest scripts/tests/test_nextest_archive_workflow.py
# 15 tests OK

python3 -m unittest scripts/tests/test_gym_release_gate_workflow.py
# 24 tests OK

python3 -m unittest scripts/tests/test_workflow_contract_wiring.py
# 3 tests OK
```

합계 48건이 통과했다. Linux AArch64 신규 target뿐 아니라 기존 release channel, nextest archive,
독립 gym release gate와 CI wiring 계약도 통과했다.

## 3. YAML 구조 검증

로컬 환경에는 `actionlint`와 `yamllint`가 없다. 설치되지 않은 도구를 즉석에서 내려받아 검증 공급망을
넓히지 않고, 이미 설치된 PyYAML 6.0.1의 `BaseLoader`로 workflow를 파싱했다.

파싱 결과 build matrix는 정확히 5개이며 Linux AArch64 행은 다음 객체로 읽혔다.

```text
target: aarch64-unknown-linux-gnu
runner: ubuntu-24.04-arm
archive: tar.gz
archive_suffix: linux-aarch64
binary_name: rhwp
```

YAML 구조와 matrix 값은 로컬에서 확인됐다. GitHub expression, runner image, toolchain 설치와 실제 binary
실행의 최종 권위는 Stage O3-4의 exact-head Actions run이다.

## 4. 문서·diff 검증

다음 검사가 통과했다.

```text
git diff --check
python3 scripts/check_markdown_links.py \
  mydocs/plans/task_m100_5949.md \
  mydocs/working/task_m100_5949_stage1.md \
  mydocs/working/task_m100_5949_stage2.md \
  mydocs/manual/publish_guide.md \
  mydocs/orders/20260901.md
```

변경은 release workflow, 이미 배선된 workflow 계약 test, 배포 정본과 #5949 작업 기록뿐이다.
제품 source, Cargo feature, package manifest와 lockfile은 바뀌지 않았다.

## 5. 비례 검증 판정

이번 O3 변경은 release job matrix·runner·artifact 실행 계약을 바꾸지만 빌드할 Rust source와 feature는
바꾸지 않는다. 따라서 로컬 x86_64 전체 Rust·WASM·Studio 회귀는 ARM64 runner 호환성을 증명하지 못하면서
중복 비용만 발생하므로 수행하지 않았다.

대신 다음 원격 증적을 필수로 남긴다.

1. PR exact head의 required CI `Build & Test`
2. workflow 변경 PR의 Full fallback
3. `Release Binary` workflow_dispatch `tag=test`
4. 다섯 matrix job 성공
5. Linux AArch64의 toolchain install, release build, `--version`, artifact upload 성공
6. 내려받은 archive의 파일 목록과 ELF 64-bit AArch64 판독

## 6. 권한·배포 경계

- `contents: write`는 release asset 첨부를 위한 기존 값이며 변경하지 않았다.
- event, release job `if`, `needs: build`, action pin을 변경하지 않았다.
- ARM64 build job은 checkout, cache, upload-artifact의 GitHub 공식 action만 사용한다.
- `tag=test`는 `v`로 시작하지 않아 release job 조건을 만족하지 않는다.
- dry-run은 artifact를 만들지만 GitHub Release·npm·extension publish를 시작하지 않는다.

## 7. rollback

원격 실행에서 runner label, linker, binary 실행, artifact 중 하나라도 실패하면 PR merge를 보류한다.
권한 확대나 self-hosted runner 추가로 우회하지 않고 원인을 좁혀 수정 계획을 재승인받는다.

merge 전 rollback은 이 작업 branch를 폐기하는 것으로 끝난다. merge 뒤 rollback은 #5949 merge commit을
revert해 matrix 항목·계약·문서만 함께 제거한다. 게시된 release archive는 같은 버전으로 조용히 교체하지
않고 새 patch release를 사용한다.

## 8. 다음 gate

로컬 제출 후보는 준비됐지만 remote push와 PR 생성은 각각 별도 승인 대상이다. 승인 뒤 branch를 push하고
`devel` 대상 PR을 만든 다음 exact head의 CI와 ARM64 dry-run을 관찰한다.
