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

두 자리 TSV에서 100.00%로 보이더라도 반올림 전에는 미대응 픽셀이 있을 수 있다.
이 측정 후보의 미대응 페이지는 두 원본에서 동일하며, Native와 fresh WASM을
각각 구분하면 다음과 같다. 이후 제품 코드가 바뀐 후보의 측정 결과는 아니다.

| 쪽 | Native raw | Native 미대응 픽셀 | fresh WASM raw | WASM 미대응 픽셀 |
| ---: | ---: | ---: | ---: | ---: |
| 43 | 99.99609% | 4 | 99.99601% | 4 |
| 130 | 99.96463% | 23 | 99.96402% | 23 |
| 141 | 99.99911% | 1 | 99.99911% | 1 |

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
적용했다. base HWPX WASM 결과는 아래에 기록한다.

## 148쪽 마지막 줄 확대 판독

HWPX 148쪽의 [독립 Print와 fresh WASM 확대 비교](wasm/handbook-hwpx-p148-text-review.png)에서
`생산등록번호`부터 `설계한다.`까지의 글자·공백 순서와 한 줄 소유, 뒤따르는
제목을 대조했다. 같은 영역을 2배로 표시했으며 획 농도·가장자리 차이가 보인다.
이 판독은 해당 원본·쪽·경로에 한정한다.

## 327쪽 마지막 줄 확대 판독

HWPX 327쪽의 [독립 Print와 fresh WASM 확대 비교](wasm/handbook-hwpx-p327-text-review.png)에서
`관리에 관한 법률 시행령`부터 `등록정보로 관리하여야 한다.`까지의 줄 구성과
앞뒤 문장을 대조했다. 같은 영역을 2배로 표시했으며 글꼴 획 농도와 가장자리에
차이가 보인다. 이 판독은 해당 원본·쪽·경로에 한정한다.

2026-10-29 고정 base의 HWPX fresh WASM도 전 383쪽을 측정했다.
[base HWPX WASM TSV](base/wasm/handbook-hwpx.tsv)의 최저 raw는 91.90995%이며,
반올림 전 raw 기준 후보 대비 하락 0쪽, 개선 252쪽, 동일 131쪽이다. 동일한
기준 WASM과 각 원본의 실제 지정 글꼴 환경을 사용했다.

## 368쪽 접수인 기호 확대 판독

HWPX 368쪽의 [독립 Print와 fresh WASM 확대 비교](wasm/handbook-hwpx-p368-dot-review.png)에서
접수인 셀의 점 세 개, 콜론, 괄호와 표 아래 경계를 대조했다. 기호의 순서와 셀
안 위치는 대응하며 획 농도·가장자리 차이가 보인다. 두 이미지는 같은 영역을
4배로 표시했으며 이 판독은 HWPX fresh WASM에 한정한다.

HWP 368쪽의 [Print·Native·fresh WASM 확대 비교](handbook-hwp-p368-three-way.png)도
같은 셀의 기호 수량·순서와 아래 경계를 확인했다. 획의 농도·가장자리 차이는
남으며 이 판독은 해당 셀에 한정한다. 세 이미지는 같은 영역을 2배로 표시했다.

HWP [148쪽](handbook-hwp-p148-three-way.png)과 [327쪽](handbook-hwp-p327-three-way.png)도
같은 영역의 Print·Native·fresh WASM을 2배로 대조했다. 선택한 마지막 줄과 앞뒤
내용은 대응하며 획 농도·가장자리 차이가 보인다. 각 이미지의 판독 범위에 한정한다.

HWPX [148쪽](handbook-hwpx-p148-three-way.png), [327쪽](handbook-hwpx-p327-three-way.png),
[368쪽](handbook-hwpx-p368-three-way.png)의 같은 영역도 독립 Print·Native·fresh
WASM으로 2배 확대해 확인했다. 선택한 마지막 줄·앞뒤 문장과 접수인 기호는
세 출력에서 대응하며 획의 농도·가장자리 차이는 남는다. 이 판독은 표시된
영역에 한정한다.

