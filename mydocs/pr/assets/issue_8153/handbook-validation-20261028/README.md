# #8153 편람 Native·fresh WASM 시각 검증

2026-10-28에 두 원본과 각각의 독립 한컴 2024 Print PDF를 96dpi에서
`scripts/visual_sweep.py`의 색상 경계 보정 전 2px 실루엣으로 비교한다.
기준 PDF 래스터는 `scripts/visual_sweep_pdf.py`의 PDFium
`pdfium_smooth_rgb_exact_dpi_v2` 경로를 사용한다. TSV의 raw 표시는 소수점 둘째
자리이며, 그 이전 정밀도의 최저값과 미대응 픽셀 수도 함께 기록한다.

## 입력과 기준

| 원본 | 원본 SHA-256 | 독립 Print PDF SHA-256 | 전체 쪽수 |
| --- | --- | --- | ---: |
| `samples/2025 행정업무운영 편람(최종).hwp` | `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` | `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593` | 383 |
| `samples/2025 행정업무운영 편람(최종).hwpx` | `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6` | `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e` | 383 |

PDF는 각각 `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`,
`pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`다.

## Native 후보 결과

측정 시 HEAD `17509242336cbeb8937639943aae858735941d1d`, 미커밋 제품 소스 diff
SHA-256 `93f11d74f272682aede1a833c9cc301a0c86820c44b3a129d3c365ff0892d5d3`,
Native Skia 실행 파일 SHA-256
`e845b2ffd8102995227b22ed3203bff5ca1a0e16fe0d7fa40996292be7caf447`이다.
이후 커밋 `b51d34faf2`는 회귀 테스트 설명 주석만 바꾸며 `src`, `crates`,
Cargo 설정과 검증 스크립트의 제품·빌드 입력은 변경하지 않았다. Native의
측정 시점 source와 후속 커밋을 구분해 기록한다.
두 원본 모두 383쪽을 측정했으며 전쪽 최저 raw는 99.96463%, 130쪽이다.
정수 미대응 0은 각 380쪽이다. 이 후보와 바로 이전 후보의 전쪽 raw와 미대응
픽셀 수는 두 원본 모두 변동이 없었다. 130쪽 WMF 내부 잔여는 사용자의 지시에
따라 추가 보정하지 않는다.

| 원본 | Native 전쪽 TSV | 9쪽 Native | 309쪽 Native |
| --- | --- | --- | --- |
| HWP | [TSV](native/handbook-hwp.tsv) | [review](native/handbook-hwp-p009-review.png) · [overlay](native/handbook-hwp-p009-overlay.png) | [review](native/handbook-hwp-p309-review.png) · [overlay](native/handbook-hwp-p309-overlay.png) |
| HWPX | [TSV](native/handbook-hwpx.tsv) | [review](native/handbook-hwpx-p009-review.png) · [overlay](native/handbook-hwpx-p009-overlay.png) | [review](native/handbook-hwpx-p309-review.png) · [overlay](native/handbook-hwpx-p309-overlay.png) |

9쪽의 두 항목은 독립 Print의 시작 xMin이 각 277.68pt(370.24px)이며,
후보 Native 렌더 트리에서는 각 370.3px이다. 309쪽 제목의 독립 Print 원점은
241.56pt(322.08px)이며 후보의 회귀 진단을 통과했다.

## fresh WASM

2026-10-29 두 원본 각각 전 383쪽은 최저 raw 99.96402%(130쪽), 미대응 0은
각 380쪽이다. [HWP WASM 전쪽 TSV](wasm/handbook-hwp.tsv),
[HWPX WASM 전쪽 TSV](wasm/handbook-hwpx.tsv),
9쪽 [review](wasm/handbook-hwp-p009-review.png) · [overlay](wasm/handbook-hwp-p009-overlay.png),
130쪽 [review](wasm/handbook-hwp-p130-review.png) · [overlay](wasm/handbook-hwp-p130-overlay.png),
309쪽 [review](wasm/handbook-hwp-p309-review.png) · [overlay](wasm/handbook-hwp-p309-overlay.png)를 보존한다. 최적화 WASM의
SHA-256은 `92f9333fcfefa541d1892bd552da26544e631efa2a14d804dd223b0c2c38e0e9`이며
빌드 source는 위 Native와 같은 HEAD·제품 diff다.
HWPX의 완료된 PNG 중 9·130·309쪽은 출력 완료 기록의 SHA-256과 대조한 뒤
독립 Print와 비교했다. raw는 각각 100.00000%, 99.96402%, 100.00000%다.
[9쪽 review](wasm/handbook-hwpx-p009-review.png) · [overlay](wasm/handbook-hwpx-p009-overlay.png),
[130쪽 review](wasm/handbook-hwpx-p130-review.png) · [overlay](wasm/handbook-hwpx-p130-overlay.png),
[309쪽 review](wasm/handbook-hwpx-p309-review.png) · [overlay](wasm/handbook-hwpx-p309-overlay.png)를 보존한다.
2026-10-29 실행에서 HWP 9쪽 PNG를 먼저 독립 Print와 비교해 raw 100.00000%와
두 항목의 개선된 시작 위치를 확인했다. HWPX 및 전쪽 결과를 대신하지 않는다.
같은 실행의 HWP 130쪽은 TSV에 99.96%로 표시된다.
HWP 383쪽 PNG 생성 후 TSV 작업자의 스크립트 재진입 오류가 발생해, 각 PNG를
WASM 출력 완료 기록의 SHA-256과 대조한 뒤 같은 PNG로 점수 계산을 재개했다.
실패한 부분 TSV를 최종 결과로 사용하지 않는다.

