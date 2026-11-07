# P81 실제 Direct PDF 소비 확인

source `cda7c8cb33`의 Native CLI에서 `export-pdf --backend direct --profile print --compat 2022`로 기존 공식 원형 대조군 6쪽을 내보냈다. 생성 PDF를 PDFium으로 96 DPI raster한 뒤 같은 독립 한컴 Print PDF와 비교했다. [전 6쪽 TSV](radial-direct-native.tsv)는 모두 raw 100%다. [같은 쪽의 실제 PDF/독립 Print 비교](radial-direct-native-print.png)에서 정사각형·가로/세로 상자·0/100% 중심의 물리 반경·색 변화와 표식 1–6을 직접 확인했다. Native PNG·WASM Canvas와 별개의 실제 PDF 소비 경로 증거다.

입력: `tests/fixtures/gradient/radial-corner-print-contract.hwpx`, 독립 기준: `tests/fixtures/gradient/radial-corner-print-contract-2020.pdf`. 입력·PDF SHA는 공통 시각 원장에 기록한다. 임시 생성 PDF·실행 로그는 커밋하지 않는다.

최신 devel은 같은 입력의 첫 쪽에서 non-linear gradient를 거부했고 PDF가 생성되지 않았다. [벡터 출력 전후 기록](vector-before-after.json)을 따른다. 보정 후보는 6개 `/ShadingType 3`와 0개 `/Subtype /Image`를 갖는 PDF를 만들었다.

대표 p1·p3·p6의 actual Direct PDF raster에서 canonical Visual Sweep review·standalone overlay를 추가했다. [실제 페이지별 지표](representative-direct-pdf-metrics.json)를 보존하며 Native PNG를 PDF 출력인 것처럼 대신하지 않는다.

| 실제 Direct PDF·독립 Print 대응 쪽 | review | overlay |
| --- | --- | --- |
| p1 ↔ p1 | [review](direct-pdf-p001-review.png) | [overlay](direct-pdf-p001-overlay.png) |
| p3 ↔ p3 | [review](direct-pdf-p003-review.png) | [overlay](direct-pdf-p003-overlay.png) |
| p6 ↔ p6 | [review](direct-pdf-p006-review.png) | [overlay](direct-pdf-p006-overlay.png) |
