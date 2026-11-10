# #8856 SVG 주석 번호의 기존 조회 검사 정정

stage14 Runtime `c88877ee5a`의 첫 전체 회귀는 13,151 PASS/2 FAIL/45 skip이었다.
실패는 단일 SVG 문자열로 참조 번호를 찾던 두 검사뿐이다. 원 수정과 실제 SVG·독립 Print를
대조한 뒤 조회 방식만 고쳤다. 제품 Runtime과 좌표 기대값·판정 문턱은 바꾸지 않았다.

미주/쪽번호 원본은 PR #8319의 도입 커밋 `8c738f5434`와 전쪽 증거를 추적했다. 현재 Native/fresh WASM
전 3쪽 100%와 직접 판독 뒤, 단일 text와 같은 줄의 연속 text를 모두 내용·순서로 찾게 했다.
완료된 3쪽의 독립 Print 원점 ±2px와 기존 원문 소유 검사는 유지한다.

글상자 각주는 `8f007066df`의 각주 수집 계약을 확인했다. 한컴 2020 독립 Print를 새로 확보했고,
최소 글꼴 Native/fresh WASM 전 1쪽이 각각 100%다. 기존 텍스트 수집 helper로 번호 존재를 검사하며
본문/각주 본문 존재 검사를 유지한다. 이 함수는 관계형이며 위치 고정 검사로 보고하지 않는다.

두 묶음 6건의 집중 검사는 모두 통과했다. [전쪽 TSV·같은 쪽 raw/review/overlay·MCP 증적](../pr/assets/issue8856/stage15-svg-note-locators/README.md)에
변경 전 전체 실패와 현재 집중 통과를 구분해 기록했다. 최종 전체 회귀는 13,153 PASS/0 FAIL/45 skip,334.650초로 완료했다. Rust lint·workspace·Native Skia·manifest·unit tier·글꼴·브라우저 계약 게이트도 완료했다.
정본 보호 TSV와 실제 소비 경로 검증은 별도로 마무리 중이다.
