# Issue #7642: 저장 단 원점 보정의 Native 영향 쪽 대조

- 생산 코드: `a347dcb6931619b6cfa76b23d318983c563c079f`.
- 원본: `samples/hwp3-sample16-hwp5-2022.hwp`.
- 독립 한컴 2020 Print: `pdf/issue7642-sample16-legacy-print-recheck/hwp3-sample16-hwp5-2022-2020.pdf`.
- 원본 SHA-256: `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`.
- PDF SHA-256: `cf332d3372d9b5423b5941356b7685afc5c23865f8d8f4c1eb9131c4eca71bae`.
- Native `--compat 2022 --profile print --embed-fonts=full`, 96 DPI, 6–8쪽.
- 6쪽 96.92349%, 7쪽 99.21753%, 8쪽 99.78673%. 이전 7쪽 43.02118%에서 개선.
- 직접 판독: 7쪽의 표, 다음 제목·문단, 본문 외곽선과 하단 내용의 큰 세로 위치 차이가 해소됐다. 글꼴·세부 획 차이는 확대 판독을 추가하며, 이 세 쪽 결과로 전쪽 완료를 판정하지 않는다.
- 전체 64쪽과 fresh WASM은 미완료이므로 이 증거만으로 신규 회귀·PR 완료를 판정하지 않는다.
- 대표 review는 동일 실행의 PNG로 canonical 비교·overlay·review 함수를 통해 합성했다. 재출력과 혼동하지 않는다.
- TSV·실행·글꼴 전송 원본 복원 증거: ignored stage463 및 stage466 출력.
