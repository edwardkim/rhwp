---
kind: pr-review
status: active
pr: 5170
issue: 5164
author: jangster77
base: devel
head: codex/5164-single-nextest-archive
last_verified: 2026-08-17
---

# PR #5170 자체검토 - 단일 nextest archive와 Native Skia 병렬화

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5170](https://github.com/edwardkim/rhwp/pull/5170) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `codex/5164-single-nextest-archive` |
| 검증 code candidate SHA | `df4c76a61069dc808bdbb2f306cd30b10214b75b` |
| draft | `false` |
| merge state | `CLEAN` |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 최종 판단 전 최신 head와 GitHub Actions,
mergeability를 다시 확인한다.

## 변경 범위

- default-feature nextest archive를 worker마다 반복 빌드하지 않고 preflight 직후 한 번만 생성한다.
- 동일 archive를 slow filter와 regular hash partition 3개가 병렬로 소비한다.
- shard 실행 수 합계와 archive runnable 총수가 같은지 최종 `Build & Test`에서 검증한다.
- archive workflow 변경이 Rust lane을 우회하지 않도록 CI impact allowlist와 정책 테스트를 맞춘다.
- Native Skia가 lint·frontend 완료를 기다리지 않고 preflight 직후 archive와 병렬로 시작한다.

## GitHub Actions 검증

| 검증 | 결과 |
| --- | --- |
| code candidate | `df4c76a61` |
| CI run | [32002991473](https://github.com/edwardkim/rhwp/actions/runs/32002991473) 성공 |
| 전체 CI 시간 | 11분 27초 (`06:46:22Z`~`06:57:49Z`) |
| CI preflight | 성공, 42초 |
| 단일 archive builder | 성공, 6분 56초 |
| Native Skia | 성공, 7분 50초 |
| regular shard 1~3 | 모두 성공 |
| slow shard | 성공 |
| Build & Test | 성공 |
| CodeQL | JavaScript/TypeScript, Python, Rust 모두 성공 |

Native Skia와 archive builder는 preflight 종료 뒤 `06:47:12Z`에 동시에 시작했다. archive가
`06:54:08Z`에 끝난 직후 네 shard가 시작됐고, 가장 늦은 shard가 `06:57:39Z`에 끝난 뒤 최종
집계가 성공했다. stage 3 이전 성공 run은 14분 48초였으므로 실측 3분 21초가 단축됐다.

## 위험과 후속 조건

- Native Skia와 archive는 독립 runner이므로 병렬 실행이 archive CPU를 나눠 쓰지 않는다.
- lint 또는 frontend가 실패해도 이미 시작된 Native Skia runner 비용은 발생할 수 있다.
- archive build 6분 56초 중 workspace test binary compile/link가 다음 성능 병목이다.
- 32개 generated suite의 직접 재배치는 열린 PR과 충돌하므로 이번 범위에 포함하지 않는다.
- review·오늘할일 trailing commit은 workflow와 테스트 구조를 변경하지 않는다.

## 현재 권고

**병합 권고.** 단일 archive가 네 shard에 전달됐고 runnable 합계 계약, Native Skia 병렬 시작,
최종 `Build & Test`, CodeQL이 모두 통과했다. trailing 문서 head의 CI가 실패·대기 없이 완료되고
merge state가 `CLEAN`이면 admin merge한 뒤 로컬·원격 작업 브랜치와 전용 산출물을 정리한다.
