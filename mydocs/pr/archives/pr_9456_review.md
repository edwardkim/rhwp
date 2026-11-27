# PR #9456 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9456)

## 검토 버전

- 원 PR head `55a3d25714a7dd023936cbf040590aa15d606919`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `55a3d25714a7dd023936cbf040590aa15d606919` | `a874cb1233ea3539e81b37a76fa946e88d02705b` |

## 구현 범위

Native C ABI에서 표 셀의 선택·클릭·캐럿이 같은 주소를 가리키는지 확인하는 기준 검사를 추가했다. 아직 셀 편집을 지원하지 않던 시점의 거부와 본문 편집의 정상 대조를 포함한다. 후속 #9461·#9462·#9464·#9465·#9466이 이 worker 경로에서 실제 셀 편집·조합·저장 계약을 확장한다.

## Native C ABI 확인

최신 후보의 실제 macOS Native C ABI에서 worker의 대기열 한도, 스레드 소유권, 저장 revision, 저장 버퍼 반환과 세션 정리를 확인했습니다.

## Merge 후 contributor PR comment 계획

Native worker 대기열·스레드 소유권·버퍼 회수·저장 revision 검증을 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
