# PR #9209 리뷰 — fix(layout): 나뉜 표 뒤 둘째 표의 쪽 예산과 칸 안 중첩 표·자리차지 그림 띠를 한컴 자리에 맞춘다 (#9038, #9045)

## 최종 판정

**메인터너 보정 후 수용 가능.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9209](https://github.com/edwardkim/rhwp/pull/9209) · planet6897 · devel
- 원 head `1789d4db6fa5d8275706176f43b9ec79cdf9e535` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `5d024cf72f70576657f2fc1f31974cedbe09859c` → `ec15aeff3c704b8c994e5d6bb48fe69ecbebe754`
  - `c1f817a0468bc163e0991a9b8b64852ab21b7df6` → `c6a6de5ca82731a76befc1d2ded0eb02425be0e4`
  - `f225cb61ac62806eb8ba8c332a9180c9373b2732` → `089117686b553869559f79c72a12c050cdfb8ddd`
  - `6b4040bd9b31447402e44324e13c14206622ca10` → `34e59aee20e4b760aa87ce2e2d6c84bd2a42ec37`
  - `1789d4db6fa5d8275706176f43b9ec79cdf9e535` → `42daeaad150913ef1a0ec93673c1f00a7493aa75`

## 원 PR 주장과 확인 범위

앞 표가 분할된 뒤 둘째 표의 쪽 예산, 중첩 표의 배치와 칸 안 그림 띠를 대조한다.

독립 Print와 후보는 모두 46쪽이다. 이전 devel의 45쪽 출력과는 먼저 실제 내용으로 대응했다. 메인터너는 셀에 저장된 종이 기준 그림을 소유 쪽에 배치해 43쪽 그림 두 개가 46쪽으로 함께 밀리던 현상을 보정했다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9038/sme_announcement_split_table_successor.hwpx` · SHA-256 `e0cb2372922ec5c89cc028e583b78a1e1d264385569f23cd0f2daf29a91f410b`
- 한컴 2020 11.0.0.9136 Hancom PDF Print · 46쪽
  `pdf/issue9038/sme_announcement_split_table_successor-2020.pdf` · SHA-256 `434e59c3c7bba89b6a13a06922c0d777494331c8557002a0cc070677dfce1d41`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 46 | 98.37% (43쪽) | 0 |
| fresh WASM | 46 | 98.36% (43쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9209/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/silhouette.tsv)

44쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9209/native/raw-044.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9209/native/pdf-044.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_044.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_044.png)
- 23쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_023.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_023.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_023.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_023.png)
- 43쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_043.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_043.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_043.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_043.png)
- 44쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_044.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_044.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_044.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_044.png)
- 45쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_045.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_045.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_045.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_045.png)

## cb85758e에서 확인한 쪽 경계

Native 고정 기준은 45쪽, 최종 렌더러 후보와 한컴 Print는 46쪽이다. 22쪽까지는 같은 쪽을 대조하며, 23~25쪽에서는 뒤 표의 배치 및 앞 표 이어받기 순서를 판독했다. 후보 26쪽부터는 한 쪽 앞선 기준 페이지의 내용과 대응한다. 쪽 번호만 맞춘 실루엣 하락폭에 이 구간을 포함하지 않고 별도로 재검토한다. 후보 23·24·25쪽의 한컴 Print 대비 공식 표시값은 각각 99.89%, 100.00%, 100.00%였다. 표 분할 구간 밖의 대응 43쪽은 쪽 번호를 제외한 render tree 텍스트가 모두 같았으며 2%p를 초과해 하락한 쪽은 없다. [쪽별 대조 JSON](../assets/planet6897_9208_9236_20261121/pagination/pagination-9209-native-aligned.json) · [24쪽 review](../assets/planet6897_9208_9236_20261121/pagination/pagination-9209-native-split-pairs/review/review_024.png) · [24쪽 overlay](../assets/planet6897_9208_9236_20261121/pagination/pagination-9209-native-split-pairs/overlay/overlay_024.png).

fresh WASM에서도 후보와 Print가 46쪽이었다. 분할 구간 23·24·25쪽은 99.89%, 100.00%, 100.00%이며, 그 밖의 대응 43쪽은 쪽 번호를 제외한 render tree 텍스트가 모두 같았다. 대응 쪽에서 2%p를 초과해 하락한 곳은 없다. [쪽별 대조 JSON](../assets/planet6897_9208_9236_20261121/pagination/pagination-9209-wasm-aligned.json).

[쪽 분할 검토 종합](../assets/planet6897_9208_9236_20261121/pagination/pagination-review-disposition.json).

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 로컬 제품 검증 결과를 기록했다. [통합 PR #9283](https://github.com/edwardkim/rhwp/pull/9283)의 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/44881574506)와 [연계 검사](../assets/planet6897_9208_9236_20261121/integration-code-ci.json)를 통과했다.

메인터너 보정 커밋은 [`6b905d0697`](https://github.com/edwardkim/rhwp/commit/6b905d0697d013655d47c29617ba29a432e12c37)다.

## Merge 후 contributor PR comment 계획

나뉜 표의 다음 조각과 둘째 표의 쪽 경계를 확인했습니다. 43쪽 그림 두 개가 46쪽으로 밀리는 문제는 칸에 저장된 종이 기준과 소유 쪽을 유지해 보정했습니다.

해당 문서의 데스크톱 한컴 Hancom PDF Print와 최신 후보의 Native·fresh WASM 비교를 기록한다. 통합 병합 SHA에서 다음 쪽의 review·overlay 바이트를 확인한 뒤 원 PR에 표시한다.

- Native · 23쪽 99.89% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_023.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_023.png)), 43쪽 98.37% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_043.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_043.png)), 44쪽 99.96% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_044.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_044.png)), 45쪽 98.43% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/native/review/review_045.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/native/overlay/overlay_045.png)).
- fresh WASM · 23쪽 99.89% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_023.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_023.png)), 43쪽 98.36% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_043.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_043.png)), 44쪽 99.96% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_044.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_044.png)), 45쪽 98.42% ([review](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/review/review_045.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9209/wasm/overlay/overlay_045.png)).