## 기준·후보 전쪽 차이표

[기준·후보 페이지별 비교 TSV](base-candidate-page-delta.tsv)는 두 원본과 두 출력
경로의 1,532개 쪽을 같은 쪽끼리 연결한다. raw와 차이의 표시는 소수점 둘째
자리이므로 작은 변화가 `0.00`으로 표시될 수 있다. 위의 하락·개선·동일 쪽수는
반올림 전 측정값으로 계산했다. 비교표 SHA-256은
`c35a5c7ee1cdb4ff2a986e7bbb688184e2c45306109e59ceb0aaef1296c01887`이다.

## 232쪽 묶음 빈칸 뒤 표지 확대 판독

두 편람 [HWP](handbook-hwp-p232-three-way.png)·[HWPX](handbook-hwpx-p232-three-way.png)의
232쪽에서 묶음 빈칸 뒤 `※ 여기서`로 시작하는 줄을 독립 Print·Native·fresh WASM의
같은 영역으로 2배 확대해 대조했다. 세 출력의 표지·글줄 위치와 내용은 대응하며
획 농도와 가장자리 차이가 보인다. render tree의 해당 앞 빈칸 run과 표지 run을
확인해 영역을 선택했다. 이 판독은 출처별 공백 전진의 모든 합성 대조 조건을
대신하지 않는다.

독립 Print에서 232쪽 표지 `※`의 xMin은 두 원본 모두 124.80pt(96dpi
166.40px)이고, 후보 Native render tree의 해당 run 시작은 두 원본 모두 165.9px다.
이는 같은 표지의 독립 좌표 대조이며 미측정 글꼴 자체의 전진 계약을 대체하지 않는다.

[HWP 46쪽](handbook-hwp-p046-three-way.png)과 [HWPX 46쪽](handbook-hwpx-p046-three-way.png)의
같은 표 영역을 독립 Print·Native·fresh WASM으로 2배 확대해 비교했다. 선택한
`기업 간 거래`의 한 줄 소유와 주변 셀의 줄바꿈은 대응하며, 글꼴 획 농도와 표
선 가장자리는 차이가 있다. 이 결과는 이미지에 표시된 영역에 한정한다.

[HWP 373쪽](handbook-hwp-p373-three-way.png)과 [HWPX 373쪽](handbook-hwpx-p373-three-way.png)의
같은 마지막 글줄 및 뒤 본문을 독립 Print·Native·fresh WASM으로 2배 확대해
비교했다. 표시한 글줄의 내용과 줄바꿈은 대응하며, 글꼴 획 농도와 가장자리
차이는 남는다. 이 판독은 이미지에 표시된 영역에 한정한다.

## 311쪽 법령 대조표 첫 조각

두 편람의 311쪽은 구역 11 문단 3의 첫 표 조각이며 각 Native·fresh WASM raw는
100.00000%다. 같은 쪽의 독립 Print와 출력에서 표 머리, 좌우 열, 첫 본문과
첫 조각 하단을 확인했다. 획 농도와 가장자리 차이가 보인다. 312쪽의 이어받기
내용·컷과 원점 수용 조건은 이 한 장의 점수만으로 판정하지 않는다.

| 원본 | Native | fresh WASM |
| --- | --- | --- |
| HWP | [review](native/handbook-hwp-p311-review.png) · [overlay](native/handbook-hwp-p311-overlay.png) | [review](wasm/handbook-hwp-p311-review.png) · [overlay](wasm/handbook-hwp-p311-overlay.png) |
| HWPX | [review](native/handbook-hwpx-p311-review.png) · [overlay](native/handbook-hwpx-p311-overlay.png) | [review](wasm/handbook-hwpx-p311-review.png) · [overlay](wasm/handbook-hwpx-p311-overlay.png) |

