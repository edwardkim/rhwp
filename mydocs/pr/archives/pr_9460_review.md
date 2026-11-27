# PR #9460 검토

[donggyun112 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9460)

## 검토 버전

- 원 PR head `01cfd69f08a042052a26214c1ea65aa69338eae7`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `01cfd69f08a042052a26214c1ea65aa69338eae7` | `273db147af5cfea22108c74bdcf72aa14168a9af` |

## 구현 범위

직렬화된 표 행이 하나일 때만, 같은 논리 행에 놓인 모든 셀이 전체 열을 덮고 영역이 같은 행을 가리키는 경우 행 주소를 0으로 맞춘다. 지정한 공개 샘플 `nested_row_first_line_height.hwpx`의 바깥 표 행 주소를 바꿔 재현했다. 명시적 빈 행, 누락 열, 빠진 행에 걸친 영역은 대조군으로 사용했다. #9451의 비공개 원본이 제공되면 동일 현상인지 확인할 수 있도록 이슈를 열어 둔다.

## 저장·재열기 검증

지정한 공개 HWPX에서 바깥 표의 행 주소만 이동시킨 입력을 사용했습니다. 기준 devel에서 이 입력을 내보내면 오류가 발생했습니다. 후보는 Native CLI와 fresh WASM에서 내보내기, 재열기, 재저장을 완료했습니다. `Contents/section0.xml`은 정상 대조군과 같았습니다. 빈 행 선언, 전체 열을 덮지 않는 한 행, 다른 행에 걸친 영역의 세 입력은 두 경로 모두 거부했습니다.

한컴 2020 11.0.0.9172 Print의 두 쪽에 대해 보정 입력과 정상 대조군의 PNG 해시가 각 경로에서 같았습니다. Native는 97.50%·99.37%, fresh WASM은 97.51%·98.71%였습니다. 인쇄 작업 `ff1f0d78-4dbc-4717-881c-bc0af9acb14c`의 PDF SHA-256은 `0dff936c01118cbe25d6bceeefc42f8d7b7c4b006bdcba576f37e6ef9c3abbd1`입니다.

[지정한 공개 HWPX](../../../samples/issue5884/nested_row_first_line_height.hwpx) · [한컴 2020 인쇄본](../../../pdf/issue5884/mcp-11.0.0.9172/nested_row_first_line_height-2020.pdf)

최신 통합 코드 후보로 정상 대조와 행 주소 이동 표본의 각 두 쪽을 다시 출력했습니다. 각 입력의 Native PNG와 TSV 해시는 앞선 후보와 같았습니다. 두 경로에서 검토 이미지를 구성한 현재 출력과 한컴 Print PNG 각각의 해시도 전쪽 TSV의 입력과 일치했습니다.

공개 HWPX의 `Contents/section0.xml`에서 표 ID `2005281692`의 `rowCnt`를 1에서 2로, 유일한 바깥 셀의 `rowAddr`를 0에서 1로 바꿨습니다. 공개 원본 SHA-256은 `c6bcf3686e8fbfbc79f5f3e2c690a62577e8ef63f0bad74e66ee3b255d76b931`이며, 변경 표본은 `38c451ccd014104c2a239a0551c2097e99c2c27d39d3df73806c93564da0cbb6`입니다. 두 쪽의 중첩 표와 이어지는 계산식 셀의 경계를 비교 이미지에서 확인했습니다.

| 쪽 | Native 비교 · 겹침 | fresh WASM 비교 · 겹침 |
| --- | --- | --- |
| 1 | [비교](../assets/issue_9451/public-row-address-20261128/native/review_001.png) · [겹침](../assets/issue_9451/public-row-address-20261128/native/overlay_001.png) | [비교](../assets/issue_9451/public-row-address-20261128/wasm/review_001.png) · [겹침](../assets/issue_9451/public-row-address-20261128/wasm/overlay_001.png) |
| 2 | [비교](../assets/issue_9451/public-row-address-20261128/native/review_002.png) · [겹침](../assets/issue_9451/public-row-address-20261128/native/overlay_002.png) | [비교](../assets/issue_9451/public-row-address-20261128/wasm/review_002.png) · [겹침](../assets/issue_9451/public-row-address-20261128/wasm/overlay_002.png) |

## Merge 후 contributor PR comment 계획

지정된 공개 중첩 표에서 행 주소 이동·저장·재열기와 잘못된 행의 거부를 확인했다고 안내합니다. 공개 표본 두 쪽의 최신 Native·WASM review·overlay를 연결합니다. [#9451](https://github.com/edwardkim/rhwp/issues/9451)은 비공개 원본의 동일 현상 확인을 위해 열린 상태로 둡니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
