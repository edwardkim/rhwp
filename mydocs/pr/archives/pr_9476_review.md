# PR #9476 검토

[semanticist21 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9476)

## 검토 버전

- 원 PR head `d3c861ab0e921daf71c2c8611011dd8939a0b2fa`
- 기준 devel `3a6b7d2b318287361f443b0814702783eb973533`
- 통합 코드 후보 `8550d0698f54fda3d66bee84232e753bb4599185`
- 로컬 회귀 실행 head `b45dfda9a3de46438d2dd14ea473bd96ec6e6c81`
- 검토 브랜치 `review/integration-8506-native-layout`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `d3c861ab0e921daf71c2c8611011dd8939a0b2fa` | `4859a7099ff27b9e1b0118bc4e4151d1b7ca14df` |

## 구현과 검증 범위

캡션을 조판할 때 기존 `CaptionOwner.sec_idx`를 글줄의 소유 구역으로 전달합니다. 대상 owner가 없는 호출에서는 기존 기본값 0을 적용합니다.

둘째 구역에 표와 캡션을 넣고 캐럿 좌표를 구한 다음 그 위치를 누르는 합성 회귀입니다. 구역 1과 캡션 셀 주소 65534를 반환하는지 검증합니다. 메인터너 보정에서는 테스트 첫머리에 합성 입력과 반환값 계약을 기록했습니다. [회귀 검사](../../../tests/cases/caption_hit_section.rs)

후보에서 `caption_hit_in_second_section_reports_that_section`를 실행해 통과했습니다.

## 병합 후 원 PR 안내

둘째 구역 캡션의 소유 주소를 안내합니다.

[통합 검증 결과](../assets/integration-8506-20261128/local-verification.json)
