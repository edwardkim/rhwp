# #7687 미완료 원본의 신규 머리말 채움 검사 보관

PR #8010의 `a9672907e47edc8fafe191711aa4251e66aefe0b`은 실제 hwpspec 원본의
4/19쪽 머리말 채움 검사 두 개를 추가했다. 같은 원본은
[#7687](https://github.com/edwardkim/rhwp/issues/7687)에 전쪽 검증·복원 범위가
보관돼 있다. 기준 `80968497c`는 72쪽, 후보 `ca26039dad`는 74쪽이며 독립 공식
Print는 71쪽이다. 일부 머리말의 관계 검사가 통과해도 원본의 쪽수·전쪽 시각
선행 조건이 충족된 것으로 세지 않는다.

머리말 채움 구현 자체는 이미 upstream에 반영돼 있다. 이 보관은 머리말이 현재
그려지지 않는다는 주장이나 #8008의 재개방이 아니다. 이미 이관된 원본에 추가된
새 검사를 정상 회귀로 수용할 수 없는 상태를 기록한다. 구현과 HWP·공식 PDF,
같은 원본을 사용하는 비시각 parser·직렬화 계약 검사는 보존한다.

제거 전 모듈 전체와 helper를 `.rs.txt`로 보존했다. source commit·Git blob·
SHA-256·원본/보존 경로·입력과 PDF 해시는 [`preservation.json`](preservation.json)에
기록했다. 제외하는 함수는 `header_paragraph_fill_is_painted_behind_the_header_text`,
`header_fill_color_reaches_the_svg` 두 개다. 범위 밖의 정상 검사는 제외하지 않는다.

복원은 같은 원본 전체의 쪽수·내용 대응, Native/fresh WASM raw 최저 90% 이상과
독립 Print의 직접 판독을 확인한 뒤 수행한다. 채움의 뒤쪽 그리기 순서와 글줄
포함 관계를 유지한다. 원본의 전쪽 미완료를 목표 머리말의 통과로 대신하지 않는다.