## 311→312쪽 표 내용 이어받기

독립 Print와 두 원본의 Native·fresh WASM render tree에서 311쪽 말미의 `2의2.`와
312쪽 첫 내용 `전자문서 형식을`, 이어지는 `기술의 표준`을 대조했다. 네 후보
출력 모두 앞의 항목은 311쪽에 한 번, 이어지는 두 문구는 312쪽에 각각 한 번
있으며 반대쪽에는 나타나지 않는다. 312쪽의 각 raw는 100.00000%다. 페이지
전체 overlay에서 표 머리·열·내용을 확인했으며 획 농도와 가장자리 차이가 보인다.
텍스트 소유와 이미지 판독은 각각 확인한 범위만 뜻한다.

| 원본 | Native | fresh WASM |
| --- | --- | --- |
| HWP | [review](native/handbook-hwp-p312-review.png) · [overlay](native/handbook-hwp-p312-overlay.png) | [review](wasm/handbook-hwp-p312-review.png) · [overlay](wasm/handbook-hwp-p312-overlay.png) |
| HWPX | [review](native/handbook-hwpx-p312-review.png) · [overlay](native/handbook-hwpx-p312-overlay.png) | [review](wasm/handbook-hwpx-p312-review.png) · [overlay](wasm/handbook-hwpx-p312-overlay.png) |

## 314→315쪽 법령 대조표 이어받기

두 독립 Print PDF와 두 원본의 Native·fresh WASM render tree에서 314쪽의
`제5조(문서 처리의 기본 원칙)`과 315쪽의 `시스템 또는 업무관리시스템`을
확인했다. 각 출력에서 앞 문구는 314쪽에 한 번, 이어지는 문구는 315쪽에 한 번
나타나고 반대쪽에는 없다. 후보 조판 항목의 앞 끝 컷과 뒤 시작 컷은 모두 `[4,1]`이다.
두 원본·두 경로·두 쪽의 raw는 모두 100.00000%다. overlay에서 표 테두리와
이어지는 본문을 직접 확인했고 획 농도·가장자리 차이가 보인다.

| 원본·쪽 | Native | fresh WASM |
| --- | --- | --- |
| HWP 314 | [review](native/handbook-hwp-p314-review.png) · [overlay](native/handbook-hwp-p314-overlay.png) | [review](wasm/handbook-hwp-p314-review.png) · [overlay](wasm/handbook-hwp-p314-overlay.png) |
| HWP 315 | [review](native/handbook-hwp-p315-review.png) · [overlay](native/handbook-hwp-p315-overlay.png) | [review](wasm/handbook-hwp-p315-review.png) · [overlay](wasm/handbook-hwp-p315-overlay.png) |
| HWPX 314 | [review](native/handbook-hwpx-p314-review.png) · [overlay](native/handbook-hwpx-p314-overlay.png) | [review](wasm/handbook-hwpx-p314-review.png) · [overlay](wasm/handbook-hwpx-p314-overlay.png) |
| HWPX 315 | [review](native/handbook-hwpx-p315-review.png) · [overlay](native/handbook-hwpx-p315-overlay.png) | [review](wasm/handbook-hwpx-p315-review.png) · [overlay](wasm/handbook-hwpx-p315-overlay.png) |

## 분리 소스의 2026-10-29 전쪽 재검증

커밋 `ffae46e8a6`의 제품 소스를 별도 검증 worktree에서 빌드해 두 원본을
같은 독립 Print PDF, 96dpi PDFium 및 2px raw 실루엣 조건으로 각각 전 383쪽
재검증했다. Native 실행 파일 SHA-256은
`9425abb8a2d6fb981992573aa618eced426f0ca745cf5d01cb3cc0c5e2970563`,
최적화 fresh WASM SHA-256은
`6894bcc9978476aa432b71a55ea149e821efcd27041a5ae137fe406b98ac888c`다.
제품 소스 출력 시점과 이후 테스트 전용 서식·문서 커밋을 구분한다.

