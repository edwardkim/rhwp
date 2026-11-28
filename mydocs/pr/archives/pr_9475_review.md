# PR #9475 검토

[semanticist21 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9475)

## 검토 버전

- 원 PR head `080b0abd74c8e60392fc2c11720f0ea774c8f990`
- 기준 devel `3a6b7d2b318287361f443b0814702783eb973533`
- 통합 코드 후보 `8550d0698f54fda3d66bee84232e753bb4599185`
- 로컬 회귀 실행 head `b45dfda9a3de46438d2dd14ea473bd96ec6e6c81`
- 검토 브랜치 `review/integration-8506-native-layout`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `080b0abd74c8e60392fc2c11720f0ea774c8f990` | `a671e74756a5f05d89eae69fe462e1ecc211a6d8` |

## 구현과 검증 범위

머리말·꼬리말의 선택 범위를 모델에서 이동 가능한 길이로 검사하고, 히트 테스트가 덧말과 글자 겹치기를 각각 한 칸으로 계산하도록 변경했습니다. 기존 캐럿과 선택 사각형의 모델 오프셋 및 표시 폭 처리 경로를 재사용합니다.

글자 셋과 겹침·덧말이 있는 합성 머리말에서 0~5 선택이 끝 캐럿까지 닿고, 마지막 덧말 뒤를 누르면 오프셋 5를 반환하는지 확인합니다. 메인터너 보정에서는 테스트 첫머리에 합성 입력과 반환값 계약을 기록했습니다. [회귀 검사](../../../tests/cases/header_footer_compose_slots.rs)

후보에서 `header_selection_reaches_past_trailing_ruby_and_overlap`를 실행해 통과했습니다. 마지막 덧말 뒤의 히트 회귀도 통과했습니다.

## 병합 후 원 PR 안내

머리말 선택 끝점과 마지막 덧말 뒤를 누를 때의 모델 오프셋을 안내합니다.

[통합 검증 결과](../assets/integration-8506-20261128/local-verification.json)
