# Task M100 #5949 Stage O3-1 — Linux AArch64 release RED 계약

- **Issue**: [#5949](https://github.com/edwardkim/rhwp/issues/5949)
- **브랜치**: `task_m100_5949`
- **기준 commit**: `upstream/devel` `cd1fe7181c657b0cedeb6d7bd8e7e7980cb1e9ef`
- **계획 commit**: `6bbba9f3e`
- **수행일**: 2026-09-01 KST
- **Stage 판정**: 완료 — 구현 전 누락을 단독 검출하는 RED 계약 확보

## 1. live baseline

현재 `.github/workflows/release-binary.yml`의 build matrix는 다음 네 target이다.

| target | runner | release suffix |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `linux-x86_64` |
| `x86_64-apple-darwin` | `macos-14` | `macos-x86_64` |
| `aarch64-apple-darwin` | `macos-14` | `macos-aarch64` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `windows-x86_64` |

최근 정상 기준은 v0.8.4 Release Binary run
[`31553193683`](https://github.com/edwardkim/rhwp/actions/runs/31553193683)이다. 종료 상태는
success이며 v0.8.4 Release에는 다음 네 archive와 `SHA256SUMS.txt`가 있다.

- `rhwp-v0.8.4-linux-x86_64.tar.gz`
- `rhwp-v0.8.4-macos-aarch64.tar.gz`
- `rhwp-v0.8.4-macos-x86_64.tar.gz`
- `rhwp-v0.8.4-windows-x86_64.zip`

Linux AArch64 archive는 없다. 이는 #5949가 지적한 배포 누락과 일치한다.

## 2. 선행 GREEN

workflow를 수정하기 전에 현재 계약의 선행 실패가 없는지 다음 순서로 확인했다.

```text
python3 -m unittest scripts/tests/test_release_channel_policy_workflow.py
# 5 tests OK

python3 -m unittest scripts/tests/test_nextest_archive_workflow.py
# 15 tests OK

python3 -m unittest scripts/tests/test_gym_release_gate_workflow.py
# 24 tests OK

python3 -m unittest scripts/tests/test_workflow_contract_wiring.py
# 3 tests OK
```

합계 47건이 통과했다. 이후 RED는 기존 실패를 이어받은 것이 아니다.

## 3. 추가한 계약

`scripts/tests/test_release_channel_policy_workflow.py`에 Release Binary include matrix를 읽는
작은 parser와 `test_release_binary_matrix_includes_linux_aarch64`를 추가했다.

계약은 다음을 고정한다.

1. 기존 네 target과 `aarch64-unknown-linux-gnu`의 정확한 5종 집합
2. Linux AArch64 runner `ubuntu-24.04-arm`
3. archive 형식 `tar.gz`
4. archive suffix `linux-aarch64`
5. binary 이름 `rhwp`
6. target 및 archive suffix 중복 금지

새 파일을 만들지 않고 이미 CI에 배선된 release channel 계약에 추가했으므로 실행되지 않는 test를
도입하지 않는다.

## 4. RED 결과

추가 뒤 같은 계약 파일을 실행했다.

```text
python3 -m unittest scripts/tests/test_release_channel_policy_workflow.py
```

결과는 6건 중 기존 5건 통과, 신규 1건 실패다. 실패는 다음 한 항목뿐이다.

```text
Items in the second set but not the first:
'aarch64-unknown-linux-gnu'
```

runner·suffix 검사에 도달하기 전에 target 집합 누락이 먼저 실패했다. 다음 Stage에서 matrix 항목을
정확히 추가하면 같은 계약이 GREEN으로 바뀌어야 한다.

## 5. 보호 불변식 확인

- workflow, event, permission, runner, artifact에는 아직 변경이 없다.
- GitHub mutation, branch push, PR, workflow dispatch를 수행하지 않았다.
- 기존 네 release target과 v0.8.4 자산을 기준 증적으로 고정했다.
- 다른 PR review worktree와 미추적 검토 문서를 변경하지 않았다.

## 6. 다음 Stage 조건

Stage O3-2에서는 다음 최소 변경만 허용한다.

- Release Binary matrix에 Linux AArch64 항목 한 개 추가
- 4플랫폼 설명을 5플랫폼으로 현행화
- publish guide에 다섯 native asset과 실제 ARM64 검증 절차 반영

신규 계약이 GREEN으로 바뀌고 기존 47건이 계속 통과하기 전에는 Stage O3-2를 완료로 판정하지 않는다.
