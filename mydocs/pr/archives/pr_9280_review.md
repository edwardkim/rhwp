# PR #9280 한컴 선 기호 변환과 병합 충돌 검토

## 최종 판정

**메인터너 보정 후 수용 가능.** 최신 devel과의 한컴 PDF 쪽수 등록 충돌을 해소한 code head `38c3b5171c52a925b2f575e9ad17a61fa620ca28`에서 로컬 검증과 GitHub CI를 통과했다. 후행 기록이 포함된 head는 필수 검사를 별도로 확인한다. [원 PR](https://github.com/edwardkim/rhwp/pull/9280) · [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45027558091)

## 원 변경과 충돌 해소

원 head `a0b40b2f937eee79413724928c79c603ce3d2433`는 HWP 5.0.1.5의 선 기호를 HWPX 보충 평면으로 변환하고, 5.0.1.6의 본문 코드 포인트는 보존한다. 저장기와 검증기가 UTF-16 단위의 글자 서식·줄 위치 계산을 공유한다. [원 기여 검증](https://github.com/edwardkim/rhwp/blob/17b17c934b527aa523f037ca12d8713394e43c82/mydocs/verification/issue5140-version-boundary/README.md)

최신 devel `622af208ee12c4cb16357d465e4802e7fd4726bc`를 원 head에 병합했다. 양쪽이 같은 자리에 추가한 한컴 인쇄 기준은 `tests/fixtures/oracle_page_count_verified_references.json`에서 대조했다. 공통 15개, 원 PR의 2개와 devel의 4개를 포함한 21개 원본·PDF 쌍의 SHA-256을 모두 확인했다. 충돌 구간 6개의 PDF 실측 쪽수와 Native에서 연 실제 쪽수가 모두 일치했다. 제품 소스와 기대값의 충돌은 없었다. [기준 목록](../../../tests/fixtures/oracle_page_count_verified_references.json) · [검증 영수증](../assets/issue5140-pr9280/validation.json)

## 기능 보존 검증

- HWP 5.0.1.5 및 5.0.1.6 두 입력의 Native CLI `export-hwpx --verify --verify-pages --json`에서 IR 차이 0건, 1쪽 유지가 확인됐다. 각 입력 15개 기호 문단의 본문과 굵은 서식은 독립 한컴 HWPX와 일치했다.
- macOS locked wrapper `--target web --out-dir pkg --no-opt`로 새로 빌드했다. fresh WASM으로 저장한 HWPX는 두 입력 모두 Native와 바이트 단위로 같았고 두 결과를 WASM으로 다시 열었을 때도 1쪽이었다.
- 최소 글꼴 환경에서 #5140 및 최신 devel에서 들어온 #8225·#9239·#9272·#9279·#9285 관련 회귀 26건과 전체 nextest 13,957건을 통과했다. 전체 제외는 45건이다. fmt, Native 및 WASM32 Clippy, 워크스페이스 빌드와 all-target Clippy, 테스트 manifest 검사도 통과했다.

## 독립 한컴 인쇄 대조

기여자가 보관한 데스크톱 한컴 Print PDF를 동일 입력으로 대조했다. Native Skia와 fresh WASM Canvas의 두 입력 전체를 96 DPI·2px 이웃 실루엣으로 비교했다. 두 형식 모두 1쪽이며 15개 기호 문단과 다음 줄의 순서가 유지됐다. 5.0.1.6의 일부 BMP 기호와 뒤 글자 사이 간격에 시각 차이가 남는다. [원본과 PDF의 출처·해시](../../../samples/issue5140-version-boundary/README.md)

| HWP 형식 | 실행 경로 | raw 실루엣 | 증적 |
| --- | --- | ---: | --- |
| 5.0.1.5 | Native Skia | 100.00% | [전쪽 TSV](../assets/issue5140-pr9280/native/legacy-export/silhouette.tsv) · [review](../assets/issue5140-pr9280/native/legacy-export/review-001.png) · [overlay](../assets/issue5140-pr9280/native/legacy-export/overlay-001.png) |
| 5.0.1.6 | Native Skia | 98.38% | [전쪽 TSV](../assets/issue5140-pr9280/native/modern-export/silhouette.tsv) · [review](../assets/issue5140-pr9280/native/modern-export/review-001.png) · [overlay](../assets/issue5140-pr9280/native/modern-export/overlay-001.png) |
| 5.0.1.5 | fresh WASM Canvas | 100.00% | [전쪽 TSV](../assets/issue5140-pr9280/wasm/legacy-export/silhouette.tsv) · [review](../assets/issue5140-pr9280/wasm/legacy-export/review-001.png) · [overlay](../assets/issue5140-pr9280/wasm/legacy-export/overlay-001.png) |
| 5.0.1.6 | fresh WASM Canvas | 98.51% | [전쪽 TSV](../assets/issue5140-pr9280/wasm/modern-export/silhouette.tsv) · [review](../assets/issue5140-pr9280/wasm/modern-export/review-001.png) · [overlay](../assets/issue5140-pr9280/wasm/modern-export/overlay-001.png) |

## Merge 후 contributor PR comment 계획

최종 merge SHA와 필수 CI를 확인한 뒤, 원 기여의 버전별 변환과 쪽수 근거 보존을 분리해 설명한다. Visual Sweep 결과는 같은 쪽의 review와 overlay를 merge SHA로 고정해 연결한다. [공식 비교 절차](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