두 입력 모두 Native 최저 raw 99.96463%, fresh WASM 최저 raw 99.96402%이며
최저 쪽은 130쪽이다. Native와 fresh WASM 각각 미대응 0은 380쪽이며,
43·130·141쪽의 정밀 raw 및 미대응 수는 위 표와 같다. 기존 후보와 전쪽 raw와
미대응 수가 변한 쪽은 없다. 소수 둘째 자리 TSV 네 파일은 이 문서에 이미
보존된 Native·WASM TSV와 바이트 단위로 같다. 130쪽 WMF 내부 차이는 사용자
지시에 따라 추가 보정하지 않는다. 이 일치율은 색 농도의 완전한 일치를 뜻하지
않으며 위 확대 판독 내용도 함께 적용된다.

HWPX 출력 후 테스트 소스의 서식 편집이 검증 worktree에 일시 반영되어 최초
완료 단계의 소스 무결성 검사가 실패했다. 해당 소스를 빌드 당시 상태로
복원하고, 이미 완성된 전쪽 PNG의 출력 기록 해시와 입력·독립 PDF·PDF 래스터
해시를 확인한 뒤 점수 계산과 소스 무결성 검사를 재실행해 완료했다. 새 글꼴은
배포하지 않으며 시스템 글꼴과 Git에서 제외한 로컬 검증 캐시만 사용했다.

이후 `30be639cb5`는 문단 편집의 빈칸 분할 인덱스 계산을 동등한 `filter`·`map`
표현으로 변경했다. 위 전쪽 출력은 이 제품 소스 변경 전 `ffae46e8a6`의 결과이며,
최종 제품 소스의 전쪽 측정은 아래에 별도로 기록한다.

## 테스트 이전 전 제품 소스의 전쪽 재검증

이 절의 측정 이후 `412426515e`에서 테스트 이전과 공통 함수 분리를 수행했다.
아래 전쪽 결과는 해당 변경 전 소스의 결과이며 변경 후 전쪽 측정과 구분한다.

제품 소스 `a1e511dbebcc27e187ff847d5510358fa9fbada4`를 독립 worktree에서
빌드한 Native 실행 파일 SHA-256은
`15facc62d7c7823fe91680decf7e795e5512635c7d08857541a8622c45e9f79c`다.
두 편람 모두 각 383쪽을 독립 Print와 비교했다. 각 전쪽 최저 raw는 130쪽의
99.96463%이고 정확한 미대응 0은 각 380쪽이다. 바로 앞 `ffae46e8a6` 결과와
모든 쪽의 정밀 raw 및 미대응 픽셀 수가 같고, 두 자리 Native TSV도 기존
`native/handbook-hwp.tsv`, `native/handbook-hwpx.tsv`와 바이트 단위로 같다.
핵심 14쪽씩의 직접 재출력도 모두 100.00000%다.

동일 제품 소스에서 빌드한 최적화 fresh WASM의 SHA-256은
`3dbc71ea881554b86eeae35fcc124779946906c1e0d5b6145747a7d5fa040c77`이다.
각 원본의 전 383쪽 최저 raw는 130쪽의 99.96402%, 미대응 0은 각 380쪽이다.
이전 `ffae46e8a6` 측정과 전쪽의 정밀 raw 및 미대응 픽셀 수가 같고, 두 자리
TSV도 `wasm/handbook-hwp.tsv`, `wasm/handbook-hwpx.tsv`와 바이트 단위로 같다.
130쪽의 WMF 이미지 내부 차이는 사용자 지시에 따라 추가 보정하지 않는다.
이 전쪽 시각 비교는 전체 회귀 테스트 통과를 의미하지 않는다.

