# PR #4923 검토 - CI agent preflight lint 배선

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4923](https://github.com/edwardkim/rhwp/pull/4923) |
| 작성자 | `kevin9327` (`kevin`) |
| 검토 방식 | #4931 누적 통합을 위한 archive review |
| base / head | `devel` / `ci/wire-agent-preflight` |
| source candidate | `e828857b3ffdea1b34b02cd1c01127feee0b7cee` |
| 통합 commit | `3d469ee78bdeac88e5ae54b5df76c3849497024c` |
| 규모 | 1 file, +10 / -0 |
| 작성 시점 상태 | `OPEN`, `MERGEABLE`, `CLEAN` |

## 범위와 판단

- 기존 `tools/agent_preflight.py`를 lint job에서 실행해 선언되지 않은 command·failure contract drift를 CI에서
  차단한다.
- lint가 이미 만든 debug binary를 재사용하고 `--no-network`로 큐 규율의 읽기 전용 네트워크 확인을 제외해,
  새 release build나 외부 네트워크 의존성을 추가하지 않는다.

## 검증

- source candidate의 Build & Test, Native Skia, CodeQL, 기본 feature 세 shard·slow shard 및 lint가 성공했다.
- frontend/WASM job은 변경 영향에 따라 skipped였다.
- #4931 누적 tree의 전체 `release-test` integration 회귀는 종료 코드 `0`으로 통과했다.

## 위험과 권고

preflight는 CI controller 경로를 바꾸므로 후속 정책 변경 PR은 이 PR의 green 결과만 재사용하지 않고 trusted
controller 규칙을 따라야 한다. 현재 배선은 최소 변경이고 source CI와 누적 회귀가 통과했으므로 #4931 통합
merge를 권고하며, 원 PR은 merge 뒤 supersede 처리한다.
