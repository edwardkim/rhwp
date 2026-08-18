# 10 — auto_number 정책

빌더는 `auto_number: true`(기본) 이면 첫 stem 텍스트 앞에 `{number}. ` 를 붙인다.
Skill 이 번호를 이미 썼으면 `false` 로 끈다. 이 필드를 끄기 위해 빌더를
고치지 않는다. JSON 한 비트다.

## 표

| 상황 | auto_number | 첫 텍스트 | 출력 |
| --- | --- | --- | --- |
| 일반 문항 | `true` 또는 생략 | `다음 글의 주제는?` | `1. 다음 글의 주제는?` |
| 공유 지문 지시문 | passages 사용 | `[1~3] 다음 글을…` | 지문에만, 번호 prefix 없음 |
| 사용자가 번호를 이미 씀 | `false` | `2. ㉠에 해당하는…` | `2. ㉠에 해당하는…` |
| MD 헤더 `## 4.` | `false` | `4. …` | `4. …` |
| 보기 본문 | 문항은 true | `[보기]를 참고하여…` | `12. [보기]를 참고하여…` |

## 권장

가능하면 `auto_number: true` + stem 은 prefix 없이.
공유 지문은 `passages` 로 분리.
번호를 원본에서 그대로 옮기고 싶을 때만 `false`.

## 중복 사고

Vision 이 `"3. 다음 중"` 을 그대로 stem 에 넣고 `auto_number` 를 생략하면
산출은 `"3. 3. 다음 중"` 이 된다. 같은 사고로 `"2. 2. ㉠에 해당하는…"` 도
생긴다. export-text 게이트에서 `"N. N."` 패턴을 검사한다 (18_verify_gate.md).

## 그룹 지시문을 문항 번호로 착각

`[1~3] 다음 글을 읽고 물음에 답하시오.` 를 문항 1 의 stem 으로 넣고
`auto_number: false` 로 끄면, 실제 문항 1 발문이 사라진다.
지시문은 passage, 발문은 question.

## 번호 체계가 이상한 시험지

학원지: `문1`, `Q1`, `1)`, `[01]`.
빌더 prefix 는 항상 `"{number}. "` (숫자+마침표+공백) 이다.
원본이 `문1` 이면 `auto_number: false` 로 원본 표기를 stem 에 남긴다.
`문1` 을 스키마에 넣는 필드는 없다.

## 미지정

필드가 없으면 Rust `default_auto_number() -> true`.
명시적으로 `true` 를 적어도 된다. 픽스처는 둘 다 가진다.

## 타입이 틀린 경우

`"auto_number": "yes"` 또는 `1` 은 serde 가 거부한다.
invalid 픽스처: `fixtures/schemas/invalid_auto_number_type.json`.

픽스처 행렬: `fixtures/matrices/auto_number.json`.
