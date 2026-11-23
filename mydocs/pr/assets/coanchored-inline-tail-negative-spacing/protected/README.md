# 공통 보호 입력과 편람 검증

실행 소스는 `af003f4001a8c5a91d47685d28be5acf600460e8`이다. Native Skia와 fresh 최적화 WASM을 같은 소스에서 빌드했다.

편람 HWPX의 Native 383쪽이 정본의 쪽수와 일치했고, 쪽별 비교에서 하락이 없었다. [전체 TSV](handbook-native/silhouette.tsv)

기여자 공통 보호 검사에서 4개 공개 원본을 각 backend로 렌더링했다. 8개 실행이 모두 통과했다.

| 입력 ID | Backend | 쪽 | Raw 실루엣 | 증적 |
| --- | --- | ---: | ---: | --- |
| `52ee61fe7201e36b` | native | 1 | 100.00% | [TSV](contributor/52ee61fe7201e36b/native/silhouette.tsv) · [비교](contributor/52ee61fe7201e36b/native/review.png) · [오버레이](contributor/52ee61fe7201e36b/native/overlay.png) |
| `52ee61fe7201e36b` | wasm | 1 | 100.00% | [TSV](contributor/52ee61fe7201e36b/wasm/silhouette.tsv) · [비교](contributor/52ee61fe7201e36b/wasm/review.png) · [오버레이](contributor/52ee61fe7201e36b/wasm/overlay.png) |
| `59f6ab1155623bda` | native | 1 | 100.00% | [TSV](contributor/59f6ab1155623bda/native/silhouette.tsv) · [비교](contributor/59f6ab1155623bda/native/review.png) · [오버레이](contributor/59f6ab1155623bda/native/overlay.png) |
| `59f6ab1155623bda` | wasm | 1 | 100.00% | [TSV](contributor/59f6ab1155623bda/wasm/silhouette.tsv) · [비교](contributor/59f6ab1155623bda/wasm/review.png) · [오버레이](contributor/59f6ab1155623bda/wasm/overlay.png) |
| `cdacd874b6e6ac66` | native | 1 | 100.00% | [TSV](contributor/cdacd874b6e6ac66/native/silhouette.tsv) · [비교](contributor/cdacd874b6e6ac66/native/review.png) · [오버레이](contributor/cdacd874b6e6ac66/native/overlay.png) |
| `cdacd874b6e6ac66` | wasm | 1 | 100.00% | [TSV](contributor/cdacd874b6e6ac66/wasm/silhouette.tsv) · [비교](contributor/cdacd874b6e6ac66/wasm/review.png) · [오버레이](contributor/cdacd874b6e6ac66/wasm/overlay.png) |
| `d03a9dde3d5dc7ac` | native | 1 | 100.00% | [TSV](contributor/d03a9dde3d5dc7ac/native/silhouette.tsv) · [비교](contributor/d03a9dde3d5dc7ac/native/review.png) · [오버레이](contributor/d03a9dde3d5dc7ac/native/overlay.png) |
| `d03a9dde3d5dc7ac` | wasm | 1 | 100.00% | [TSV](contributor/d03a9dde3d5dc7ac/wasm/silhouette.tsv) · [비교](contributor/d03a9dde3d5dc7ac/wasm/review.png) · [오버레이](contributor/d03a9dde3d5dc7ac/wasm/overlay.png) |
