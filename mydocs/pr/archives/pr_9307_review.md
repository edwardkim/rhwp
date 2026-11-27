# PR #9307 검토 — 셀 줄간격 편집 시 내어쓰기 복원

[#7407](https://github.com/edwardkim/rhwp/issues/7407) · [#9307](https://github.com/edwardkim/rhwp/pull/9307) · [오늘 작업](../../orders/20261122.md#issue-7407)

## 최종 판정

**승인.** 2026년 11월 27일 KST에 원본 편집·저장, 전체 정본 보호와 정확한 코드 head의 필수 CI를 다시 확인했다. 기록 커밋의 필수 CI와 최종 병합 가능 상태 확인이 남아 있다. 아래 11월 22일 기록은 최초 검토 조건이며 최신 판정은 [11월 27일 재검증](#11월-27일-재검증)에 기록했다.

## 구현 검토

[원본 HWP](../../../samples/issue6639/rhwp-table-cell-minimal-repro.hwp)의 셀 31에서 문단 10개의 줄간격을 140%로 일괄 변경하면 한컴 2020 Print는 문단 5의 긴 줄에 ‘가’ 44자를 놓고 마지막 한 자를 다음 줄로 보낸다. 기존 편집 재조판은 저장 줄의 내어쓰기 미적용 플래그를 사용해 45자를 한 줄에 놓았다. 저장 줄의 폭은 39,208 HWPUNIT이고 내어쓰기가 적용되는 이어지는 줄의 폭은 38,208 HWPUNIT이다. 실제 저장 줄과 HWP 사양의 LineSeg 속성 및 독립 인쇄본을 대조했다. [원본·사양·인쇄본 설명](../../../samples/issue6639/README.md)

저장 줄이 선언한 폭과 어긋나게 단어 중간에서 끝나면 기존 underfill 판정으로 확인한다. 파일을 열 때와 편집할 때 모두 해당 판정으로 거절한 저장 줄을 공통 함수 `try_fill_frame_after_discarding_stored_rows`에서 비우고, 정상 문단 상자로 다시 조판한다. 문단 상자의 유효 폭에서 확정한 줄과 LineSeg가 이후 표 셀 크기 계산과 그리기에 쓰인다. 유효한 저장 줄은 기존 경로로 처리한다. 160% 저장 원본과 한컴이 다시 저장한 정상 HWPX, HWP·HWPX 두 차례 저장·재열기를 대조했다. #7410·#7414·#7416·#7546의 기존 줄 소속과 정상 대조군은 관련 회귀와 실제 출력에서 확인했다. [조판 코드](../../../src/renderer/composer/line_breaking.rs) · [관계형 회귀](../../../tests/cases/issue_7416_self_contradicting_ladder_is_reflowed.rs)

## 제출 검증과 코드 head CI

- 제출 시점 `devel` `1a27faacff48ea7329e76ce8eca15aa9ca8aa877`과 후보를 독립 Hancom Print에 PDFium 96 DPI로 대조했다. 원본 140%의 raw 실루엣은 Native 91.62%, 최적화 fresh WASM 91.47%에서 각각 100.00%로 개선됐다. 각 형식의 두 번째 재열기에서도 100.00%였다. 원본 160%와 정상 HWPX를 포함한 32개 비교 행에서 기준 대비 하락은 없었다. 확대 이미지에서 문단 5의 줄 소속, 앞뒤 내용 순서와 표 경계를 판독했다. [전쪽 TSV와 대표 확대 이미지](../assets/issue_7407/README.md)
- Native·WASM lint와 로컬 회귀 13,938건, Native Skia 4,032건, 그림 자리 표시자 2건, 직접 PDF 36건을 통과했다. 기여자 보호 4개 문서는 Native·fresh WASM 8건 모두 100.00%였다. 편람 HWPX Native 383쪽은 최저 99.97%로 보호값 하락 없이 통과했다. [시각 증거](../assets/issue_7407/README.md)
- 코드 head `dc357fd5d035c9082dafb31710c2e48339ee4458`의 GitHub Build & Test, Native Skia, 기본 기능 회귀 A–E, Canvas Visual Diff, Adapter inter-diff, Proptest, CodeQL 분석 및 CI Impact Policy가 통과했다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44958736393) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44958736286) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44958736294) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44958736433) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44958736354) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/44961176967)

## 통합 검토 — 정본 보호 하한

