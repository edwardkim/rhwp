# 셀 공유 테두리의 한컴 Print 대조군

Issue: #7642. 이 입력은 선의 소유·중복 출력 여부를 구분하기 위한 합성 진단이며, 실제 문서의 시각 검증이나 회귀 추가 선행 조건을 대신하지 않는다.

- 입력: `shared-border-probe.hwpx`, SHA-256 `514be0031ff0aa655726e5ec3b6e60843128e64e7fe68bc50e630aea91d523c1`
- 독립 출력: `shared-border-probe-2020.pdf`, SHA-256 `beb4a047020eda0a0043d2d9ade3c5bb279edbb33a72d08888a2aea34ae0ab46`, 3쪽
- 생성: Hancom Office 2020 MCP, build `11.0.0.9136`, `hancom2020_pdf_driver_one_up`, `PrintMethod=0`
- 작업 ID: `d45ad9d8-986f-464b-83ea-b688c4816d6c`
- 상세 측정: ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage1252-shared-border-measurement.json`

첫 여섯 조건은 기존 `issue7642-border-owner-probe-20261015`와 같다. 추가 조건 7~10의 관측은 다음과 같다.

| 조건 | 공유 경계의 실제 PDF stroke |
|---|---|
| 양쪽 셀이 같은 선 | 동일 구간당 1회 |
| 양쪽 셀의 색·굵기가 다름 | 각 색·굵기별 1회 |
| 굵기는 같고 색만 다름 | 각 색별 1회 |
| 한쪽 셀이 NONE | 반대 셀 구간당 2회 |

상단 외곽선은 동일 구간이 2회 출력됐다. 모든 경계를 일률적으로 두 번 그리거나, 공유 경계에서 두 선 중 굵은 하나만 고르는 규칙의 독립 근거로 사용할 수 없다. 측정 좌표는 PDF point이며 합성 입력의 용지 밖으로 이어지는 구간과 실제 clip 안의 가시 구간을 구분한다. 원본 주거 계약서의 외곽·내부 경계에서도 별도로 대조했다. 구현 보정과 최종 Native/fresh WASM 대조는 진행 중이다.
