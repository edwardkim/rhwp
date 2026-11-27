# PR #9462 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9462)

## 검토 버전

- 원 PR head `90f6d00656a80f70970f131c70216a5e944472c4`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `55a3d25714a7dd023936cbf040590aa15d606919` | `a874cb1233ea3539e81b37a76fa946e88d02705b` |
| `c128f81d4a7033f28ae5c997f43b2dc690273bd9` | `744e54e5d1ad620958f22f36494956d3d63684f3` |
| `90f6d00656a80f70970f131c70216a5e944472c4` | `951486b2b70a339056ed3d1b2dc3b17f6431a16e` |

## 구현 범위

한 셀 문단에서 한글 조합 문맥을 읽고 시작·갱신·확정·취소를 처리한다. 같은 revision이라도 커서가 이웃 셀로 이동하면 이전 셀의 조합 문맥은 효력을 잃는다. 하나의 확정된 조합을 독립된 실행 취소 단위로 복원하며, 조합 중 저장은 거부한다.

## Native C ABI 확인

최신 후보의 실제 Native worker에서 셀 IME 편집이 통과했습니다. 조합 경계와 선택 복원은 별도의 Rust 회귀에서 확인합니다.

## Merge 후 contributor PR comment 계획

표 셀 IME의 조합 처리와 선택 복원에 대한 Rust 및 Native worker 결과를 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