## 고정 base 대조

현재 작업의 고정 base는 `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다.
기존 전쪽 base TSV는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`에서
생성됐으며 두 커밋의 제품·빌드 입력이 다르므로 현재 base의 증거로 사용하지
않는다. `9418ed0c7c`의 동일 조건 전쪽 Native/fresh WASM 결과를 확인해야 한다.
기존 로컬 base worktree의 HEAD는 `9418ed0c7c`이며 두 편람 입력 SHA-256은 위
후보 입력과 같다. 해당 worktree에는 별도의 `rhwp-studio/public/rhwp.js` 변경과
`output`이 있으므로 이를 보존하며 제품·빌드 입력이 깨끗한 상태를 확인했다.
2026-10-29 고정 base HWP Native 전 383쪽의 [TSV](base/native/handbook-hwp.tsv)를
산출했다. TSV 표시 정밀도인 소수점 둘째 자리 기준으로 후보 대비 하락 0쪽,
개선 221쪽, 동일 162쪽이다. 반올림 전 raw 정밀도로도 하락 0쪽이며,
개선 242쪽, 동일 141쪽이다. base 최저 raw는 91.89462%다.
HWPX Native의 [base 전쪽 TSV](base/native/handbook-hwpx.tsv)도 383쪽이며,
반올림 전 raw 기준 후보 대비 하락 0쪽, 개선 252쪽, 동일 131쪽이다.
base 최저 raw는 91.89462%다. 이후 base fresh WASM 결과는 아래에 기록한다.
먼저 산출된 base HWP Native의 직접 비교에서 9쪽 raw는 99.99464% → 후보
100.00000%, 130쪽은 98.92624% → 99.96463%, 309쪽은 양쪽 모두 100.00000%다.
이는 선택한 세 쪽의 결과이며 base 전쪽 산출을 대신하지 않는다.

## 373쪽 독립 Print와 Native 확대 판독

후보에서 이미 출력한 HWP 373쪽 Native PNG와 동일 쪽의 독립 Print PDF PNG를
같은 영역·배율로 확대한 [본문 비교](native/handbook-hwp-p373-text-review.png)를
확인했다. 첫 글줄의 원문과 줄바꿈은 서로 대응하며 글꼴 획의 농도·가장자리에는
차이가 보인다. 이는 해당 영역의 Native 판독이며 HWPX·fresh WASM이나 다른 쪽의
확대 판독을 대신하지 않는다. 원본 페이지의 raw는 100.00000%지만 실루엣 지표가
이 농도 차이까지 동일함을 뜻하지는 않는다.

같은 영역의 HWPX [독립 Print와 fresh WASM 확대 비교](wasm/handbook-hwpx-p373-text-review.png)에서도
원문·줄바꿈이 대응하며 획 농도·가장자리 차이가 보인다. 이 판독은 HWPX의 해당
영역과 fresh WASM에 한정한다. 두 이미지는 각각 해당 원본의 PDF 및 후보 PNG에서
같은 좌표를 잘라 2배로 표시했다.

## 46쪽 글자·공백 fit 확대 판독

HWPX 46쪽의 [독립 Print와 fresh WASM 표 확대 비교](wasm/handbook-hwpx-p046-table-review.png)는
`기업 간 거래, 기업 간 거래(B2B)`를 포함한다. 같은 셀의 글자·공백 순서와 한 줄
소유를 대조했고, 글꼴 획 농도와 표 선의 가장자리에 차이가 보인다. 이 판독은
해당 영역의 HWPX fresh WASM에 한정하며 다른 원본·경로의 판독을 대신하지 않는다.
두 이미지는 각각 PDF와 후보 PNG의 같은 좌표를 잘라 2배로 표시했다.

2026-10-29 고정 base의 HWP fresh WASM도 전 383쪽을 측정했다.
[base HWP WASM TSV](base/wasm/handbook-hwp.tsv)의 최저 raw는 91.90995%이며,
반올림 전 raw 기준 후보 대비 하락 0쪽, 개선 242쪽, 동일 141쪽이다. 최적화
WASM SHA-256은 `5fd7d45d8c3484abf58952d28d4954b95a1eee6dd7c471068a66b5d88a2bfb31`이다.
이전 base Native의 출력 메타데이터에는 글꼴 환경 필드가 없어 실제 Native 실행에
전달한 환경 파일을 확인하고 후보 WASM의 환경과 같은 것을 확인해 base WASM에
적용했다. base HWPX WASM은 진행 중이다.

## 148쪽 마지막 줄 확대 판독

HWPX 148쪽의 [독립 Print와 fresh WASM 확대 비교](wasm/handbook-hwpx-p148-text-review.png)에서
`생산등록번호`부터 `설계한다.`까지의 글자·공백 순서와 한 줄 소유, 뒤따르는
제목을 대조했다. 같은 영역을 2배로 표시했으며 획 농도·가장자리 차이가 보인다.
이 판독은 해당 원본·쪽·경로에 한정한다.
