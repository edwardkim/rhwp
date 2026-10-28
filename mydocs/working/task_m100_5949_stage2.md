# Task M100 #5949 Stage O3-2 — Linux AArch64 release matrix 구현

- **Issue**: [#5949](https://github.com/edwardkim/rhwp/issues/5949)
- **브랜치**: `task_m100_5949`
- **기준 commit**: `upstream/devel` `cd1fe7181c657b0cedeb6d7bd8e7e7980cb1e9ef`
- **RED 계약 commit**: `01ab9281c`
- **수행일**: 2026-09-01 KST
- **Stage 판정**: 완료 — 최소 matrix 구현으로 RED 계약 GREEN

## 1. 구현 범위

### Release Binary matrix

`.github/workflows/release-binary.yml`의 기존 Linux x86_64 항목 다음에 아래 target 한 개를 추가했다.

| 필드 | 값 |
| --- | --- |
| target | `aarch64-unknown-linux-gnu` |
| runner | `ubuntu-24.04-arm` |
| archive | `tar.gz` |
| archive suffix | `linux-aarch64` |
| binary | `rhwp` |

workflow 상단 설명도 Linux 2종, macOS 2종, Windows 1종의 5플랫폼으로 현행화했다.

### 배포 정본

`mydocs/manual/publish_guide.md`에 다음을 추가했다.

- GitHub Release CLI를 공식 배포 대상으로 명시
- `release-binary.yml`의 trigger·역할
- 다섯 native target·runner·archive suffix 표
- `tag=test` dry-run이 release job을 실행하지 않는 경계
- Linux AArch64 artifact 내부 파일, ELF architecture, `--version` 확인 절차
- v0.8.5 배포 전 5플랫폼 archive·checksum 체크리스트

runner label은 GitHub-hosted runners 공식 문서를 정본 링크로 사용했다.

## 2. RED → GREEN

Stage O3-1의 신규 계약을 같은 명령으로 다시 실행했다.

```text
python3 -m unittest scripts/tests/test_release_channel_policy_workflow.py
```

수정 전에는 `aarch64-unknown-linux-gnu` 누락으로 1건 실패했으며, 수정 뒤에는 6/6이 통과했다.
계약 parser는 target과 archive suffix의 중복도 함께 거부한다.

## 3. 기존 계약 무회귀

다음 계약을 순차 실행했다.

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

합계 48건이 통과했다. 기존 네 target, 철회 채널 차단, gym release gate 독립성, workflow 계약 CI
배선에 회귀가 없다.

추가 형식 검증도 통과했다.

```text
git diff --check
python3 scripts/check_markdown_links.py mydocs/manual/publish_guide.md
```

## 4. 보호 불변식 대조

- `push.tags: v*`와 `workflow_dispatch` trigger는 바뀌지 않았다.
- `contents: write` permission과 release job 조건은 바뀌지 않았다.
- 기존 네 target의 runner·archive·suffix·binary 값은 바뀌지 않았다.
- build·verify·package·upload step은 공통 matrix 경로를 그대로 사용한다.
- Linux AArch64는 native runner이므로 cross linker나 self-hosted runner를 추가하지 않았다.
- `native-skia`, 제품 source, package version, secret, environment를 바꾸지 않았다.
- 아직 push, PR, workflow dispatch, release mutation은 수행하지 않았다.

## 5. 잔여 검증

로컬 텍스트 계약은 matrix 배선을 검증하지만 runner image의 실제 toolchain·link·binary 실행과 artifact
생성을 증명하지 않는다. Stage O3-3의 비례 검증과 제출 준비를 거친 뒤, Stage O3-4에서 exact remote
head의 `ubuntu-24.04-arm` job으로 이 경계를 확인한다.
