# PR #9469 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9469)

## 검토 버전

- 원 PR head `1de96f394ff4fab7efa6e4e5c29808db35250726`
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
| `8371f5b03448e31c048ffd7a5213965c1969bcd0` | `91021a4bc2f654bf76cb919d3097300782d709ba` |
| `1de96f394ff4fab7efa6e4e5c29808db35250726` | `2ffc72e0d7053c369b12e713d3b651bc15985d24` |

## 검토 근거

#9468까지의 Native 편집·클립보드 커밋을 먼저 적용했습니다. 고유 커밋 두 개로 본문·중첩 표 검색, 명시 범위 치환 및 접근성 응답에 선택적 `cellLayout`을 추가합니다. 검색은 문서 revision의 Unicode scalar 범위를 반환합니다. 치환은 전달된 범위를 사용하고 undo는 치환 직전 선택을 복원합니다. `cellLayout`은 직접 소속된 표의 셀 좌표와 병합 범위를 담습니다.

[검색·치환 회귀](../../../tests/cases/issue_8701_native_search.rs) · [접근성 회귀](../../../tests/cases/issue_536_native_accessibility.rs) · [실제 Native worker 계약](../../../bindings/Native/tests/native_search_contract.py)

## Native C ABI 확인

최신 후보의 실제 Native worker에서 명시 범위 검색·치환과 실행 취소, 접근성 전송을 검증했습니다. 접근성 전송의 다섯 검사가 통과했습니다. 검색 계약은 HWP와 HWPX를 저장하고 각 결과를 반복 재열었습니다.

## 원 PR CI의 보호 셀 회귀

원 PR의 Archive D에서 `copy_selection`이 보호된 셀을 거부해야 한다는 이전 assertion이 실패했습니다. 통합 후보에는 #9468의 읽기 전용 일반 텍스트 복사 계약과 #9470의 후속 검사를 적용했습니다. 최종 기본 기능 nextest에서 보호 셀 복사와 편집 거부 검사, 보호된 상위 셀이 있는 중첩 경계 검사가 모두 통과했습니다. [원 PR 실패 작업](https://github.com/edwardkim/rhwp/actions/runs/46027270410/job/140611809301)

## Merge 후 contributor PR comment 계획

명시 검색 범위 치환, stale revision, 실행 취소, 요청형 접근성 정보와 두 형식 저장 검증을 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
