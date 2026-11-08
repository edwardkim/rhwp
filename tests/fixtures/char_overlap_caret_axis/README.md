# 글자겹침·주석 캐럿 축의 정식 입력과 Print 대조군

[원 회귀 검사](../../cases/char_overlap_caret_axis.rs)의 실제 `source`·`variants` API로 저장한
HWP/HWPX 20개와, 같은 글꼴·글자모양에서 겹침 payload만 한 글자로 줄인 대조군 2개다.
수동으로 글자모양이나 저장 줄 정보를 보정한 입력으로 대체하지 않았다.

- `nonbmp-endnote-{false,true}`: `앞🦦뒤끝`, 겹침 한 슬롯, 각주 또는 미주.
- `same-anchor-terminal-…-note-first-…-{left,right}`: 두 제어의 같은 앵커, 원래 제어 순서,
  문단 끝 또는 뒤 본문, 왼쪽·오른쪽 정렬.
- `single-payload`: 같은 `source("앞", 1)`에서 `CharOverlap.chars.truncate(1)`만 적용했다.
- 입력 22개와 각 입력을 실제 한컴 2020에서 Print한 PDF 22개를 함께 보존한다.
  [PDF 디렉터리](../../../pdf/issue7819_char_overlap_caret_axis)를 따른다.
- [입력·PDF SHA-256과 파일별 Native/fresh TSV 목록](../../../mydocs/pr/assets/issue_7819_caret_axis/source_evidence.tsv)에
  모든 입력을 연결했다. Native/fresh 측정 TSV는 각각 원본 그대로다.

## 검증 환경과 직접 판독

엔진 source SHA는 `8c32167d3d87f41beb5c6b045615f4f4e7df112d`다. Native142와 표준 release
fresh WASM048, Print 프로필, 96dpi, 등록된 원본 글꼴을 사용했다. 기준 PDF는 한컴
11.0.0.9136·PrintMethod 0·`hancom2020_pdf_driver_one_up`이다. 입력마다 전쪽 Native와 fresh
WASM의 최저 raw는 모두 100%이며 쪽수는 각각 1쪽으로 일치한다. 평균으로 대체하지 않았다.

원문과 각주·미주·뒤 본문의 보존, 겹침·주석의 같은 앵커 순서, 끝 주석의 빈 앵커와 본문 끝을
직접 확인했다. HWPX의 큰 내부 글자는 같은 HWPX Print에도 있다. 형식별 원본의 일치와
서로 다른 형식의 화면 동일성을 혼동하지 않는다. 원본에 있던 크기 값을 보기 좋게 바꾸지 않았다.

작업 중 명령·영수증·확대 자료는 ignored `output/pr-review/semanticist21-20261019/`의
`caret-original-current-api-native142-2191`, `caret-original-current-api-wasm048-2206`,
`single-payload-native142-2217`, `single-payload-wasm048-2237`, `caret-contract-reviews-2238`,
`caret-contract-detail-2242`에 있다. 최종 PR의 엔진이 바뀌면 필요한 재검증을 수행한다.

## 회귀 계약과 제한적인 px 보호

기본 검사는 논리 한 슬롯, source 제어 순서·원문 보존, 실제 렌더 경계와 캐럿·hit-test·선택
영역의 관계다. 겹침 폭은 같은 글꼴의 한 글자 대조군과 비교한다. 공개 선택 JSON의 x와 width는
각각 소수 첫째 자리로 직렬화되므로 각각을 독립적인 본문 시작·끝 관계로 검사한다.

`issue_7819_print_qualified_overlap_note_caret_boundaries_stay_within_two_pixels`는 완료된
`nonbmp-endnote-{false,true}.{hwp,hwpx}`의 첫 쪽만 ±2px로 보호한다. 원점은 PDF 좌상단,
단위는 96dpi px다. Print 본문에서 앞·테두리·비 BMP 문자·주석·뒤·끝의 glyph x 원점은 각각
113.378550, 126.331533, 139.284515, 157.514628, 166.309865, 179.262797이다.
본문 끝 192.215729는 마지막 두 한글의 같은 advance를 더한 독립 Print 값이다.
정확한 문단 소속·원문·전체 쪽수와 기존 관계 검사도 함께 유지한다. 허용 조건은 해당 페이지의
Native/fresh raw 각각 98% 이상·확대 판독이며, 이 px 보호를 미달 쪽으로 확대하지 않는다.
