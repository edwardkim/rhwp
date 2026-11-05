# 원형 점선의 독립 Print 물리 기하

2026-11-05 KST. #8368 원본 확대 판독에서 발견한 차이를 독립 Windows 한컴으로 재현했다. 기존 `border_dash`의 선 굵기 index 계약과 같은 계층에서, 원형 점선도 **선언 index → 물리 지름·간격 → 공통 Path → 각 backend**로 전달한다. 이미 반올림한 실선 폭으로 지름·주기를 재추정하지 않는다.

## 입력과 관측

- 원 기여 `borders.hwp`의 실선·원형 점선 모두 실제 선언은 **1.0 mm(index 10)**다. 문서에 보이는 “0.7 mm” 설명은 저장값과 달라 근거로 쓰지 않았다.
- 같은 원본의 Mac 12.30 Print와 새 Windows 2020 Print 모두 원형 점의 중심 간격은 약 10.56pt(88/600 inch)였다. 원 기여 구현은 48/600 inch로 그려 점이 지나치게 촘촘했다. Mac 원의 fill 지름은 4.32pt, Windows fill+stroke는 약 4.318pt(36/600 inch)였다. 기존 구현은 24/600 inch였다.
- 16개 선언 굵기 진단은 공개 기여 HWPX를 복제해 borderFill/paraPr를 연속 ID로 추가하고 원형 점선의 선언 굵기만 바꿨다. LineSeg 위치를 수동 작성한 **벡터 측정용 합성 자료**다. 이 자료의 페이지 배치를 한컴 정상 저장 조판이나 기존 실물의 픽셀 정답지로 사용하지 않는다. 레이아웃의 수용 근거는 원 기여 원본과 최종 Native/fresh WASM 전쪽 비교다.
- Windows 한컴 **2020(11.0.0.9136)**과 **2024(13.0.0.3901)**의 실제 비동기 Print 응답·출력 byte/hash를 확인했다. 16개 지름·간격이 두 출력에서 모두 같았다. 600 DPI의 독립 물리 pen 단위로 측정하며 화면 DPI와 문서 ID에 맞춘 보정은 없다.
- PDF 벡터의 원 fill 경계에 명시 stroke 폭을 더해 실제 지름을 계산하고, 인접 원 중심 차이에서 간격을 측정했다. 단위는 1/600 inch다. 원형 점선은 기존 plain Dash/Dot·도형 pen과 다른 BorderFill 속성이므로 해당 경로를 변경하지 않는다.

| index | 선언 mm | 2020/2024 지름 | 2020/2024 중심 간격 |
| ---: | ---: | ---: | ---: |
| 0 | 0.1 | 4 | 8 |
| 1 | 0.12 | 4 | 10 |
| 2 | 0.15 | 6 | 13 |
| 3 | 0.2 | 8 | 18 |
| 4 | 0.25 | 10 | 23 |
| 5 | 0.3 | 10 | 25 |
| 6 | 0.4 | 14 | 35 |
| 7 | 0.5 | 18 | 43 |
| 8 | 0.6 | 22 | 53 |
| 9 | 0.7 | 24 | 60 |
| 10 | 1.0 | 36 | 88 |
| 11 | 1.5 | 52 | 130 |
| 12 | 2.0 | 70 | 173 |
| 13 | 3.0 | 104 | 260 |
| 14 | 4.0 | 140 | 348 |
| 15 | 5.0 | 174 | 433 |

## 정식 진단 자료

| 파일 | SHA-256 |
| --- | --- |
| [circle-width-matrix.hwpx](../../../../pdf/semanticist21-20261105-next10/circle-width-matrix.hwpx) | `c613abdd0b5894e9a3599cc8503171d2c062863b49c463eb910766fb6987c957` |
| [circle-width-matrix-2020.pdf](../../../../pdf/semanticist21-20261105-next10/circle-width-matrix-2020.pdf) | `43bd2866856b7f94b202fe4ff10a25faccd7d5e07db5f1dc567e3c2039481c4d` |
| [circle-width-matrix-2024.pdf](../../../../pdf/semanticist21-20261105-next10/circle-width-matrix-2024.pdf) | `9bd40e0acf0c4c38f753367c2b49c3f36c10ecc009b9dd46738febee8bbc2c71` |
| [borders-windows-2020.pdf](../../../../pdf/semanticist21-20261105-next10/borders-windows-2020.pdf) | `faaee897ccaf741c84bef1a32d4db575fd0334dabd51c97f572040118cafa732` |

실제 2024 제품 버전과 최종 원본의 Native/fresh WASM raw·직접 판독은 최종 검증 기록에 연결한다. 측정표 확보만으로 시각 수용을 선언하지 않는다.
