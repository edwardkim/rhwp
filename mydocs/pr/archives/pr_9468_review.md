# PR #9468 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9468)

## 검토 버전

- 원 PR head `0124ece786fdbb52349a6795a696eaaf30ab985e`
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
| `22127cb7b64408041745f4802c4215564b17e98f` | `e56df09cbc5a4e52fa12b3c145a35d4753f2834a` |
| `e98e6bfb2bf55afa693e9a09061654a00908c3ef` | `57f71d6f5244ecfa471a2c2d2f01dd9352088c9b` |
| `0124ece786fdbb52349a6795a696eaaf30ab985e` | `37bec05ec3c5d0c245a7c717513448f2fea6e4fe` |

## 검토 근거

중첩 표의 한 셀 문단에서 일반 텍스트를 복사·잘라내기·붙여넣기하고 Tab으로 셀을 순회합니다. 보호된 셀에서는 일반 텍스트 복사가 가능합니다. 보정 커밋 `7490f5271f`에서 앞선 회귀의 복사 거부 기대값을 갱신했습니다. 보호된 셀의 쓰기 거부와 선택·revision·undo 보존 검사도 적용합니다.

[클립보드 회귀](../../../tests/cases/issue_536_native_clipboard.rs) · [셀 순회 회귀](../../../tests/cases/issue_8701_native_table_navigation.rs)

## Native C ABI 확인

최신 후보의 실제 Native worker에서 중첩 셀 일반 텍스트 클립보드·Tab 순회가 통과했습니다. 클립보드 검사는 HWP·HWPX 저장과 반복 재열기를 실행했습니다.

## 원 PR CI의 보호 셀 회귀

원 PR의 Archive B에서 `copy_selection`이 보호된 셀을 거부해야 한다는 이전 assertion이 실패했습니다. 통합 후보에는 #9468의 읽기 전용 일반 텍스트 복사 계약과 #9470의 후속 검사를 적용했습니다. 최종 기본 기능 nextest에서 보호 셀 복사와 편집 거부 검사, 보호된 상위 셀이 있는 중첩 경계 검사가 모두 통과했습니다. [원 PR 실패 작업](https://github.com/edwardkim/rhwp/actions/runs/46024613149/job/140605524718)

## Merge 후 contributor PR comment 계획

중첩 셀 일반 텍스트 복사·붙여넣기·Tab 순회 및 두 형식 저장 결과를 안내합니다. 보호 셀 복사에 관한 앞선 검사와의 충돌을 정리한 이유와 #9470의 후속 검사까지 설명합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
