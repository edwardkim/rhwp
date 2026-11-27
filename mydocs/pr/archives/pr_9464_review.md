# PR #9464 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9464)

## 검토 버전

- 원 PR head `a62f0eecdaf414f54fb9d26eff460d35f7b0d3d2`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `55a3d25714a7dd023936cbf040590aa15d606919` | `a874cb1233ea3539e81b37a76fa946e88d02705b` |
| `c128f81d4a7033f28ae5c997f43b2dc690273bd9` | `744e54e5d1ad620958f22f36494956d3d63684f3` |
| `90f6d00656a80f70970f131c70216a5e944472c4` | `951486b2b70a339056ed3d1b2dc3b17f6431a16e` |
| `a62f0eecdaf414f54fb9d26eff460d35f7b0d3d2` | `0452161bcbcffdbaa2b09258f49e34fbd41fe749` |

## 구현 범위

셀 포커스에서 HWP 저장은 코어가 저장한 커서 값을 유지하고, 본문 포커스에서는 현재 커서를 저장한다. 편집 중인 셀의 선택과 문서 revision 및 실행 취소 이력은 저장 준비로 변하지 않는다. 실제 파일 쓰기 이후 저장 확인을 받는 순서와 이전 토큰의 revision 처리를 확인한다.

## Native C ABI 확인

최신 후보의 실제 Native worker·클립보드 검사에서 저장 revision과 셀 내용을 확인했습니다. 클립보드 계약은 HWP와 HWPX 저장·재열기도 실행했습니다.

## Merge 후 contributor PR comment 계획

표 셀에 포커스가 있는 상태에서 Native worker 저장·재열기와 revision 확인 결과를 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
