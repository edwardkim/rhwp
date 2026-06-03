# Task #816 완료 보고서

## 1. 결론

#816은 현재 `devel`에서 재현되지 않는다.

이슈 등록 당시의 핵심 현상은 다음 두 가지였다.

```text
1. HWPX export-svg가 4페이지만 생성된다.
2. export-pdf에서 5페이지 이후 표 셀 텍스트가 비어 있다.
```

현행 `devel`에서는 대표 10페이지 이상 HWPX 샘플이 전체 페이지를 SVG로 생성하고,
후반 페이지의 표 셀 텍스트도 SVG/PDF 단일 페이지 출력에서 보존된다.

## 2. 검증 요약

| 파일 | 포맷 | 페이지 수 | export-svg 결과 | 후반 텍스트 |
|---|---|---:|---:|---|
| `samples/hwpx/exam_kor.hwpx` | HWPX | 20 | 20 SVG | `<text>` 다수 |
| `samples/exam_kor.hwp` | HWP | 20 | 20 SVG | 정상 |
| `samples/hwpx/aift.hwpx` | HWPX | 74 | 74 SVG | `<text>` 다수 |
| `samples/aift.hwp` | HWP | 74 | 74 SVG | 정상 |

PDF 단일 후반 페이지 검증:

| 파일 | 페이지 | 결과 |
|---|---:|---|
| `samples/hwpx/aift.hwpx` | 5 | PDF 생성 성공, `pdftotext` 텍스트 추출 성공 |
| `samples/hwpx/aift.hwpx` | 11 | PDF 생성 성공, `pdftotext` 텍스트 추출 성공 |

## 3. 산출물

```text
output/poc/task816_current/
```

## 4. 해석

보고 환경은 `rhwp v0.7.11 custom build from main branch`였고,
현재 `devel`은 v0.7.13 계열이다.

그 사이 HWPX 페이지네이션, 다중 섹션, 표 셀 텍스트 렌더링, HWPX→HWP 저장/렌더링 정합 관련 수정이 다수 반영되어
#816이 자연 해소된 것으로 판단한다.

## 5. 잔여 사항

`export-pdf` 전체 문서 변환은 일부 대형 문서에서 오래 걸릴 수 있다.
이번 이슈의 본질인 "후반 페이지 텍스트 누락"과는 별개로, 전체 PDF 변환 성능이 문제로 확인되면 별도 이슈로 분리한다.

## 6. 권장 처리

#816은 GitHub 이슈에 검증 결과를 남기고 close 처리한다.
