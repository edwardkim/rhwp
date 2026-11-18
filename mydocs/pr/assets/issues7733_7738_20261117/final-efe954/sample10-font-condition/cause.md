# sample10 원본 HFT·HNC 공급 조건 복원

원 해결은 `99a176b7cb34fde54454c03433baa1382b2de9fa`(#8397)다. 원본 HFT의 기호 전진과 영문 슬롯 전진을 구분해 보존했고, 후속 `3f1619339f066ab9f4757948b6f1e78013d3988c`는 sample10 474쪽 HNC 이중선의 Haansoft Batang 반각 프로그램과 본문 HFT 굵기 자간을 분리했다. 최신 보호 기준은 코드 `8ad1c5c08aa4cd68accbd2af6b9c60bc458d8992`의 전쪽 수용 기록이다. 원본과 독립 데스크톱 한컴 2020 Hancom PDF Print PDF는 변경하지 않았다.

현재 Mac에서 JSON 없는 자동 선택은 `한양중고딕` 요청에 설치된 TTF `H2GTRM.TTF`(내부 `HYGothic-Medium`·`HY중고딕`)를 선택했다. 이 TTF에는 U+2550 `═`가 없다. 현재 자동 선택 경로에서 측정한 0.5em은 해당 TTF의 실제 hmtx가 아니라 결측 대체 결과다. 원본 HFT 분할 프로그램 `SPJGT.otf`는 내부 `HY Junggothic`·`한양중고딕`, CFF이며 U+2550의 전진은 1000/1000em이다. 독립 PDF 2쪽의 Type3 T4 역시 20자 약160pt·글자 크기 약8pt로 전각 전진을 기록한다. 두 파일은 같은 중고딕 계열이지만 동일한 프로그램이나 동일 문자 집합이 아니다. 형식·이름·문자 수·직접 측정값은 [파일 비교](font-program-comparison.json)에 있다.

표시 글꼴 설정을 통째로 비우고 원본 폭만 보존한 진단은 2쪽을100%로 복원했으나 다른 두 쪽을95.78/96.63%로 악화시켜 수용하지 않았다. 실제 관측한 다른 프로그램 선택과 영문 전진을 유지하면서 HFT 중고딕 원본 family를 공급했다. 474쪽의 남은25픽셀은 제목 밑 이중선 끝부분이었다. Native 실제 글꼴 영수증에서 해당 HFT 요청의 결측이 OS HCR Dotum으로 내려갔고, 독립 PDF는 Haansoft Batang의 원 HNC 프로그램을 사용했다. 해당 HFT family의 검증된 결측 공급 체인에 Haansoft Batang을 명시했다. 일반 제품 코드의 TTF→TTC→OpenType 우선순위와 다른 문서의 자동 선택은 바꾸지 않았다.

최종 [환경](font-environment.json)은 원본 HFT family와 검증된 HNC 공급자를 명시하며, 영문 등 나머지 관측 프로그램은 유지한다. HFT family의 프로그램 선택을 임의 파일 해시로 인증하지 않고 실제 제공 파일·face와 출력 영수증으로 확인한다. `preserveSourceMetrics`는 true다. 이 결과를 일반 자동 환경의 제품 개선으로 확대하지 않는다.

현재 코드 `efe9540bd4f1cbaddd89bf018771ef1de20389c8`의 fresh Native/fresh WASM 집중 재검증은 각각 2쪽100.00%,474쪽100.00%,690쪽99.94%다. 기존 보호 하한100.00/100.00/99.94를 유지했으며 기준 TSV·DPI·PDFium·2px 계산·원본 텍스트를 변경하지 않았다. [Native 집중 TSV](native/focused-silhouette.tsv)·[WASM 집중 TSV](wasm/focused-silhouette.tsv)는 세 쪽의 복원 근거이며 전763쪽 수용을 뜻하지 않는다. 같은 환경의 고정 base/후보 Native/fresh WASM 전쪽 검증은 진행 중이다. 이전 실패 전쪽 출력과 진단 조건은 output에 보존한다.
