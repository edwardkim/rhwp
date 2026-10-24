# 개체 번호 소유 회귀 입력

원본 `samples/issue_265.hwp`의 그림·저장 줄 정보를 유지한 대조군이다.
입력 생성기·변환 요청/응답·독립 Print PDF·로그는 ignored output에 보존한다.
PDF는 engine2020/Hancom11.0.0.9136, Print method0, one-up, preprocess none이며 모두16쪽이다.

|입력|SHA256|독립 Print PDF|PDF SHA256|
|---|---|---|---|
|[old-none-hwp.hwp](old-none-hwp.hwp)|`884cebac56260ba7ac53091504dc6be9a646681888694969a559cb47e2dfc3c5`|[issue265-caption-flow-hwp-save-20261020-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/old-none-hwp-2020.pdf)|`f13729e187dd000760ca8ee36b3b2cb3f6bac21d98e4c6acc670758c010f2ff1`|
|[old-none-hwpx.hwpx](old-none-hwpx.hwpx)|`c1d57147e6d7cb22317ad55ea12eed426699d62b46b0a74e88283e6eac09e1e9`|[issue265-caption-flow-hwpx-save-20261020-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/old-none-hwpx-2020.pdf)|`ed5126b5b7dfaceb2c904694a31ad0e207229ca0b4887853c907623fd40bd919`|
|[inline-unbound.hwpx](inline-unbound.hwpx)|`41d23167b4c2d546ac4dc52c7361e9589c4fd2170a7a1de1a82023e2f41bc8f9`|[inline-unbound-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/inline-unbound-2020.pdf)|`b69a71e6fcc5150863475a11682aa8ac4ab564acd952a2cd0752e237ce969e22`|
|[all-picture.hwpx](all-picture.hwpx)|`8ad8665945e45a1f1eacd70c540040b7de4019b6752ac43eb92f15349667efc4`|[all-picture-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/all-picture-2020.pdf)|`16e565fab166cc3726ab1e27cb7feaabf6f63ee363c9ce1c79b1937cd7292712`|
|[mixed-picture-none.hwpx](mixed-picture-none.hwpx)|`557ae42786f5331aab387cf5f7676bffc55e0783e1e44d962ba4c99a3175b6aa`|[mixed-picture-none-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/mixed-picture-none-2020.pdf)|`91ab502240acd69034f7db574e4d2319e51eb14e3059bddef3b77525a4926894`|
|[duplicate-field-owner.hwpx](duplicate-field-owner.hwpx)|`02d9f35d301e3a1239e1de2b3d1549a9de9dee0f8c26e9fc58750142015d8e75`|[duplicate-field-owner-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/duplicate-field-owner-2020.pdf)|`92e545d61eb8618ac5003f6ad72c3102b2fde97fb245bebf595a8550fe902bc8`|
|[none-start7.hwpx](none-start7.hwpx)|`6905f7e56dfaab821c79c49d1dcd10d96607444fa3862c64d841765807a546a5`|[none-start7-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/none-start7-2020.pdf)|`c9268270d8f86c74508d423825e60f2c2c5ee4a884925e52b0a7391b76c9b3de`|
|[picture-start7.hwpx](picture-start7.hwpx)|`cf0ec693c7509eed2d45546cef11b348100ba5c772ef15ee79545a9e7f554b58`|[picture-start7-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/picture-start7-2020.pdf)|`23dd06992fb1cf0c39712fc7c1091e791942a9e4318ab11cd165b5d1a1cce4a9`|
|[cross-caption-category.hwpx](cross-caption-category.hwpx)|`6cceca506261994ee28ebcbc9466ee72fa24f42af0d808c84ab7baa87a8a9b98`|[cross-caption-category-2020.pdf](/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/reference/cross-caption-category-2020.pdf)|`539941dfcb23ef5330e969a7e794cb6a1b8a75fe4a2f97af2fc671da8b458843`|

## 독립 기대값

- old-none 두 저장본과 none-start7: 네 캡션 모두1. 원본 저장 캐시1/2/3/4는 보존한다.
- all-picture: 캡션 없는 꼬리말 그림도 개체 순번에 포함하여2/3/4/5.
- mixed-picture-none: 범주가 없는 개체는1, 나머지는 그림 순번2/3으로1/2/1/3.
- duplicate-field-owner: 같은 캡션의 두 필드는 같은 개체 번호2/2. 뒤는3/4/5.
- cross-caption-category: 그림 캡션 안 TABLE 필드도 그 소유 그림 번호2를 표시한다.
- picture-start7: 캡션 없는 첫 그림7 이후8/9/10/11.
- inline-unbound: 기존 빈 본문 두 문단만 변경하고 저장 줄·문단 수를 유지했다.
  본문 `Before:1/1`, `After:5/5`; 본문 필드가 그림 순번을 소비하지 않는다.
  원본 캐시99가 아니라 현재 개체 번호를 표시한다.
- 원본 HWP3→공개 HWP/HWPX 저장·재열기는2/3/4/5를 유지한다.

## 시각 선행 증거

[12입력 ×16쪽 ×Native/fresh WASM 전체 결과](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair611-owned-number-slot-whole/results.json).
원본 최저95.34478/95.33483%, 다른11입력 최저99.02719/98.92066%,90% 미달0쪽이다.
canonical PDFium smooth_rgb_v1/192DPI/compat2022/raw2px. 입력·PDF·전체 글꼴 SHA와
producer의 실제 소스는 같은 폴더 producer.json/source ledger로 추적한다.
`df52e39848f65fccb973e7bf846bb6396a33df23` 위7개 소스 변경을 빌드한 Native607/fresh610 결과이며,
이 부모 SHA만의 결과로 주장하지 않는다.

본문1쪽 번호 두 개의 독립 CSS x는162.9516805013021/188.2179972330729다.
1쪽 Native99.03275%, fresh WASM98.92675%, review/overlay 직접 판독 후 ±2CSSpx를 허용한다.
동일번호·원문 placeholder 소유·필드 두 개·저장 왕복 관계 검사를 함께 유지한다.
원본16쪽 전체의 98%를 주장하지 않는다. 이 파일들은 글꼴 바이너리를 포함하지 않는다.
