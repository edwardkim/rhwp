# PR #9188 리뷰 — fix(font): HCI Tulip 보통체 영문 전진폭을 독립 한컴 Print Type3 폭으로 보강

## 최종 판정

**통합 승인.** [통합 PR #9195](https://github.com/edwardkim/rhwp/pull/9195)의 코드 head `bd20cae04547db2053a541d259748eff6dc898e0`에서 필수 CI와 개별 주장 검증을 통과했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9188](https://github.com/edwardkim/rhwp/pull/9188) · `planet6897` |
| 원 head | `c372fce6ecaac4a1c73e20cd66944fb7b437f7b9` |
| 적용 커밋 | `305f8d98b175` |
| 시각 검증 base·측정 head | `af53659e19f70c18d0d3591a719c238235007a05` · `a84c52f4dcade0a068050226851fe61ba550a813` |
| 원 PR 검토 담당 | `jangster77` |

## 변경과 검토 범위

원 PR은 HCI Tulip 영문 전진폭을 복원한다. 통합 검토에서 7쪽의 큰 차이가 상단 BMP 그래프의 원본 자르기 기준에서도 발생함을 확인했다. 밀도·imgDim 없는 BMP의 실제 픽셀 전체 범위와 저장 자르기 창을 공통 paint에서 보존했다.

## 메인터너 보정 사유와 범위

원 PR이 한컴 Print Type3의 폭으로 HCI Tulip 영문 전진을 보정했다. 그림 3-1의 가로 배율 차이는 DPI 정보가 없어 원 PR에서 보류한 범위였다. 사용자 요청에 따라 16쪽 전체를 대조하면서 7쪽의 남은 차이를 좁혔다. 원본 BMP 헤더는 758×527px를 기록하고, 저장된 자르기 창은 별도로 존재했다. 렌더 경로가 창의 끝 좌표를 원본 이미지 전체 크기로 사용해 그림의 가로 자르기와 배율이 어긋났다.

밀도와 저장 원본 크기가 비어 있는 BMP에서 실제 픽셀 크기를 이미지 전체 범위로 사용하고, 자르기 창은 독립해서 적용하도록 공용 paint를 수정했다. 원 PR의 영문 폭을 보존하면서 같은 유형의 BMP를 검사하는 회귀를 추가했다. 7쪽 두 경로가 99.92%로 올랐고 16쪽 모두 99.57% 이상이다. [수정 커밋](https://github.com/edwardkim/rhwp/commit/89dc6fb3ac) · [회귀 커밋](https://github.com/edwardkim/rhwp/commit/afd5ce3474) · [원본 BMP 측정](../assets/open9_integration_20261119/9188/bitmap-source-geometry.json)

통합에서 Windows CRLF TSV 3개의 줄바꿈도 LF로 정리했다. 쪽별 값은 모두 원본과 같다. [정리 커밋](https://github.com/edwardkim/rhwp/commit/f38ba519e0)

## 검증 입력과 결과

검증한 통합 head는 `a84c52f4dcade0a068050226851fe61ba550a813`이며 고정 devel은 `af53659e19f70c18d0d3591a719c238235007a05`다. 통합 최소 글꼴 전체 nextest는 13,816건 통과, 실패 0건, 건너뜀 45건이다.

한컴 2022 Print와 같은 16쪽이다. 7쪽 Native 90.91→99.92%, fresh WASM 90.87→99.92%다. 양쪽 최저는 스캔 그림 차이가 남은 15쪽 99.57%다. 새 원본 자르기 관계 검사와 7쪽 영문 전진폭 검사가 통과했다.

현재 소비 경로의 CanvasKit 준비는 한양신명조 미준비를 감지해 Canvas2D로 전환했다. 두 Print Sweep의 16쪽 검증과 출력 경로 진단을 구분한다.

## 시각 증적과 검증 기록

[검증 소스와 리베이스 확인](../assets/open9_integration_20261119/validation.json) · [원 PR 커밋 대응표](../assets/open9_integration_20261119/source-commits.json)

[주장 범위 회귀 실행](../assets/open9_integration_20261119/9188/regression.json)

Native 7쪽 [raw PNG](../assets/open9_integration_20261119/9188/tulip/native/raw-007.png) · [한컴 PDF PNG](../assets/open9_integration_20261119/9188/tulip/native/pdf-007.png) · [review](../assets/open9_integration_20261119/9188/tulip/native/review-007.png) · [overlay](../assets/open9_integration_20261119/9188/tulip/native/overlay-007.png) · [전쪽 TSV](../assets/open9_integration_20261119/9188/tulip/native/silhouette.tsv)

fresh WASM 7쪽 [raw PNG](../assets/open9_integration_20261119/9188/tulip/wasm/raw-007.png) · [한컴 PDF PNG](../assets/open9_integration_20261119/9188/tulip/wasm/pdf-007.png) · [review](../assets/open9_integration_20261119/9188/tulip/wasm/review-007.png) · [overlay](../assets/open9_integration_20261119/9188/tulip/wasm/overlay-007.png) · [전쪽 TSV](../assets/open9_integration_20261119/9188/tulip/wasm/silhouette.tsv)

[원본 BMP 좌표](../assets/open9_integration_20261119/9188/bitmap-source-geometry.json)

## 독립 Print 입력

[tulip 원본](../../../samples/issue677/1480000-200900112-hci-tulip-table.hwp) · [한컴 Print](../../../pdf/issue677/1480000-200900112-hci-tulip-table-2022.pdf)

## 통합 CI

검증 head `bd20cae04547db2053a541d259748eff6dc898e0`. [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44509238552) · [CI](https://github.com/edwardkim/rhwp/actions/runs/44509238545) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44509238349) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44509238197) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44509238311) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44509238321)

## Merge 후 contributor PR comment 계획

원 PR의 head와 통합 PR #9195의 병합 SHA·코드 CI를 확인하고 아래 문안을 사용한다. 병합 SHA의 asset 바이트를 대조한 뒤 `--body-file`로 게시하고 API의 본문과 렌더링된 이미지를 확인한다.

> HCI Tulip 보통체 영문 전진폭을 한컴 Print의 Type3 폭과 대조해 주셔서 감사합니다. 원 PR에서 보류하신 7쪽 그림 배율을 독립 Print와 다시 대조했습니다. BMP 헤더에는 758×527px가 있고 저장 자르기 창은 별개인데, 기존 출력이 자르기 끝 좌표를 이미지 전체 크기에 사용하고 있었습니다. 이 차이를 해결하기 위해 원본 이미지 범위와 자르기 창을 분리하는 메인터너 보정과 회귀 검사를 추가했습니다. 데스크톱 한컴 2022 Hancom PDF Print와 같은 16쪽이며 7쪽 Native·fresh WASM 모두 99.92%, 전쪽 최저는 15쪽 99.57%입니다. 15쪽에는 스캔 그림 경계 차이가 남습니다.

[Visual Sweep 병합 코멘트 지침](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

| 입력·쪽 | 출력 | raw·독립 Print | review·overlay |
| --- | --- | --- | --- |
| tulip 7쪽 | native | [raw](../assets/open9_integration_20261119/9188/tulip/native/raw-007.png) · [pdf](../assets/open9_integration_20261119/9188/tulip/native/pdf-007.png) | [review](../assets/open9_integration_20261119/9188/tulip/native/review-007.png) · [overlay](../assets/open9_integration_20261119/9188/tulip/native/overlay-007.png) |
| tulip 7쪽 | wasm | [raw](../assets/open9_integration_20261119/9188/tulip/wasm/raw-007.png) · [pdf](../assets/open9_integration_20261119/9188/tulip/wasm/pdf-007.png) | [review](../assets/open9_integration_20261119/9188/tulip/wasm/review-007.png) · [overlay](../assets/open9_integration_20261119/9188/tulip/wasm/overlay-007.png) |
| tulip 15쪽 | native | [raw](../assets/open9_integration_20261119/9188/tulip/native/raw-015.png) · [pdf](../assets/open9_integration_20261119/9188/tulip/native/pdf-015.png) | [review](../assets/open9_integration_20261119/9188/tulip/native/review-015.png) · [overlay](../assets/open9_integration_20261119/9188/tulip/native/overlay-015.png) |
| tulip 15쪽 | wasm | [raw](../assets/open9_integration_20261119/9188/tulip/wasm/raw-015.png) · [pdf](../assets/open9_integration_20261119/9188/tulip/wasm/pdf-015.png) | [review](../assets/open9_integration_20261119/9188/tulip/wasm/review-015.png) · [overlay](../assets/open9_integration_20261119/9188/tulip/wasm/overlay-015.png) |
