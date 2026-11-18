# #8863 후보 출력 최소 증적

- `native-silhouette.tsv`, `wasm-silhouette.tsv`: 각 backend 전 19쪽 raw 수치.
- `*-silhouette-manifest.json`: 원본 manifest를 바이트 변경 없이 보존했다. 입력/PDF/producer/글꼴 해시와 원시값이 포함된다.
- `*-compare-page1.png`: 변경 위치가 포함된 1쪽 독립 Print PDF 비교.
- `wasm-export-complete.json`: 실제 WASM 쪽수·브라우저·worker별 로딩 글꼴 근거.
- `hancom-print-evidence.json`: 독립 편집·Print 설정과 결과 해시.
- `SHA256.json`: 보존한 파일의 해시와 바이트 수.

PDF는 `pdf/issue8863/hancom-edited-expected-2020.pdf`에 있다.
명령·검증 범위·한계는 `mydocs/report/task_m100_8863_report.md`를 따른다.
manifest의 절대 경로와 `output/` 참조는 원 실행 provenance이며 영구 파일의 위치를 뜻하지 않는다.
전 19쪽 PNG는 복제하지 않았으므로 필요하면 입력·PDF·producer·글꼴 조건을 맞춰 재생성한다.

두 backend 결과는 모두 실제 수정 후보와 독립 한컴 PDF의 비교다.
별도의 base/head renderer 쌍을 산출하지 않았으므로 backend별 devel 대비 2%p 무회귀를
이 증적만으로 입증하지 않는다. Native와 Canvas는 실제 글꼴 공급 경로도 구분한다.
