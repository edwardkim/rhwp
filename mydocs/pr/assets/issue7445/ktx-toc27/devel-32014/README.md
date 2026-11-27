# KTX 목차 27쪽의 정렬 회귀 복원 근거

2026-11-26 KST, `upstream/devel` `32014de6e0ade1a3baea8852c02270e40b27b345`에서
Native Skia와 최적화한 fresh WASM을 각각 새로 빌드했다. [27쪽 원본](../KTX.hwp)과
[독립 한컴 2020 Print PDF](../KTX-hwp-2020.pdf)를 96 DPI·PDFium·2px raw 실루엣으로 비교했다.
원본·PDF·빌드 SHA-256, 글꼴 경로, 명령, TSV 해시와 쪽번호 기하 결과는 [검증 기록](validation.json)에 있다.

| 출력 | 전쪽 최저 | 목차 2쪽 | 표와 후속 본문 26쪽 | 전체 쪽수 |
| --- | ---: | ---: | ---: | ---: |
| [Native](native.tsv) | 99.98792% · 1쪽 | 100.00000% | 100.00000% | 27/27 |
| [fresh WASM](wasm.tsv) | 99.87075% · 8쪽 | 100.00000% | 99.99701% | 27/27 |

목차의 쪽번호 17개는 한컴과 같은 순서로 같은 셀 안에 있다. 두 출력 각각 번호 오른쪽 끝의
최대 편차가 0.10px이고, 한컴 Print의 공통 오른쪽 끝 701.64px와 차이는 최대 0.06px다.
점선 리더, 제목과 쪽번호의 줄 순서, 외곽선을 확대해 확인했다.
[Native 2쪽](native-p002-review.png), [fresh WASM 2쪽](wasm-p002-review.png).

26쪽의 표 경계 및 뒤 본문 소속과 순서를 확인했다. fresh WASM 최저 8쪽에서는
덧칠한 장 번호와 제목 주위에 미세한 획 차이가 있으며 제목 줄과 위아래 가로 경계를 보존한다.
[Native 26쪽](native-p026-review.png), [fresh WASM 26쪽](wasm-p026-review.png),
[fresh WASM 8쪽](wasm-p008-review.png).

과거 전체 SVG 검사와 쪽번호 숫자 최대 x 검사가 보호하던 2쪽 목차의 쪽번호에 대해
같은 셀 소속, 17개 번호 순서, 같은 오른쪽 끝 및 한컴 기준 ±1px를 검사한다.
기존 첫 점 중심·점 간격 검사를 함께 유지한다.
