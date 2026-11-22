# 한컴 선 기호 변환 검증

upstream `devel` 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22`와 수정 소스 `b0cfac0dc8b81d00bf2dbe2725538d334c8889ce`를 2026-11-21 macOS ARM64, Rust 1.93.1에서 비교했다. [같은 재현 입력과 한컴 기준본](../../../samples/issue5140-version-boundary/README.md)을 사용했다.

| HWP 파일 형식 | 기준 devel | 수정 후 | 굵은 `B` 시작 | 둘째 줄 `NEXT` 시작 |
| --- | --- | --- | ---: | ---: |
| 5.0.1.5 | 변환 누락 9종 | 한컴과 본문 차이 0곳 | 11 | 13 |
| 5.0.1.6 | 오변환 6종 | 한컴과 본문 차이 0곳 | 10 | 12 |

각 입력의 기호 문단 15개 모두에서 굵은 서식의 범위와 UTF-16 줄 시작 위치가 한컴 HWPX와 일치했다. 원본 HWP에 저장된 줄 정보가 없어 줄 좌표 검사는 같은 본문에 한컴 5.0.1.6 HWPX의 줄 정보를 적용한 뒤 양쪽 기준본과 비교했다. 두 CLI 변환은 `--verify --verify-pages` 종료 코드 0이었다. 집중 검사 7개가 통과했고, 새로 빌드한 WASM에서 변환한 HWPX도 Native 결과와 바이트 단위로 일치했다.

## 인쇄 PDF 대조

각 입력의 한 페이지 전체를 96dpi에서 비교했다. 일치율은 색상 경계 보정 전 2px 실루엣 수치다. 네 비교 모두 90% 기준을 통과했고 쪽수와 줄 순서가 일치했다. 5.0.1.6의 일부 기호 모양과 간격은 한컴과 다르게 표시된다.

| HWP 파일 형식 | 출력 | 일치율 | 증적 |
| --- | --- | ---: | --- |
| 5.0.1.5 | native | 100.00000% | [TSV](native/legacy-export/silhouette.tsv) · [비교](native/legacy-export/review_001.png) · [겹쳐 보기](native/legacy-export/overlay_001.png) |
| 5.0.1.6 | native | 98.39398% | [TSV](native/modern-export/silhouette.tsv) · [비교](native/modern-export/review_001.png) · [겹쳐 보기](native/modern-export/overlay_001.png) |
| 5.0.1.5 | wasm | 100.00000% | [TSV](wasm/legacy-export/silhouette.tsv) · [비교](wasm/legacy-export/review_001.png) · [겹쳐 보기](wasm/legacy-export/overlay_001.png) |
| 5.0.1.6 | wasm | 98.38726% | [TSV](wasm/modern-export/silhouette.tsv) · [비교](wasm/modern-export/review_001.png) · [겹쳐 보기](wasm/modern-export/overlay_001.png) |

## 보호 문서

편람 Native 383쪽 검사는 최저 99.97%였고, 기준보다 하락한 페이지는 0쪽이었다. [편람 전체 TSV](handbook-native.tsv)

기여자 공통 보호 검사는 네 문서의 Native·WASM 총 8건이 통과했다.

| 원본 | Native | WASM |
| --- | --- | --- |
| `samples/re-03-latin-only-hancom.hwp` | 100.00% · [TSV](protected/52ee61fe7201e36b/native/silhouette.tsv) · [겹쳐 보기](protected/52ee61fe7201e36b/native/overlay_001.png) | 100.00% · [TSV](protected/52ee61fe7201e36b/wasm/silhouette.tsv) · [겹쳐 보기](protected/52ee61fe7201e36b/wasm/overlay_001.png) |
| `mydocs/pr/assets/issue_8371/angle-narrow/angle-2400.hwpx` | 100.00% · [TSV](protected/59f6ab1155623bda/native/silhouette.tsv) · [겹쳐 보기](protected/59f6ab1155623bda/native/overlay_001.png) | 100.00% · [TSV](protected/59f6ab1155623bda/wasm/silhouette.tsv) · [겹쳐 보기](protected/59f6ab1155623bda/wasm/overlay_001.png) |
| `samples/pic-crop-01.hwp` | 100.00% · [TSV](protected/cdacd874b6e6ac66/native/silhouette.tsv) · [겹쳐 보기](protected/cdacd874b6e6ac66/native/overlay_001.png) | 100.00% · [TSV](protected/cdacd874b6e6ac66/wasm/silhouette.tsv) · [겹쳐 보기](protected/cdacd874b6e6ac66/wasm/overlay_001.png) |
| `samples/shape-group-02.hwp` | 100.00% · [TSV](protected/d03a9dde3d5dc7ac/native/silhouette.tsv) · [겹쳐 보기](protected/d03a9dde3d5dc7ac/native/overlay_001.png) | 100.00% · [TSV](protected/d03a9dde3d5dc7ac/wasm/silhouette.tsv) · [겹쳐 보기](protected/d03a9dde3d5dc7ac/wasm/overlay_001.png) |
