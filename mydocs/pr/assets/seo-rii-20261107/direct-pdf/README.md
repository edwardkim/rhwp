# P81 실제 Direct PDF 소비 확인

source `cda7c8cb33`의 Native CLI에서 `export-pdf --backend direct --profile print --compat 2022`로 기존 공식 원형 대조군 6쪽을 내보냈다. 생성 PDF를 PDFium으로 96 DPI raster한 뒤 같은 독립 한컴 Print PDF와 비교했다. [전 6쪽 TSV](radial-direct-native.tsv)는 모두 raw 100%다. [같은 쪽의 실제 PDF/독립 Print 비교](radial-direct-native-print.png)에서 정사각형·가로/세로 상자·0/100% 중심의 물리 반경·색 변화와 표식 1–6을 직접 확인했다. Native PNG·WASM Canvas와 별개의 실제 PDF 소비 경로 증거다.

입력: `tests/fixtures/gradient/radial-corner-print-contract.hwpx`, 독립 기준: `tests/fixtures/gradient/radial-corner-print-contract-2020.pdf`. 입력·PDF SHA는 공통 시각 원장에 기록한다. 임시 생성 PDF·실행 로그는 커밋하지 않는다.
