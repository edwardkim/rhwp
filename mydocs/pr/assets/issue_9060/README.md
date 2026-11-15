# PR #9075 검토 증적

제품 source `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`를 base `f8234f6f90cb25d9601c9d5d762e0a565aea19e4` 위에서 최종 검증했다. [최종 대상 문서 증적](final/main/README.md)과 [검증 기록](verification-record.md)을 정본으로 본다. 기존 독립 한컴 Print를 재인쇄하거나 원본을 바꾸지 않았다.

- 원 PR의 공개 86036 입력은 Native·fresh WASM 모두 전 14쪽 raw 100.00%다.
- 첫 줄은 5쪽, 나머지 두 줄은 6쪽, 후속 제목은 7쪽이며 전체 내용의 누락·중복이 없다.
- [같은 입력 4·5·6쪽 review·overlay와 전쪽 TSV](final/main/README.md)를 제출한다.
- [보호 전쪽 TSV](final/protected/README.md) 36개 데이터셋·3,542행은 상향된 기준 대비 하락 0행·누락 0행·추가 상승 0행이다. 앞서 수용한 30행의 상향 하한을 유지한다.
- [새 base의 동일 환경 대조](final/hft-style-coverage-tsv/README.md)는 Native/fresh WASM 총 778행이며 일반 허용폭 2%p를 넘는 하락이 없다. 특별 보호 하락 금지와 구분한다.
- 기본 전체 Rust: `Summary [1294.861s] 13583 tests run: 13583 passed (11 slow), 45 skipped`. Native 3종·필수 lint·workspace·신규 입력 보안·Studio 검사도 통과했다.
- [굵은 기호의 실제 보정](hft-bar-style-coverage-analysis.md)은 일반 중첩 표 대조군과 같은 backend의 base를 각각 보존한다.

루트의 초기 TSV·5·6쪽 PNG와 단계 문서는 당시 원인 분석·수정 근거다. 최종 증거는 위 final 경로를 따른다. 원시 로그·중간 run manifest·분석 JSON·중간 raster는 output에서만 관리한다. 글꼴 프로그램을 배포하지 않으며 100.00%를 RGB 동일성으로 주장하지 않는다.

로컬 검증 완료와 원격 CI·PR 생성·병합 완료를 구분한다. 실제 원격 결과는 후속 PR review에 기록한다.
