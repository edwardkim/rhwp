# PR #9414 검토

[hyunmyungJaneLee 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9414)

## 검토 버전

- 원 PR head `dbb281275371765eeaf4e0996b6cbe1c293fd94f`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `e12085f2477395d2bd324eaf8b351d6f0868835b` | `7a259a44aad0a6dad00be7d267e4eeee75f16fa9` |
| `fbac22ef141074e2d25e1bb4fc6ccc130329714e` | `3b585eba067b06fb3e504cc07e686cf22ad02938` |
| `27b9333a6227f8afbb42c71b15ae165e70adcba3` | `cb01ab55e2cf4fc7a788884803f1dbf6d3f06e24` |
| `dbb281275371765eeaf4e0996b6cbe1c293fd94f` | `03aebb0e50237d2727ce621527025afee42171b1` |

## 구현 범위

자동 값축의 세로 눈금 수는 그림 영역 높이와 글줄 높이로 정하고, 양수 자료가 좁게 모인 가로 축은 실제 눈금 글자의 폭을 사용한다. 아래쪽 눈금과 끝 눈금은 선택한 단위로 함께 계산한다. 원 PR은 한컴 2020에서 인쇄한 두 재현물의 차트 30개와 기존 재현물 네 개로 동작을 확인했다. 검토 범위는 #8678의 남은 자동 값축 동작이다.

## 한컴 2020 Print 대조

두 재현물의 Native 전쪽 11행이 원 PR 제출값과 일치했습니다. 차트를 고친 4쪽(set1)은 88.68%에서 99.92%, 2쪽(set2)은 80.92%에서 99.52%로 올랐습니다. 원본 HWPX의 차트 30개가 모두 함초롬돋움을 지정했습니다. 두 TTF 버전을 각각 공급했을 때 최신 WASM의 쪽별 값은 그대로였습니다. 최신 WASM은 통합 후보의 이전 비교에서 생성한 11쪽과 PNG 해시가 같았습니다. 원 PR의 제출 WASM TSV보다 이 Mac에서 측정한 값이 0.14~0.48%p 낮았습니다. 원본 head를 같은 Mac에서 별도로 빌드해 원본 HWPX·Print PDF·글꼴 해시, 호환 모드, 비교 프로필 및 래스터 조건을 맞췄습니다. 원본 head와 통합 후보는 WASM 전쪽 11행의 점수와 출력 PNG 해시가 모두 같았습니다. 이 조건에서 통합에 따른 출력 하락은 0쪽입니다. [같은 조건의 WASM 쪽별 결과](../assets/integration-8678-20261128/source-9414-wasm-page-comparison.tsv)

[원본 Print와 측정 자료](../assets/issue_8678_residual/)

최신 통합 코드 후보의 Native 실행 파일에서 두 차트 표본 11쪽의 PNG와 TSV 해시를 대조했습니다. 앞선 Native 결과와 전부 일치했습니다.

## 차트 값축 직접 판독

최신 Native와 바이트 동일성이 확인된 fresh WASM을 한컴 2020 Print와 직접 대조했습니다. set1 4쪽의 세 차트는 값축 75~100의 5 간격, 0~4의 1 간격, 0~3.5의 0.5 간격을 같은 순서로 표시했습니다. set2 2쪽의 세 차트는 8.6~10의 0.2 간격, 0~7의 1 간격, 0~6의 2 간격이 일치했습니다. 막대 개수와 범례 위치도 대조했습니다.

현재 코드 후보의 두 대표 쪽은 Native 99.92%·99.52%, fresh WASM 98.52%·98.54%였습니다. 검토 이미지를 구성한 WASM과 한컴 Print PNG는 전쪽 비교에 사용한 PNG와 각각 일치합니다.

| 표본과 쪽 | Native 비교 · 겹침 | fresh WASM 비교 · 겹침 |
| --- | --- | --- |
| set1 4쪽 | [비교](../assets/issue_8678_residual/axis_residual_set1_hwpx-native-review.png) · [겹침](../assets/issue_8678_residual/axis_residual_set1_hwpx-native-overlay.png) | [비교](../assets/issue_8678_residual/integration-20261128/wasm/axis_residual_set1/review_004.png) · [겹침](../assets/issue_8678_residual/integration-20261128/wasm/axis_residual_set1/overlay_004.png) |
| set2 2쪽 | [비교](../assets/issue_8678_residual/axis_residual_set2_hwpx-native-review.png) · [겹침](../assets/issue_8678_residual/axis_residual_set2_hwpx-native-overlay.png) | [비교](../assets/issue_8678_residual/integration-20261128/wasm/axis_residual_set2/review_002.png) · [겹침](../assets/issue_8678_residual/integration-20261128/wasm/axis_residual_set2/overlay_002.png) |

## 관측 범위와 원 PR CI

자동 눈금의 1.5줄 간격, 3.3칸 경계와 가로 1.25배 간격은 데스크톱 한컴 2020 Print에서 확인한 값입니다. 10pt 표본과 공개 차트에서 검증했습니다. 283px보다 높은 차트·다른 크기의 글꼴·누적 및 콤보 차트의 독립 한컴 대조는 원 PR의 주장 범위에 포함되지 않습니다. 막대 띠 위치는 별도 동작입니다.

원 PR의 고정 head에서 이전에 실패했던 Rust CodeQL은 이후 성공했습니다. [원 PR CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45649023296)

## Merge 후 contributor PR comment 계획

두 차트 표본의 값축 범위·간격과 막대 개수를 직접 대조한 결과를 안내합니다. set1 4쪽과 set2 2쪽은 한컴 2020 Print의 대응 review·overlay를 연결합니다. 원본 head의 같은 조건 재현에서 통합 후보와 전쪽 점수·PNG가 같았음을 안내합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
