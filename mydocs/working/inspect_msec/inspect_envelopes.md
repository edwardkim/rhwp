# inspect 계약 봉투 작업 기록 (#5476)

이 장은 기존 규칙의 소비 분기만 적는다. 새 kind 를 제안하지 않는다.
개별 봉투는 `tests/fixtures/inspect_msec/envelopes/` 가 정본이다.

## 가족 `exception` (2건)

- 양성 0 / 음성 0 / 그 외 2
- 대표 `ex-inspect-no-axis`
- 출처 `src/main.rs` `inspect_command`
- 대표 분기: {'branch': 'stdout empty', 'doNotParseStdoutAsJson': True, 'stderrIsDiagnosis': True}
- 왜: 축 없음. 수복 줄을 지어내지 않는다(오제안 0).

- `ex-inspect-no-axis` polarity=exception exit=2 pair=-
- `ex-inspect-unknown-axis-utf8` polarity=exception exit=2 pair=-
