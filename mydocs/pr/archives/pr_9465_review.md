# PR #9465 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9465)

## 검토 버전

- 원 PR head `3eafd9d13d00f6ac82e7b934086ebe4e6dcdeea3`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `55a3d25714a7dd023936cbf040590aa15d606919` | `a874cb1233ea3539e81b37a76fa946e88d02705b` |
| `c128f81d4a7033f28ae5c997f43b2dc690273bd9` | `744e54e5d1ad620958f22f36494956d3d63684f3` |
| `90f6d00656a80f70970f131c70216a5e944472c4` | `951486b2b70a339056ed3d1b2dc3b17f6431a16e` |
| `a62f0eecdaf414f54fb9d26eff460d35f7b0d3d2` | `0452161bcbcffdbaa2b09258f49e34fbd41fe749` |
| `3eafd9d13d00f6ac82e7b934086ebe4e6dcdeea3` | `28c75a7e5dc8a45279168a8dc542579a7ad20676` |

## 구현 범위

중첩 표의 전체 셀 경로를 기존 코어 치환에 전달한다. 한 셀의 한 문단에서 텍스트와 한글 조합을 처리하며 상위 표와 셀의 잠금·보호를 확인한다. 공개 육아기 근로시간 단축 신청서에서 이름·생년월일을 편집하고 이웃 셀·문단 구조를 저장·재열기 뒤에도 대조한다.

## Native C ABI 확인

최신 후보의 실제 Native worker에서 중첩 셀의 편집과 IME 세션을 실행했습니다. 클립보드 경로의 HWP·HWPX 재열기에서 이웃 셀 내용도 확인했습니다.

## Merge 후 contributor PR comment 계획

중첩 셀의 편집·IME와 저장 후 이웃 셀 보존을 확인한 결과를 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
