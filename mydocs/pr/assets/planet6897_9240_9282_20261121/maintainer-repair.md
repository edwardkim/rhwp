# 통합 검토 중 발견한 글상자·표·OLE 차이

제품 코드 `cef9ff9acc08f92431e89450945462514fffa772`를 Native와 fresh WASM에서 확인했다. 기준은 같은 원본을 데스크톱 한컴의 Hancom PDF 프린터로 인쇄한 PDF다. 출력 세대·입력 해시·MCP 영수증은 [인쇄 기록](mcp-print-verification.json)에 있다. 웹한글기안기 PDF는 공식 비교에 사용하지 않는다.

## #9240 고베대지진 목차와 이어지는 표

2쪽 목차의 글상자에서 오른쪽 점 리더 탭 정지는 문단의 실제 오른쪽 경계를 넘어갔다. 한컴 2020 Print는 쪽번호를 글상자 오른쪽에서 정렬했다. Native의 제3장 쪽번호 원점은 621.4px였고, 독립 Print의 471.12pt는 96 DPI에서 628.16px이다. 이 조건에서 저장한 탭 전진을 재사용하지 않고 기존 같은 런·교차 런 오른쪽 정렬로 계산한다. 수정 후 원점은 628.1px이며 2쪽 Native 일치율은 97.49%에서 99.87%로 올랐다. fresh WASM도 99.87%다.

8쪽은 #9277의 조각 그리드가 1월 18일 칸의 #FAFABF 배경을 본문보다 먼저 그린다. 통합 충돌은 이 조각 경로를 유지해 해소했다. 원본 HFT 공급 조건에서 8쪽 양쪽 raw 99.99%이며, 배경 위쪽 좌표는 Print의 215.284pt에서 계산한다. 1쪽 원본 HFT 부재 검사는 HY견고딕이 표시 대체로 선택되는 관계를 확인한다.

[고베대지진 전쪽 TSV·좌표](coordinate-evidence/886/evidence.json) · [2쪽 review](coordinate-evidence/886/wasm/review_002.png) · [2쪽 overlay](coordinate-evidence/886/wasm/overlay_002.png) · [8쪽 review](coordinate-evidence/886/native/review_008.png) · [8쪽 overlay](coordinate-evidence/886/native/overlay_008.png)

## #9269 고령자 취업지원 본문과 OLE 차트

12·16쪽 원본 굴림체의 작은 네모(U+25AA)는 TTC face의 512/1024em 전진을 가진다. 한컴 인쇄도 네모 다음 글자까지 반각으로 진행했다. 통합 전 측정은 해당 글자 하나를 1em으로 잡았다. 원본 굴림체에서 확인된 문자 폭을 공통 측정 경로에 공급한다. 제목 표가 11쪽 각주 위에 겹치지 않고 12쪽으로 이동하는 원 PR의 동작은 그대로 보호한다.

15·21쪽은 OLE의 내부 차트와 HWP 레코드의 바깥 선을 각각 그려야 한다. 두 원본 레코드가 BinData ID 3·4와 가는 실선 정보를 갖는다. 26바이트 레코드에서 원본 필드를 읽고, 미리보기 바로 다음에 외곽선을 그린다. 차트 속 EMF에는 음수 LOGFONT 높이와 TA_TOP 원점이 있다. 원본 메트릭을 확인한 글꼴에서 문자 셀 위 원점을 GDI 정수 기준선으로 내려 글자 위치를 복원했다.

12·15·16·21·43쪽은 Native와 fresh WASM에서 각각 98.00%를 넘었다. 12쪽 제목 표, 15·21쪽 차트 외곽선, 43쪽 캡션 표 구분선은 Print 독립 좌표를 ±2px로 검사한다. 12·16쪽 네모는 원본 폭, 43쪽 표는 쪽 소유·한 번 그리기·후속 표와의 순서도 검사한다. 전쪽 최저는 Native 98.87%와 fresh WASM 98.86%로 둘 다 24쪽이다.

[고령자 취업지원 전쪽 TSV·좌표](coordinate-evidence/9239/evidence.json) · [21쪽 review](coordinate-evidence/9239/native/review_021.png) · [21쪽 overlay](coordinate-evidence/9239/native/overlay_021.png) · [43쪽 review](coordinate-evidence/9239/wasm/review_043.png) · [43쪽 overlay](coordinate-evidence/9239/wasm/overlay_043.png)

## #9278·#9282의 100.00% 경계

#9278의 6·7쪽은 이어지는 표가 맡은 쪽과 후속 표의 아래 선을 검사한다. #9282의 2쪽은 같은 문단에 있는 표 줄과 도형 줄의 순서와 각 위쪽 경계를 검사한다. 해당 페이지의 Native와 fresh WASM 모두 공식 표시값 100.00%를 확인했다. 독립 Print 좌표의 허용 오차는 ±1px다.

[#9278 전쪽 TSV·좌표](coordinate-evidence/9272/evidence.json) · [#9282 전쪽 TSV·좌표](coordinate-evidence/9279/evidence.json)

## WMF 통합 후 다시 확인한 범위

제품 코드 `19646e69fc9`를 포함한 `9d6eee31ea`의 전체 nextest 13,876건과 Native Skia 4,034건이 통과했다. Native/WASM Clippy, 워크스페이스 빌드·Clippy, 검사 매니페스트도 완료했다. Studio는 같은 제품 코드로 다시 빌드한 WASM의 초기화 응답 60건을 확인했다. [Rust 검증 결과](validation/rust-v4.json) · [Studio 검증 결과](validation/actual-studio-v4.json)

기존 보호값보다 낮게 관측한 fresh WASM 32쪽의 입력·Print PDF·글꼴 공급·래스터 조건을 고정해 `devel`과 후보를 비교했다. 32쪽 모두 PNG 바이트가 같았다. 이 결과는 해당 쪽의 코드 변경 영향 대조이며 보호값 재현은 별도로 확인 중이다. #8040은 원본에 기록된 신그래픽체를 공급한 별도 전쪽 대조에서 보호값을 되찾았다. [32쪽 대조 결과](validation/protected-wasm-observed-declines-v4.json) · [#8040 글꼴 공급과 전쪽 TSV](protected-issue8040/README.md)
