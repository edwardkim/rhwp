# #8174 v2-S40 — lazy asset·진단 JSON의 공개 저장 경계

Issue: #8174. 2026-10-26, 기준 `da5b26bff8`.

`tests/cases/issue_8174_public_storage_boundaries.rs`의 공개 API 검사 **2 PASS**를 확인한 뒤
대응하는 Allocation source unit 2개를 제거했다. 나머지 private decimal/숫자·clone·guard
검사를 이 결과만으로 제거하지 않았다. 생산 코드는 변경하지 않았다.

- 실제 PDF Session 첫 HWP 저장에 공개 `BinDataResolver`를 연결했다. 크기 상한 조회와
  실제 읽기를 별도 카운터로 관측한다. 공개 사용량 사이에서 읽기 전 거절 경계를 찾고,
  known/opaque 상한 모두에서 호출 0·실패 고정·낮은 cap 보존을 확인했다. 다음 정상 요청은
  실제 데이터를 읽고 정상 산출물과 같은 HWP bytes로 복구한다. private Storage나 요청
  중간 cap 조작으로 대신하지 않았다.
- 상세 표 추론을 포함한 실제 진단을 공개 metadata의 독립 Value 인코딩과 비교했다.
  정확한 backing 길이만큼 누적 계상이 증가하며 출력 cap 직전 거절은 backing을 쓰지 않고
  요청을 닫는다. 최초 기대값은 in-memory f64와 JSON parser 결과를 직접 비교해 실패했다.
  `float_roundtrip`이 꺼진 parser의 1 ULP roundtrip 차이를 확인하고, 독립 reference도 같은
  JSON wire roundtrip을 거친 뒤 **정확히 같은 Value**인지 비교했다. 오차 허용치를 추가하거나
  생산 인코더를 바꾸지 않았다. 실패 로그는 `initial-json-reference-failure.log`에 보존했다.

원본 source 전체·Git blob·source SHA·제거 함수와 실행 명령/로그는 ignored
`output/pr-review/issue8174/v2-s40-20261026/`에 보존한다. 집중 실행은 기존 공유 target의
동일한 생산 API에 직접 연결했으며 최종 자동 suite는 T08에서 수행한다. source unit
정책은 이 절편 기준으로 감소하며, 브랜치 전체의 신규 모듈/지원 항목과 base 동기화 차이는
계속 남는다. 전체 PR 준비 완료로 보고하지 않는다.
