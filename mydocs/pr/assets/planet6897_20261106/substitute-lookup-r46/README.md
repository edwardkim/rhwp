# 문서 대체 글꼴 선택 검사의 독립 근거 재확인

전체 Rust 실행에서 실패한 기존 검사의 제목 위치와 선택 프로그램을 확인했다. 새 조판 개선을 추가한 범위가 아니다.
원 해결 PR #8191, 반영 PR #8754와 [Stage35 독립 Print 근거](../../issue7445/completed-20261009/stage35-20261021/README.md)를 먼저 대조했다.
제품 source는 `65195e9730f35bdbd7ea0c9a393e07e66125202e`이며 기존 원본·Print PDF를 재산출하지 않았다.

[실제 전쪽·46쪽 raw 원장](measurements.tsv)은 96 DPI·print profile·색상 경계 보정 전 2px 내용 실루엣이다.
HWP/HWPX 각각 46쪽이며 Native/fresh WASM의 누락·90% 미만 쪽은 없다.
46쪽의 review와 standalone overlay를 두 경로에서 직접 읽어 제목·하단 문구의 소속·순서·위치·닫는 괄호를 확인했다.
raw100은 RGB 동일성을 뜻하지 않는다. 이전 Stage35의 수치를 현재 실측으로 표시하지 않는다.

실물 시각 검증은 설치된 시스템 전체 글꼴을 사용했다. 정식 Rust 검사는 mock 가용 프로그램 ID를
공급하는 계약으로서 사유 글꼴의 실제 설치나 글꼴 바이너리에 의존하지 않는다.
HWP의 `Haansoft Batang,한컴바탕`과 HWPX의 `Haansoft Batang`은 선두 선택 프로그램이 같다.
CSS 후보 목록 전체를 단일 문자열과 비교하던 assertion만 선두 family의 정확한 비교로 바로잡는다.
독립 Print 시작44.16pt·끝213.78pt의 ±2CSSpx, 셀 소유·내용 순서·겹침·원본 직렬화 검사는 유지한다.

|원본|Native 전쪽|WASM 전쪽|Native 46쪽|WASM 46쪽|
|---|---|---|---|---|
|HWP|[hwp-native.tsv](hwp-native.tsv)|[hwp-wasm.tsv](hwp-wasm.tsv)|[review](hwp-native-p046-review.png) · [overlay](hwp-native-p046-overlay.png)|[review](hwp-wasm-p046-review.png) · [overlay](hwp-wasm-p046-overlay.png)|
|HWPX|[hwpx-native.tsv](hwpx-native.tsv)|[hwpx-wasm.tsv](hwpx-wasm.tsv)|[review](hwpx-native-p046-review.png) · [overlay](hwpx-native-p046-overlay.png)|[review](hwpx-wasm-p046-review.png) · [overlay](hwpx-wasm-p046-overlay.png)|
