# PR #9466 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9466)

## 검토 버전

- 원 PR head `98cf98902b37cbfdc6eed8856dc07cbcaed64352`
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
| `98cf98902b37cbfdc6eed8856dc07cbcaed64352` | `384cc4933bc38ef6c612bd9c3229fbfbc29cf840` |

## 구현 범위

단일·중첩 셀에서 문단 분리와 바로 앞 문단에 합치기를 한 편집 revision으로 적용한다. 같은 셀의 선택과 문단 주소를 사용하고, 잠금·보호와 필드 경계, 조합 중 요청을 검사한다. 실제 Native worker의 `tableCellParagraphCommands` 보고와 HWP·HWPX 저장 후 이웃 셀의 구조 보존이 검증 범위다.

## Native C ABI 확인

최신 후보의 실제 Native worker에서 문단 분리·이어받기 명령을 검사했습니다. 중첩 셀 주소, 편집 revision과 되돌리기 계약은 저장·재열기 경로와 함께 검증했습니다.

## Merge 후 contributor PR comment 계획

중첩 셀의 문단 분리·이어받기와 주소 복원, 실행 취소를 확인한 결과를 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
