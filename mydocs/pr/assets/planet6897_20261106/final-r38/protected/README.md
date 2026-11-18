# 리베이스 전 R38의 PR #8389·#8397 전쪽 보호 검증

실제 완료한 R38 producer의 측정 원본이다. 이후 R41 리베이스 후보를 실행한 결과로 표시하지 않는다.

- 제품 source: `09ada34bf1be3a3dcb60e36f2de016e8d35cfcd9`
- 검증 source: `7f88708f08d14d47537ef34578c969dc057ee6e8`
- Native producer SHA-256: `3941b00db042b95fdb348566f7e0dcabfdb25d40a21d7ebeca06f8dd480d2da0`
- 최적화 WASM SHA-256: `633f8216abc0038d18ee10e9cf5875113cd60d5349fbc1ed79c548dccccc8935`
- WASM glue SHA-256: `0099b8a6e00f750e1dbddb3a89bf42874f24effcf8bf4dc5eb328d69e691606b`
- 96 DPI, Print profile, 각 원본의 compat·독립 한컴 Print, 색상 경계 보정 전 2px 실루엣.
- 시각 측정은 전체 시스템 글꼴과 기록한 기존 대체 설정을 사용했다. 최소 공개 글꼴 Rust 회귀 환경과 구분하며 글꼴 프로그램을 배포하지 않는다.
- 양쪽 각각 17개 입력·1770쪽을 완료했다. 원 PR의 중복 기준까지 포함한 각 2153행을 대조해 표시값 하락·누락이 모두 0건이다.
- [전체 입력·PDF·TSV·환경 SHA](inventory.tsv), [Native 기준 대조 2153행](native-historical-reference-comparison.tsv), [WASM 기준 대조 2153행](wasm-historical-reference-comparison.tsv).

| 문서 | 쪽수 | Native 최저 raw | fresh WASM 최저 raw | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| `sample16-hwp2022` | 64 | 100.00% | 100.00% | [Native](native/sample16-hwp2022/full.tsv) · [WASM](wasm/sample16-hwp2022/full.tsv) |
| `sample16-hwp2024` | 64 | 99.99% | 99.99% | [Native](native/sample16-hwp2024/full.tsv) · [WASM](wasm/sample16-hwp2024/full.tsv) |
| `sample16-hwpx` | 64 | 99.99% | 99.99% | [Native](native/sample16-hwpx/full.tsv) · [WASM](wasm/sample16-hwpx/full.tsv) |
| `issue7905` | 6 | 98.86% | 98.87% | [Native](native/issue7905/full.tsv) · [WASM](wasm/issue7905/full.tsv) |
| `issue8040-insert` | 4 | 99.99% | 99.99% | [Native](native/issue8040-insert/full.tsv) · [WASM](wasm/issue8040-insert/full.tsv) |
| `issue8040-insert-delete` | 4 | 99.99% | 99.99% | [Native](native/issue8040-insert-delete/full.tsv) · [WASM](wasm/issue8040-insert-delete/full.tsv) |
| `handbook-hwp` | 383 | 99.96% | 99.96% | [Native](native/handbook-hwp/full.tsv) · [WASM](wasm/handbook-hwp/full.tsv) |
| `handbook-hwpx` | 383 | 99.96% | 99.96% | [Native](native/handbook-hwpx/full.tsv) · [WASM](wasm/handbook-hwpx/full.tsv) |
| `seo-body-list-typed` | 1 | 100.00% | 100.00% | [Native](native/seo-body-list-typed/full.tsv) · [WASM](wasm/seo-body-list-typed/full.tsv) |
| `seo-helvetica-styles` | 1 | 100.00% | 100.00% | [Native](native/seo-helvetica-styles/full.tsv) · [WASM](wasm/seo-helvetica-styles/full.tsv) |
| `seo-pic-crop-01` | 1 | 100.00% | 94.65% | [Native](native/seo-pic-crop-01/full.tsv) · [WASM](wasm/seo-pic-crop-01/full.tsv) |
| `seo-shape-group-02` | 1 | 100.00% | 100.00% | [Native](native/seo-shape-group-02/full.tsv) · [WASM](wasm/seo-shape-group-02/full.tsv) |
| `seo-re-03-latin` | 1 | 100.00% | 100.00% | [Native](native/seo-re-03-latin/full.tsv) · [WASM](wasm/seo-re-03-latin/full.tsv) |
| `yeongwol` | 4 | 99.97% | 99.98% | [Native](native/yeongwol/full.tsv) · [WASM](wasm/yeongwol/full.tsv) |
| `outline` | 9 | 99.99% | 99.94% | [Native](native/outline/full.tsv) · [WASM](wasm/outline/full.tsv) |
| `nested` | 17 | 94.96% | 96.35% | [Native](native/nested/full.tsv) · [WASM](wasm/nested/full.tsv) |
| `sample10` | 763 | 99.41% | 99.40% | [Native](native/sample10/full.tsv) · [WASM](wasm/sample10/full.tsv) |

## 편람 PDF 130쪽

HWP/HWPX의 Native·fresh WASM 네 TSV 모두 PDF 130쪽(인쇄 122쪽)의 raw는 99.96%, 경계 보정 후 보조값은 99.97%다. 나머지 382쪽은 모두 raw 100.00%다. 알려진 WMF 차이에 100%를 강제하지 않는 지침의 실제 측정 근거로 기록한다.

## 재현과 후속 검증

전체 TSV는 각 입력을 `scripts/visual_sweep.py --silhouette-only --resume --hwp <입력> --pdf <독립 Print> --key <문서> --rhwp-bin <기록된 Native producer> --compat <표의 compat> --dpi 96 --out <ignored 출력 경로>`로 측정했다. WASM은 여기에 `--wasm-pkg <기록된 최적화 package>`를 공급했다. 명시된 글꼴 환경 입력은 `--font-environment`로, sample10의 외부 참조 그림은 각각 `--linked-image`로 공급했다. 실제 입력·환경·링크 그림의 SHA는 inventory에 기록했다.

각 행의 실측 TSV 보존과 기존 기준의 하락 없음만 확정한다. R41 새 source의 실제 비교·원 PR 주장 페이지 판독·개별 최종 승인·원격 CI는 별도로 완료한다.
