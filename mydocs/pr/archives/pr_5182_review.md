---
kind: pr-review
status: active
pr: 5182
issue: 5181
author: jangster77
base: devel
head: ci/5181-nextest-disk-preflight
last_verified: 2026-08-17
---

# PR #5182 자체검토 - Nextest archive 디스크 정리 사전 측정

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5182](https://github.com/edwardkim/rhwp/pull/5182) |
| 관련 issue | [#5181](https://github.com/edwardkim/rhwp/issues/5181) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `ci/5181-nextest-disk-preflight` |
| 검증 code candidate SHA | `b6582f6fe8ff4e3a977cc5896d8076b8efc85a29` |
| merge state | `MERGEABLE` |

workflow 변경 PR의 자체검토이므로 reviewer를 별도로 지정하지 않았다. 문서 trailing commit 이후에는
최신 head, Actions 결과와 mergeability를 다시 확인한다.

## 변경 범위

- Nextest archive 생성 전 root 파일시스템의 가용 공간을 `df`로 기록한다.
- 가용 공간이 30GB 미만일 때만 Android와 .NET 사전 설치 디렉터리를 크기 측정 후 제거한다.
- 가용 공간이 충분하면 정리를 건너뛰어 불필요한 16GB 삭제와 정리 시간을 피한다.
- Rust build cache 복원 직후에도 root 파일시스템 사용량을 기록한다.
- archive 생성·shard 분배 방식과 테스트 대상은 변경하지 않았다.

## 로컬 검증

다음 workflow 범위 검증을 완료했다.

- `python3 -m unittest scripts/tests/test_nextest_archive_workflow.py` — 12 passed
- `actionlint .github/workflows/build-nextest-archives.yml` — 통과
- `git diff --check` — 통과
- workflow-only 변경이므로 Cargo 전체 회귀 테스트는 실행하지 않았다.

## 실측 근거

기존 조건 없는 정리 동작을 확인하기 위해 [CI run #32018260377](https://github.com/edwardkim/rhwp/actions/runs/32018260377)을
확인했다.

- 정리 전 root: 84GB available (145GB 중 61GB 사용)
- Android: 11GB, .NET: 5.2GB
- 정리 후 root: 100GB available
- Rust build cache 복원 후: 98GB available
- archive build: 4분 47초, archive 기록 2.72초, 성공
- 생성 archive: 61 files / 49 binaries, 273,369,819 bytes

이 runner에서는 정리 전부터 84GB가 확보되어 있었으므로 16GB 삭제는 필요하지 않았다. 최신 구현은
30GB 미만인 경우에만 정리하고, 정리 전·후와 cache 복원 후의 사용량을 남긴다.

## GitHub Actions 검증

code candidate `b6582f6fe`의 Full CI 및 CodeQL을 완료했다.

| 검증 | 결과 |
| --- | --- |
| CI run | [32019381731](https://github.com/edwardkim/rhwp/actions/runs/32019381731) 성공 |
| CodeQL run | [32019381393](https://github.com/edwardkim/rhwp/actions/runs/32019381393) 성공 |
| Build test archive | 성공, 7분 28초 |
| Native Skia tests | 성공, 7분 43초 |
| regular shard 1~3 | 모두 성공 |
| slow shard | 성공, 1분 46초 |
| Analyze (Rust/Python/JavaScript) | 모두 성공 |
| Lint, preflight, frontend package gates | 모두 성공 |
| Frontend unit gates, WASM Build | 정책상 skipping |

## 위험과 후속 조건

- 30GB 임계값은 runner의 root 파일시스템 기준이며, 사전 설치 디렉터리가 없는 runner에서도 오류 없이
  `absent`를 기록해야 한다.
- 정리 대상은 현재 archive 빌드에 사용하지 않는 Android와 .NET 디렉터리로 한정했다.
- 실제 runner의 정리 전·후·cache 복원 후 `df` 출력으로 임계값 판단과 공간 변화를 추적할 수 있다.
- 문서 trailing commit은 workflow·source·test를 변경하지 않으며, code candidate의 Full CI 결과를
  대체하지 않는다.

## 현재 권고

**병합 권고.** 디스크가 충분한 runner에서는 정리를 생략하고, 부족한 runner에서만 정리하는 조건부
동작과 전후 측정이 구현됐다. code candidate의 로컬 검증과 Full CI·CodeQL이 모두 통과했다.
문서 trailing head의 최신 preflight·aggregate가 성공하고 merge state가 `CLEAN`이면 작업지시자 승인
후 병합할 수 있다.