[#9292](https://github.com/edwardkim/rhwp/pull/9292)가 기록한 아래 두 문서의 글꼴 파일 58개 중 기존 공급본과 11월 15일 공급본에서 SHA-256이 일치한 파일은 41개였다. 확보된 공급본으로 위 `devel`과 후보를 같은 조건에서 다시 렌더링했다. Native와 최적화 fresh WASM에서 174쪽씩 대조했으며, 348개 후보 PNG가 대응 `devel` PNG와 바이트 단위로 동일했다. 이 조건에서 공식 두 자리 보호 하한보다 낮은 행은 36개였다. [정본 보호 계약](../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존) · [정본 TSV](../assets/visual-protected-baseline/final.tsv)

| 문서 | 출력 | 미달 행 | 대표 쪽과 공식 하한 → 실측 |
| --- | --- | ---: | --- |
| 한글 문서 파일 형식 사양서 | Native | 1 | 2쪽 100.00% → 99.94% |
| 한글 문서 파일 형식 사양서 | fresh WASM | 1 | 2쪽 100.00% → 99.94% |
| 화학물질 표시 기준 | Native | 33 | 96쪽 100.00% → 99.74%, 92쪽 100.00% → 99.79% |
| 화학물질 표시 기준 | fresh WASM | 1 | 72쪽 100.00% → 99.98% |

화학물질 표시 기준 Native 미달 쪽은 25, 26, 31, 32, 33, 34, 35, 41, 44, 45, 47, 50, 52, 53, 55, 60, 61, 67, 69, 70, 72, 73, 76, 85, 86, 87, 88, 89, 92, 93, 94, 95, 96쪽이다. 형식 사양서 2쪽의 독립 PDF는 견고딕과 한컴 돋움을 사용한다. 정본 `HYGTRE.TTF`는 공급본에 없었고 `HDOTUM.TTF`는 SHA-256이 다른 버전만 확인했다. 화학물질 표시 기준의 정본 manifest가 지정한 `RHWPIndependentHFTSourceSymbols-Regular.otf`와 `RHWPIndependentHFTSourceSymbols-Bold.otf`도 공급본에 없었고 `batang.ttc`는 다른 버전이었다. 개별 글꼴과 모든 미달 쪽 사이의 인과관계는 확정하지 못했다.

현재 환경에서 확인한 두 문서는 후보 자체의 출력 차이가 없었지만 절대 하한을 재현하지 못했다. 정본을 재현할 수 있는 공급본 또는 검증 환경에서 통합 담당자 보호 대상 전체를 확인한 뒤 병합을 진행한다.

## 11월 27일 재검증

코드 후보 `66385e932b5e32e5d41a870ff48cac044e34a826`에는 `devel` `c6fcb1a61211d3d6eb8698660074849431b63e34`의 #9306·#9436·#9435가 들어 있다. 충돌한 파일은 `mydocs/orders/20261122.md` 한 개였으며 최신 기록에 #7407 항목을 합쳤다. 병행 검토 #9303과 제품·테스트 파일 공유는 없었다. #9438이 추가된 `2373cf77a58d4d34f22b22c452f9883d64a7b5df`에 대해 병합 시뮬레이션 `e593028d799a9cee92abc99e0085a1c8cb3e91b5`에서 코드·활성 검증 입력·정본 보호표를 대조했다. 이 시점의 [공개 리뷰](https://github.com/edwardkim/rhwp/pull/9307#pullrequestreview-6082769633)에 실측과 정확한 작업 브랜치를 기록했다.

### 140% 편집과 160% 대조

원본 HWP의 셀 31에서 열 문단의 줄간격을 140%로 일괄 적용했다. 기준 HWP에서는 오판된 저장 줄의 내어쓰기 플래그를 편집 재조판에 재사용해 45자가 앞줄에 남았다. 검증 후보에서는 한컴 Print처럼 앞줄 44자와 다음 줄의 마지막 ‘가’ 한 자로 나뉜다. Native 원본과 정상 한컴 HWPX의 CLI 편집 및 160% 출력은 각 100.00%였다. 최적화한 Chrome WASM은 두 입력에서 140% 직후와 HWP·HWPX 각각 두 차례 저장·재열기, 160% 초기·모양 ID·전체 스냅샷 복원을 합쳐 16상태 모두 100.00%였다. 96 DPI에서 표 바닥선은 한컴과 같은 1047px이고, 대상 셀 하단은 1px 이내였다. [재열기 TSV](../assets/issue_7407/merge-20261127/wasm-roundtrip.tsv) · [140% 확대 비교](../assets/issue_7407/merge-20261127/wasm-140-review.png) · [160% 대조](../assets/issue_7407/merge-20261127/wasm-160-review.png)

#9435의 36쪽에서 한컴 PDF가 사용하는 함초롬바탕 일반·굵게와 함초롬돋움을 공급한 fresh WASM과 Native 모두 100.00%였다. 브라우저에서 글상자의 세 번째 줄과 바로 아래 본문을 확인했다. #9436의 실제 클릭·편집·실행 취소도 브라우저가 SHA-256 `7a67f608be2f102ad4a04ae12399c9329a94a4c0eb2eeb27cb74a315ffcb9958`의 최적화 WASM을 가져온 상태에서 통과했다. #7416·#7114·#9422 집중 Rust 검사 12건과 Rust lint·workspace 빌드를 통과했다.

### 정본 보호와 글꼴 대조

최신 공식 TSV `8141acb297ce2f39148f9014e463986431d266120629e8f368665e5917138f9d`의 4,046행을 96 DPI의 색상 경계 보정 전 2px raw로 검사했다. Native와 fresh WASM은 각각 2,023행, 하락·누락 0행, 최저 98.86%였다. 최저값은 두 경로 모두 종합부동산세법 2쪽이며 해당 하한을 충족한다. 후보의 출력과 독립 기준 PNG의 해시가 이전 실행과 모두 같은 3,215건만 기존 실루엣 계산을 재사용했다. 공개 글꼴 공통 보호 8건과 편람 HWPX Native 383쪽도 통과했다. [전쪽 비교 TSV, SHA-256 `9a1e833c983dd622fff9b9bd42110dbc2d8592c9bd811153ab21e5b452f6a9ce`](../assets/issue_7407/merge-20261127/protected.tsv)

| 범위 | 기본 공급에서 관측 | 검증 공급에서 관측 |
| --- | --- | --- |
| sample10 Native 23·51·66·131·147·184·207·351·606·676·730쪽 | 99.89~99.99% | 11쪽 모두 100.00%, 전체 763쪽 통과 |
| sample16 HWP 2022 Native 6·9·31·46·47·50·55·57쪽 | 99.60~99.97% | 6쪽 99.96%로 하한 99.95% 충족, 나머지 일곱 쪽 100.00%, 전체 64쪽 통과 |
| sample10 WASM 81·351쪽 | 각 99.98%로 최신 하한 99.98% 충족 | SPJGT v3 단일 교체 시 각 100.00%, 미일치 픽셀 각각 7개에서 0개 |

같은 실행 파일에서 공급만 바꿔 차이를 재현했다. sample10 Native에는 허용된 Windows 10 굴림 `4b9ac63e8920…`과 해당 프로세스의 HFT 등록을 사용했다. sample16은 두 비교 모두 수식 글꼴 1.13 `a3a3cd992a89…`을 적용했고, 검증 조건에는 기본 휴먼명조 `d033df71fa41…`와 `SPMJ.otf`·`SPSMJ.otf`가 들어 있다. 앞선 동일 조건의 조판 및 글자 윤곽 증거와 연결한다. WASM에 등록한 SPJGT v3는 `HY Junggothic`, PostScript `HYJunggothic-SPJGT`, face 0이다. 글꼴 예외 적용은 0행이다. [검증된 Mac 공급 조건](../assets/issue_7114/protected-mac-20261127/README.md) · [SPJGT 두 쪽 수치와 파일 해시](../assets/issue_7407/merge-20261127/sample10-spjgt.tsv)

### 코드 head CI

코드 head `66385e932b5e32e5d41a870ff48cac044e34a826`의 필수 검사와 범위별 별도 검사가 모두 성공했다. 해당 head의 Full CI에 포함된 전체 Rust·Native Skia 회귀를 재사용했고 제품 경계는 위 집중 검사와 실제 출력으로 독립 확인했다. [Full CI](https://github.com/edwardkim/rhwp/actions/runs/45930639463) · [Canvas Visual Diff](https://github.com/edwardkim/rhwp/actions/runs/45930639371) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45930639376) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45930640009) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45930639298) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45933860629)

### #9303 반영 뒤 최종 결합 검증

#9303이 병합된 `devel` `a165c6c7d2106cfb16df9511fbd1fcfd2a166dc2`과 코드 head를 합친 `7274fa68b25ebfe2d19c9f69a1b915813b57428f`에서 Native와 최적화한 WASM을 새로 빌드했다. #9307 수정 파일 및 회귀 검사에는 #9303과의 충돌이 없었다. 정본 보호 4,046행을 다시 출력했고 하락·누락·글꼴 예외 적용은 각각 0행이었다. 4,046행의 PNG 해시와 raw가 모두 앞선 후보와 같으므로 공개한 전쪽 TSV를 재사용했다.

#7407의 140% 줄 소속과 HWP·HWPX 각 두 번 재열기, 160% 복원을 포함한 16개 WASM 상태와 sample10 81·351쪽의 공식·SPJGT v3 출력 해시도 앞선 후보와 같았다. #9303의 원본 HWP·한컴 저장 HWPX는 두 출력에서 2쪽이었고, 마지막 빈 문단 하나를 제거하면 각각 1쪽이었다. #7416과 #7910 집중 회귀 및 공개 글꼴 공통 보호 8건을 다시 통과했다. [#9303 경계 측정](../assets/issue_7407/merge-20261127/after-9303-boundaries.tsv) · [#9303 검토](pr_9303_review.md)
