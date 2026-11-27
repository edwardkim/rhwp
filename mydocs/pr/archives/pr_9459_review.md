# PR #9459 검토

[planet6897 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9459)

## 검토 버전

- 원 PR head `a6633bff4e70d1137d67fcd77c2a13394cb9977d`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `12d2c7e0a132c860d1668fdcdf5b9d19f5b4a65f` | `2ef52fce6b8eb763903c616620bbcc6428306da4` |
| `a6633bff4e70d1137d67fcd77c2a13394cb9977d` | `fe662f8c960fdfa17eaca0bf975937184111ff48` |

## 구현 범위

여러 쪽으로 분할된 빈 호스트의 자리 차지 RowBreak 표가 마지막 조각을 출력한 뒤, 저장 줄 위치의 차이가 호스트 줄 높이와 줄 간격의 합일 때 다음 문단 앞의 줄을 한 번 소비한다. 조판 예약과 배치가 같은 함수 결과를 사용한다. 실제 공백이 있는 이어받기 조각과 물리 사다리 대조군을 확인한다. 원 PR TSV 세 파일의 줄 끝을 LF로 맞추고 모든 페이지의 값이 같은지 확인했다.

## 전쪽 검증

기존 한컴 2022 Print에 대한 4쪽은 Native 86.78% → 96.22%, fresh WASM 86.63% → 96.11%였습니다. 원 PR에서 제출한 4쪽 값은 각각 95.67%, 95.97%였습니다. 원 PR의 Native·WASM 제출값과 전쪽을 대조해 하락이 없었습니다. 1~3쪽은 기준 devel과 PNG 해시가 모두 같았습니다.

한컴 2020 11.0.0.9172의 Hancom PDF 한 쪽씩 인쇄본으로도 4쪽을 비교했습니다. Native 86.79% → 96.21%, fresh WASM 86.63% → 96.11%였습니다. 인쇄 작업은 `84e9f822-884f-4038-b7f8-f210bd4158e1`입니다. 기준 PDF SHA-256은 `036f96aead9a7daaca4511919147654876a34e666d52f32c106cdc872758edc9`입니다.

[한컴 2020 인쇄본](../../../pdf/issue687/gochang_wetland_final_report-2020.pdf) · [원 PR 전쪽 결과](../assets/issue_687/)

최신 통합 코드 후보로 고창습지의 한컴 2020·2022 인쇄본을 다시 대조했습니다. 두 기준 모두 네 쪽의 Native PNG와 TSV 해시가 앞선 후보와 일치했습니다. 새 한컴 2020 인쇄본과 최신 WASM의 네 쪽도 다시 비교했습니다. 원본, PDF, 52개 공급 글꼴을 해시로 확인했으며 이전 비교와 출력·기준 PNG 및 전쪽 TSV가 일치했습니다.

## 마지막 표 다음 문단 직접 판독

한컴 2020 Print와 최신 Native·fresh WASM의 4쪽을 나란히 확인했습니다. 기준 devel에서는 `향후 계획` 제목이 마지막 표의 하단 괘선에 붙어 있었고, 최신 후보에서는 표 아래에 기준 Print와 같은 간격이 생겼습니다. 제목과 다음 항목 및 붙임 문구가 같은 쪽에 순서대로 남았습니다. 표의 마지막 셀 내용과 괘선도 유지됐습니다.

| 기준 | Native 비교 · 겹침 | fresh WASM 비교 · 겹침 |
| --- | --- | --- |
| 한컴 2020 4쪽 | [비교](../assets/issue_687/hancom2020-20261128/native/review_004.png) · [겹침](../assets/issue_687/hancom2020-20261128/native/overlay_004.png) | [비교](../assets/issue_687/hancom2020-20261128/wasm/review_004.png) · [겹침](../assets/issue_687/hancom2020-20261128/wasm/overlay_004.png) |
| 한컴 2022 4쪽 | [비교](../assets/issue_687/native/review_p004.png) · [겹침](../assets/issue_687/native/overlay_p004.png) | [비교](../assets/issue_687/wasm/review_p004.png) · [겹침](../assets/issue_687/wasm/overlay_p004.png) |

## Merge 후 contributor PR comment 계획

고창습지 4쪽의 마지막 표와 `향후 계획` 사이 간격이 복원된 결과와 1~3쪽 이미지 동일성을 안내합니다. 4쪽의 한컴 2022 원 PR 증적과 새 한컴 2020 Print의 Native·fresh WASM 비교·겹침을 연결합니다. TSV의 LF 보정과 모든 값의 동일성을 설명합니다. 관련 이슈는 [#9458](https://github.com/edwardkim/rhwp/issues/9458)입니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
