# sample10 474쪽의 실제 기호 프로그램 공급 재현

기존 본문 프로그램의 선택은 보존하고 누락 기호만 독립 Print에서 확인한 `Haansoft Batang`으로
공급하자 Native와 fresh WASM의 공식 raw 표시값이 99.96%에서 100.00%로 복원됐다.
[집중 결과](focused-results.json)의 각 review·overlay를 직접 판독해 등호 10개 뒤 이중선 25개의
굵기와 끝 위치, 제목·본문 줄 소속을 확인했다. **Native 전 763쪽은 보호 하락 0쪽·최저 99.57%로 완료했다. WASM 전 763쪽도 보호 하락 0쪽·최저 99.57%로 완료했다.**

입력은 `samples/hwp3-sample10.hwp`, 기준은 독립 한컴 2020 Print
`pdf/issue8371/hwp3-sample10-2020.pdf`다. 소스·원본 문자·기준 PDF·96 DPI·compat 2022·
2px 측정법·보호 하한을 바꾸지 않았다. 원 계약 #5793 → #5829 → #5844와 후속
`hft_mapped_symbol_program_tracking` 검사의 기호 전진·본문 꾸밈 분리도 유지했다.

## 이름이 같은 글꼴과 실제 원 글리프의 구분

PDF의 실제 기호는 `Haansoft Batang` 부분집합 glyph18이며 전진은 512/1024em이다.
[원본 대조](pdf-rule-source-glyph-matches.json)에서 확보한 TTF
`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`의 U+F0827이
윤곽 SHA와 전진 모두 일치했다. U+2550은 1024/1024em이며 다른 윤곽이다. 따라서
원 PUA를 이름이 비슷한 Unicode 이중선으로 표시하는 것과 원 글리프 공급을 구분한다.
PDF의 FontFile은 읽기 전용 진단에만 사용했고 렌더링 글꼴을 추출·생성하지 않았다.

정확한 HBATANG 파일을 기존 목록에 추가하는 것만으로는 99.96%가 유지됐다. 명시 환경을
빈 대체 목록으로 새로 만들면 자동 본문 선택이 사라져 제목 폭도 달라졌으며
Native 95.79%·WASM 95.93%로 악화했다. 이 실패 조건은 채택하지 않았다.

[Native 환경](native-environment.json)과 [WASM 환경](wasm-environment.json)은 각각 직전 실제
자동 프로그램 선택을 그대로 보존하고 `glyphFallbacks`에 기호 공급을 추가한 것이다.
본문 대체·자연 전진·원 메트릭 정책은 새로 바꾸지 않았다. 측정과 paint는 기존 공통 PUA
공급 경로를 소비하며 제품 코드에 문서별 분기를 넣지 않았다.
[브라우저 소비 증거](consumer-font-proof.json)에는 `HY Junggothic,Haansoft Batang`의 실제
PUA 공급자와 원본 SHA·브라우저 정규화 후 SHA를 구분해 기록했다. 비공개 글꼴 파일과
윤곽 좌표는 Git에 포함하지 않는다.

[Native 전쪽 결과](native-full-results.json)와 [전763쪽 TSV](native/full-silhouette.tsv)에 동일 공급 조건의 전체 기록을 보존한다. 474쪽은 100.00%다.

[WASM 전쪽 결과](wasm-full-results.json)와 [전763쪽 TSV](wasm/full-silhouette.tsv)에서도 보호 하락0이다. 양쪽474쪽은100.00%다. 전체 PR 필수 검증은 최신 원장에서 별도로 확인한다.
