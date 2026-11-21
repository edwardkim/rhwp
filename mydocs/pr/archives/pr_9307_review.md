# PR #9307 검토 — 셀 줄간격 편집 시 내어쓰기 복원

[#7407](https://github.com/edwardkim/rhwp/issues/7407) · [#9307](https://github.com/edwardkim/rhwp/pull/9307) · [오늘 작업](../../orders/20261122.md#issue-7407)

2026년 11월 22일 KST에 collaborator self-review를 수행했다. 제품·회귀 검사와 코드 head의 필수 CI는 통과했다. 통합은 정본 보호 하한 재검증을 기다린다.

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
