---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 15. 채점은 라이브다 — 휴게실이 바꾸지 않는 것

이 페이지는 방문자가 오해하기 쉬운 채점 성질을 적는다. **채점 논리를
바꾸라는 지시가 아니다.** `gym/core/checks.py` 의 연산자 등록부와
`gym/core/runner.py` 의 채점 절차가 정본이다. 휴게실·PARK·INVITE·
`gym/docs/tutorial.md` 는 장식과 안내다.

돌아가기: [README.md](README.md) · PARK 정직 조항: [../PARK.md](../PARK.md)

## 정직 조항 (PARK 와 같은 말)

- 점수는 pack 별로 보존되고, 총점은 편의값이다.
- 기대값은 골든 파일로 박제하지 않고 **채점 시점에 rhwp 로 재계산**한다.
- 보스 어트랙션도 예외 없이 기준 풀이 왕복을 통과해야 등재된다.
- 부재는 실패로 위장하지 않는다. 요구 명령이 없는 pack 은 `unavailable`.

놀이공원이라 부르는 이유는 사람을 부르기 위해서지, 판정을 무르게 하기
위해서가 아니다.

## 연산자 등록부 — 읽기만

`gym/core/checks.py` 의 `REGISTRY` 는 지금 이 서른세 이름이다. 휴게실
작업이 서른네 번째를 추가하지 않는다. 시험이 이 집합을 잠근다.

파일 연산자(CLI 를 부르지 않음):

- `same_hash`
- `differs_from_input`
- `file_exists`
- `files_differ`
- `xml_root_eq`
- `json_value_eq`
- `csv_cell_eq`
- `utf8_bom`
- `json_len_eq`
- `csv_row_count_eq`
- `ndjson_count_eq`
- `ndjson_field_eq`
- `json_keys_contain`
- `text_line_eq`
- `json_type_eq`
- `json_len_ge`
- `json_array_item_eq`
- `csv_col_count_eq`
- `csv_header_eq`
- `csv_row_eq`
- `ndjson_keys_contain`
- `ndjson_len_eq`
- `text_line_count_eq`
- `text_line_contains`

봉투 연산자(CLI 를 부름):

- `answer_eq`
- `len_answer_eq`
- `len_ge`
- `value_eq`
- `value_ge`
- `value_in`
- `deep_contains`
- `not_contains`
- `cell_text_eq`

전역 훑기(`GLOBAL_SCAN_OPS`)는 `deep_contains` 와 `not_contains` 다.
편집 과제에서 좌표 없이 쓰면 스키마가 막는다(#4600). 그 막음은
`gym/core/schema.py` 가 이미 한다.

## 라이브 오라클이란

CR01 을 예로 든다. 저장소 어디에도 "정답은 3쪽"이라고 박제된 숫자가
없다. 채점기가 그때 `rhwp info samples/table-001.hwp --json` 을 돌려
`pageCount` 를 읽는다. 픽스처가 진화하면 정답도 따라 진화한다.

그래서 이 안내의 예시 숫자(`{"pages": 3}`)는 **설명용**이다. 네
바이너리가 다른 수를 말하면 그 수가 정답이다.

## 기준 풀이(reference/)의 자리

`reference/<id>.json` 은 "이 과제를 풀 수 있다"는 선언이다. 채점기가
방문자의 답을 이 파일과 문자열 비교하지 않는다. 기준 풀이는 과제를
등재할 때 왕복으로 실측하는 데 쓴다. 방문자가 그걸 베끼면 채점은
통과할 수 있어도, 측정되는 능력은 "따라 치기"다.

## 이 작업이 명시적으로 하지 않는 것

이슈 #5263 의 범위다.

- `gym/core/checks.py` 를 고치지 않는다.
- 다른 열린 PR 의 pack 과제 JSON 을 고치지 않는다.
- 새 check 연산자를 추가하지 않는다.
- 점수 가중·만점 계산·unavailable 규칙을 바꾸지 않는다.
- `cargo fmt --all` 을 돌리지 않는다 (Rust 변경이 없다).

잠그는 시험은 `scripts/tests/test_gym_tutorial.py` 다.
