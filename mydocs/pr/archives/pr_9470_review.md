# PR #9470 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9470)

## 검토 버전

- 원 PR head `8916cb277fe2ebc7921346782b86be516c89db5f`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `c35c78a47319d215a5d0bfeeea0edaa2bd5ea31f` | `8ff388002d5117ea5c6f1bb0df6e4b4d3c5b7ea8` |
| `8916cb277fe2ebc7921346782b86be516c89db5f` | `0b62f72b31a8e521f4cf957733dd1611ffead268` |

## 구현과 체리픽 보정

#9469 위에 글자 굵기·크기와 문단 정렬 명령을 적용했습니다. 본문과 중첩 표의 한 문단을 대상으로 하며, 한 번의 유효한 편집을 한 revision과 undo 항목으로 기록합니다. 보호 셀과 조합 중 편집, 같은 서식의 재적용 및 이웃 셀 보존이 검토 대상입니다.

보호된 셀의 읽기 전용 복사 검사는 메인터너 보정과 겹치는 구간에서 충돌했습니다. 원 PR의 한 줄 붙여넣기 및 잘라내기 거부 검사까지 반영했고, 결과 테스트 파일은 원 PR head와 바이트 단위로 일치합니다.

[Rust 명령 검사](../../../tests/cases/issue_536_native_commands.rs) · [Native worker 검사](../../../bindings/Native/tests/native_commands_contract.py)

## Native C ABI 확인

최신 후보의 실제 Native worker에서 글자 굵기·크기와 문단 정렬을 실행했습니다. 같은 서식의 재적용과 유효하지 않은 크기의 처리, 되돌리기를 검사했습니다.

## 회귀 실행 확인

기본 기능의 Rust 회귀에서 본문·중첩 셀 서식과 입력 검증이 통과했습니다. 초기 전체 회귀 중 공용 CLI가 Skia 포함 빌드로 바뀌어 다섯 케이스가 실패했습니다. 해당 케이스 전부를 당시 빌드한 기본 기능 CLI에 고정하여 재실행했고 모두 통과했습니다.

## Merge 후 contributor PR comment 계획

본문·중첩 셀 서식과 보호 셀 읽기 및 편집 거부 검증을 안내합니다. 체리픽 중 중복된 보호 셀 검사 충돌을 원 PR의 최종 검사와 같게 해결한 결과를 설명합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
