# PR #9189 통합 검토 · 보호문서 검증

[통합 PR #9189](https://github.com/edwardkim/rhwp/pull/9189)의 원 PR #9158·#9159·#9161·#9162·#9163·#9166에 공통으로 적용한 보호문서 검증 기록이다.

| 출력 경로 | 문서 | 쪽수 | 보호값 하락 | 확인 방법 |
| --- | ---: | ---: | ---: | --- |
| Native Skia | 23종 | 2,023쪽 | 0쪽 | 전쪽 새 캡처 및 TSV 대조 |
| fresh WASM | 23종 | 2,023쪽 | 0쪽 | 새 빌드의 JS·WASM 바이트 동일성 및 기존 수용 TSV 전쪽 대조 |
| 공개 웹폰트 편람 HWPX Native | 1종 | 383쪽 | 0쪽 | `--handbook-gate` 전쪽 실행. Native 23종에 포함 |

- 코드 head `5b3e013dfb49e82f24fc769f81170e2310adf612`의 적용 대상 CI가 모두 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44470122043) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44470122101) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44470121749) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44470122018) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44470122039) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44470121845)
- 소스 `64ba35858d4f511bdd993f24eae9337880a7f2b2`로 검증했다. GitHub의 코드 CI head와 결과는 개별 PR 검토 기록에 명시한다.
- 정본 `final.tsv`의 SHA-256은 `295f7774a65d722e1fcc965910628dad3e43c9620a3d0181f7388cec111d6f9a`다. 기준 PDF·입력·backend 조건은 정본 manifest를 따랐다. 페이지별 `silhouette_raw_match_percent`를 공식 소수 둘째 자리로 비교했다. [정본 TSV](../visual-protected-baseline/final.tsv) · [정본 manifest](../visual-protected-baseline/manifest.json)
- Native는 23종을 전쪽 재산출했다. 각 TSV의 바이트가 저장소의 수용 TSV와 같았으며 상승과 하락은 각각 0쪽이다. [Native 생산·대조 기록](protected-native-validation.json)
- fresh WASM 빌드의 JS와 WASM은 기존 수용 환경의 프로그램과 각각 SHA-256이 같았다. 기존 수용 TSV 23개는 파일 해시와 현재 정본 하한을 다시 확인했다. [WASM 동일성 및 TSV 검증 기록](protected-wasm-byte-identity.json)

## 문서별 전쪽 증적

