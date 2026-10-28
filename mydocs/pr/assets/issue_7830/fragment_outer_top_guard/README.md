# #7830 원본에 다시 추가된 첫 표 조각 검사 보관

같은 `samples/hwpx_sample2.hwp` 전 29쪽은 사용자의 선행 이관 지시에 따라
[#7830](https://github.com/edwardkim/rhwp/issues/7830)에 보관 중이다. PR #7965의
`e64b513ed5faf7392ab187b7cb70bb112cbe05d1`은 이 원본의 6쪽 검사를 다시 추가했다.
현재 기준 `80968497c`와 후보 `ca26039dad`를 같은 입력·독립 2024 Print로 비교했지만,
후보 전쪽 최저는 Native 47.81904%, fresh WASM 48.38627%이며 4/8/13/14쪽이
90% 미만이다. 6쪽 한 곳의 높은 점수로 전쪽 조건을 대신하지 않는다.

새로 발생한 2쪽의 하락은 별개의 현재 구현 회귀다. 이를 이관으로 면제하지 않고
공통 글꼴 메트릭 출처를 수정했다. `c96354d4e6`의 Native 2쪽은 99.83257%로
기준 점수를 회복했으며, 이 기록 시점의 새 fresh WASM 검증은 진행 중이다.
보관은 문서의 해결 또는 최종 검증 통과를 뜻하지 않는다.

제거 전 모듈 전체와 helper를 `.rs.txt`로 보존했다. source commit·Git blob·
SHA-256·원본/보존 경로·입력 및 PDF 해시와 양쪽 측정 기록은
[`preservation.json`](preservation.json)에 있다. 제외하는 함수는
`deferred_first_fragment_spends_its_offset_and_opens_the_outer_top` 한 개다.
1342000의 이어받기·되감김 검사 두 개와 156555538의 정상 대조군 한 개는 유지한다.
구현과 원본 HWP·PDF 및 범위 밖의 정상 API 검사는 제거하지 않는다.

복원은 같은 원본 전쪽의 Native/fresh WASM raw 최저 90% 이상, 독립 Print의
쪽수·내용 대응과 확대 판독을 확인한 뒤 수행한다. 개체 소속·순서·누락·중복·
포함 관계를 보호하며, 해당 완료 쪽이 양쪽 98% 이상이면 독립 좌표를 ±2px로
보완할 수 있다. 현재의 0.5px 핀을 검증 없이 다시 활성화하지 않는다.
