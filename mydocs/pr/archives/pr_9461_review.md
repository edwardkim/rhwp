# PR #9461 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9461)

## 검토 버전

- 원 PR head `c128f81d4a7033f28ae5c997f43b2dc690273bd9`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `55a3d25714a7dd023936cbf040590aa15d606919` | `a874cb1233ea3539e81b37a76fa946e88d02705b` |
| `c128f81d4a7033f28ae5c997f43b2dc690273bd9` | `744e54e5d1ad620958f22f36494956d3d63684f3` |

## 구현 범위

같은 최상위 표 셀의 한 문단에서 선택 영역을 삽입·삭제·치환하고, 실행 취소와 다시 실행으로 문서 및 선택을 함께 복원한다. 보호된 셀과 잠긴 표, 두 셀에 걸친 편집 요청은 거부한다. 원 PR은 #9456의 worker 계약을 이어받는다.

## Native C ABI 확인

최신 후보의 실제 Native worker에서 표 셀 편집을 실행했습니다. 보호된 셀의 일반 텍스트 복사와 편집 요청은 #9468·#9470의 계약과 함께 대조합니다.

## Merge 후 contributor PR comment 계획

최상위 표 셀에서 편집과 undo·redo가 동작하고 보호 경계가 유지되는지 확인한 검사를 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
