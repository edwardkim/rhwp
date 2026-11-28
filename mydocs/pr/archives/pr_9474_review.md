# PR #9474 검토

[semanticist21 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9474)

## 검토 버전

- 원 PR head `acd5d38f3198e14a4ed24e8f5d81b51728dd90e3`
- 기준 devel `3a6b7d2b318287361f443b0814702783eb973533`
- 통합 코드 후보 `8550d0698f54fda3d66bee84232e753bb4599185`
- 로컬 회귀 실행 head `b45dfda9a3de46438d2dd14ea473bd96ec6e6c81`
- 검토 브랜치 `review/integration-8506-native-layout`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `acd5d38f3198e14a4ed24e8f5d81b51728dd90e3` | `7664a3ea5ccb01e57b31352d2f484f4e78c6a0cf` |

## 구현과 검증 범위

쪽을 넘어 나뉜 표에서, 글줄 없이 글자처럼 취급한 중첩 표만 담은 셀의 캐럿을 구합니다. 텍스트 run과 첫 쪽 셀 상자 조회에 실패하면 뒤쪽 후보 쪽의 렌더 트리에서 셀 상자를 찾아 안 여백을 적용합니다. 새 분기는 첫 쪽에서 위치를 찾은 경로 뒤에 실행됩니다.

기존 partial-nested-wrapper.hwpx에서 두 칸의 캐럿 조회가 성공하고 둘째 칸의 쪽 인덱스가 첫 칸보다 큰지 확인합니다. 원 PR의 재현은 뒤쪽 칸의 조회 오류였습니다. 메인터너 보정에서는 검사 입력과 쪽 소속의 관계를 테스트 첫머리에 명시했습니다. [회귀 검사](../../../tests/cases/split_table_continuation_cell_caret.rs)

후보에서 `continuation_cell_with_only_a_nested_table_has_a_caret`를 실행해 통과했습니다.

## 병합 후 원 PR 안내

이어지는 쪽에서 셀 캐럿을 찾은 결과와 첫 쪽의 기존 조회 순서를 안내합니다.

[통합 검증 결과](../assets/integration-8506-20261128/local-verification.json)