| 문서 | 쪽수 | Native 최저 | Native 재현 환경 | 전쪽 TSV |
| --- | ---: | ---: | --- | --- |
| `yeongwol` | 4 | 100.00% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/yeongwol.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/yeongwol.tsv) |
| `seo-body-list-typed` | 1 | 100.00% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/seo-body-list-typed.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/seo-body-list-typed.tsv) |
| `handbook-hwp` | 383 | 99.97% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/handbook-hwp.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/handbook-hwp.tsv) |
| `seo-helvetica-styles` | 1 | 100.00% | 기존 수용 글꼴 조건 재현 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/seo-helvetica-styles.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/seo-helvetica-styles.tsv) |
| `sample16-hwpx` | 64 | 99.99% | 기존 수용 글꼴 조건 재현 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/sample16-hwpx.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/sample16-hwpx.tsv) |
| `sample16-hwp2022` | 64 | 100.00% | 기존 수용 글꼴 조건 재현 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/sample16-hwp2022.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/sample16-hwp2022.tsv) |
| `seo-re-03-latin` | 1 | 100.00% | 기여자 공개 환경 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/seo-re-03-latin.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/seo-re-03-latin.tsv) |
| `angle-narrow` | 1 | 100.00% | 기여자 공개 환경 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/angle-narrow.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/angle-narrow.tsv) |
| `outline` | 9 | 100.00% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/outline.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/outline.tsv) |
| `issue8040-insert` | 4 | 99.99% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/issue8040-insert.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/issue8040-insert.tsv) |
| `nested` | 17 | 99.98% | 기존 수용 글꼴 조건 재현 | [Native](../../../working/assets/issue9104/final/protected/native/nested.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/nested.tsv) |
| `sample16-hwp2024` | 64 | 99.99% | 기존 수용 글꼴 조건 재현 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/sample16-hwp2024.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/sample16-hwp2024.tsv) |
| `handbook-hwpx` | 383 | 99.97% | 공개 웹폰트 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/handbook-hwpx.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/handbook-hwpx.tsv) |
| `issue7905` | 6 | 98.86% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/issue7905.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/issue7905.tsv) |
| `sample10` | 763 | 99.57% | 기존 수용 글꼴 조건 재현 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/sample10.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/sample10.tsv) |
| `issue8040-insert-delete` | 4 | 99.99% | 검증용 설치 글꼴 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/issue8040-insert-delete.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/issue8040-insert-delete.tsv) |
| `seo-pic-crop-01` | 1 | 100.00% | 기여자 공개 환경 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/seo-pic-crop-01.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/seo-pic-crop-01.tsv) |
| `seo-shape-group-02` | 1 | 100.00% | 기여자 공개 환경 | [Native](../jinho-von-choi-20261114/final-source-8ad/protected/native/seo-shape-group-02.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/protected/head/wasm/seo-shape-group-02.tsv) |
| `chemical` | 103 | 99.75% | 검증용 설치 글꼴 | [Native](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/native/head/chemical.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/wasm/head/chemical.tsv) |
| `spec13` | 71 | 99.69% | 검증용 설치 글꼴 | [Native](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/native/head/spec13.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/wasm/head/spec13.tsv) |
| `outline-demo` | 3 | 100.00% | 검증용 설치 글꼴 | [Native](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/native/head/outline-demo.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/wasm/head/outline-demo.tsv) |
| `decree` | 74 | 99.74% | 기존 수용 글꼴 조건 재현 | [Native](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/native/head/decree.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/wasm/head/decree.tsv) |
| `outline-minimal` | 1 | 100.00% | 검증용 설치 글꼴 | [Native](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/native/head/outline-minimal.tsv) · [WASM](../issues7733_7738_20261117/hy-hanyang-font-identity/scope-whole/wasm/head/outline-minimal.tsv) |

## 공개 글꼴 환경

- 편람 HWPX Native 383쪽은 격리 macOS에서 웹으로 공급한 KoPub 바탕·돋움 Light·Medium·Bold 6종과 함초롬바탕으로 `--handbook-gate`를 통과했다. 최저 99.97%, 보호값 하락 0쪽이었다. [글꼴 프로그램 및 게이트 기록](handbook-webfonts.json)
- 기여자 공통 대상 `seo-re-03-latin`, `angle-narrow`, `seo-pic-crop-01`, `seo-shape-group-02`는 Native와 fresh WASM 양쪽에서 확인했다. 설치 글꼴 및 기존 수용 글꼴을 재현한 다른 행의 공개 환경 재현 여부는 별도 확인이 필요하다. [기여자 적용 기준](../../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존)

## 원 PR의 렌더링 검토 증적

- #9166 P136의 독립 기준은 한컴 데스크톱 2022의 Hancom PDF Print다. 96 DPI 및 2px raw 실루엣으로 대조했다. 통합 후보와 고정 devel은 각 backend의 2쪽 PNG·TSV가 바이트 단위로 같았다. [대조 기록](p136-devel-comparison.json) · [기준 PDF 및 산출물 출처](p136-validation.json)

| 출력 경로 | 1쪽 | 2쪽 | 전쪽 TSV |
| --- | --- | --- | --- |
| Native | 99.66% · [review](native/review_001.png) · [overlay](native/overlay_001.png) | 100.00% · [review](native/review_002.png) · [overlay](native/overlay_002.png) | [TSV](native/silhouette.tsv) |
| fresh WASM | 99.67% · [review](wasm/review_001.png) · [overlay](wasm/overlay_001.png) | 100.00% · [review](wasm/review_002.png) · [overlay](wasm/overlay_002.png) | [TSV](wasm/silhouette.tsv) |