309쪽의 두 원본·두 출력 경로에 대한 위 review와 overlay는 이 절의 제품 소스
`a1e511dbebcc27e187ff847d5510358fa9fbada4`의
실제 출력 PNG와 독립 Print PNG의 SHA-256을 각각 확인하고 다시 생성했다.
네 비교 모두 2px raw 실루엣은 100.00000%다. 부록 제목과 회색 띠 및
후속 제목을 직접 대조했으며 작은 색상 경계 차이는 overlay에서 구분된다.

## 글꼴 배포 범위 확인

고정 base `9418ed0c7c`와 후보 `135b3d4c22` 사이의 추가·변경 파일을
확장자와 파일 헤더로 확인했으며 새 글꼴 파일은 없다. 추가된 HWPX 3개의
ZIP 항목에도 글꼴 확장자나 글꼴 헤더가 없고, 추가된 PDF 3개는 `pdffonts`에
글꼴 항목이 없다. 이후 `fba5de4d5a`까지의 변경에는 제품 코드·회귀 검사와
문서가 포함되며, 글꼴·HWP/HWPX·PDF 파일의 추가나 변경은 없다. 추가된 소스에도
글꼴 데이터 URL과 일반적인 인코딩 글꼴 시그니처를 확인하지 못했다. 검증에 쓴 시스템
글꼴과 로컬 캐시는 저장소에 추가하지 않는다.

## 테스트 이전 후 제품 소스의 전쪽 재검증

2026-10-29 제품 소스 `412426515e1d367edf4c3cb8a0c38f31eb454b29`에서
두 편람의 Native와 fresh WASM 출력을 각각 독립 Print PDF와 같은 조건으로
전 383쪽 비교했다. Native 실행 파일 SHA-256은
`22e352737575c1c5f3cf9cd61dc92c409b792f22429f80b69d5611d849334ece`,
최적화 fresh WASM SHA-256은
`9454c941a1a1bd16ea87d1a744695aa1bddc2fe48d167c086ce656b445f0f902`다.
제품 소스와 검증 작업 트리의 내용은 별도로 대조했다.

네 결과 각각 이전 `a1e511db` 측정과 383쪽의 정밀 raw와 미대응 픽셀 수가
모두 같으며 하락한 쪽은 없다. 두 원본의 최저 raw는 Native 99.96463%,
fresh WASM 99.96402%이고 모두 130쪽이다. 각 결과의 정확한 100%는 380쪽,
소수점 둘째 자리 표시상 100.00%는 382쪽이다. 130쪽의 WMF 내부 차이는
사용자 지시에 따라 추가 보정하지 않았다. 네 개의 두 자리 TSV는 각각 기존
`native/`, `wasm/`의 동일 문서 TSV와 바이트 단위로 같다. 이 비교는 전체 회귀
테스트 결과와 구분한다.

위 309쪽 대표 review·overlay를 만들 때 사용한 `a1e511db` 출력 PNG와
`412426515e`에서 다시 출력한 PNG의 SHA-256을 각각 대조했다. 두 원본의 Native
PNG는 모두 `fb05203cb36ebae7405c6d4662c20b374e1f68a40a5349a6c9d125b41e`,
fresh WASM PNG는 모두 `b4e711c88e07a0c9bca518cb8f18d10c0a960c001192ef857a2b5edc6a33f111`로
출력 경로별 바이트가 일치한다. 독립 Print와의 비교 이미지 생성에 사용한 입력도
같으므로 기존 대표 이미지를 최종 소스의 해당 출력 증거로 연결한다.

같은 제품 소스의 Native Skia 라이브러리 검사는 본체 3,858건 통과·12건
무시, 나머지 라이브러리는 각각 15·165·2건 통과했다. 편람 관련 Skia 회귀는
자동 수집 suite의 해당 모듈을 두 차례 선택해 총 32건 통과했다. 첫 실행은
4개 suite의 29건, 추가 실행은 그림 샘플링과 기호 fallback 두 모듈의 3건이다.
검증 전후 소스 대조 결과가 같음을 확인했다. 이 선택 실행도 전체 회귀 검사와
구분한다.
