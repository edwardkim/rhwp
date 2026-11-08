# PUA source/display JSON 계약 재검증

기준 code head `d00a7b02f529d428fb16c5346f2374402c398f86`의 Native393/fresh WASM394 실제 PNG를 전쪽 다시 비교했다. 테스트 모듈 변경 전 검증이며 후속 변경은 `src/paint/json.rs`의 기존 `#[cfg(test)]` 모듈만이다. 생산 코드 접두부와 후속 테스트 모듈은 동일하다.

canonical PDFium smooth_rgb_v1 / 192DPI / compat2022 / raw 2px /90% 문턱. 원본 두 문서는 각1쪽이며 Native/WASM 네 조건 쪽수 일치·미달0이다.

| 입력 | Native raw | fresh WASM raw |
| --- | --- | --- |
| issue7017 출품 접수증 | 99.35164% | 99.37179% |
| 복학원서 | 96.92177% | 97.68584% |

절취선·표·본문·서명·접수증 순서를 직접 review/overlay로 대조했다. 복학원서의 PDF에 없는 상단 로고, 서명 및 글자 래스터 차이와 하단 채움 획은 남는다. 복학원서는98% px 회귀 선행 조건을 충족하지 않는다. 점수 통과는 RGB 동일이나 전체 PR 완료를 뜻하지 않는다.

Native393 CLI `4a2f63a7423c3f0c87c715af569a9a9f4788ca3386cd7bca1446a94bbf9bc0a0`, fresh WASM394 `174ef06e842525682c71e2368c755f327890cd9899964c16ddef6effd4aa4aa2`. locked wrapper web/pkg/no-opt Mac 로컬 대체 빌드. 원본·PDF·글꼴·생산 hash 원문은 ignored `repair408-pua-current-whole/producer.json`에 보존한다.

아래 최종 TSV·review·overlay는 새408 산출물의 바이트 그대로이며 SHA-256으로 복사 일치를 확인했다.

| 파일 | SHA-256 |
| --- | --- |
| [native-receipt7017-silhouette.tsv](native-receipt7017-silhouette.tsv) | `7579789c6f2d41760f17a50173b0e72080d1c4693a98fbcacb850b0819d2c1b6` |
| [native-receipt7017-review_001.png](native-receipt7017-review_001.png) | `8901de772f6025832ea7ea053a1487d3989d2ab8d233635a59fa5b76656dd059` |
| [native-receipt7017-overlay_001.png](native-receipt7017-overlay_001.png) | `c656eb54adec6dcc3c617b665b5e2c2dd3a91f3dfadadac91a9bd7967f8b29ac` |
| [native-return937-silhouette.tsv](native-return937-silhouette.tsv) | `cbde07b171dc7eb18de85634f96771228d74805ef6a23542f88a4fa6a52c0463` |
| [native-return937-review_001.png](native-return937-review_001.png) | `727b748682cc8a5e851f7381793f6b0d2b83dcc6e68c47cda7c8f7f021e552b6` |
| [native-return937-overlay_001.png](native-return937-overlay_001.png) | `c1c0425538a8c487ea225a3b9f4a6ae61119c0f137d4243196b507d29c42295f` |
| [wasm-receipt7017-silhouette.tsv](wasm-receipt7017-silhouette.tsv) | `0c3acf6192dcaa09b173277ba8cc6519e75687e396b03a3e8a8cf8ba353f4c10` |
| [wasm-receipt7017-review_001.png](wasm-receipt7017-review_001.png) | `1ccba9f42c593e2d2038024465da79c176561bfa87a39aae1be112b58cb18a87` |
| [wasm-receipt7017-overlay_001.png](wasm-receipt7017-overlay_001.png) | `cb44c01cdd0e69a41d3ebc1c0576ffd5e91091df47767e81072b8b1b02aa97b1` |
| [wasm-return937-silhouette.tsv](wasm-return937-silhouette.tsv) | `494bbcb21dba686e7a2031048afa1371e345a4ec2ee4dc308ade2e53c1cddc7f` |
| [wasm-return937-review_001.png](wasm-return937-review_001.png) | `0fe74eb4a1b2e5725ea7e6008f5f1eb4ef054877dd559537e01d7b393d18880c` |
| [wasm-return937-overlay_001.png](wasm-return937-overlay_001.png) | `e41d5f7e97289f0c6ba1ce4f105c2684272ad05dd3c36e5dfcc813c540cb0322` |
