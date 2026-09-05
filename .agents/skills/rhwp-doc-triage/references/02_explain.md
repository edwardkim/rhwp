# 02 — explain : 결정론 한 줄 요약

LLM 요약이 아니다. info·export-structure·export-tables·fields 의 템플릿 조립이다 (#3828).

이슈 #3828. 권위는 `cli_commands.md` 의 `explain` 절.

## 트리에서의 위치

사다리: `info → explain → export-structure → digest → search → extract-data`

이 단의 목적: 결정론 한 줄 요약. LLM 판정 아님

- 성공 다음: 표 많음→table-exchange, 누름틀→form-fill, 아니면 질문별 분기
- 실패 다음: info 와 같은 암호/런타임 규약

## 호출

```bash
rhwp explain <파일> --json
```

읽기 전용. 원본을 쓰지 않는다. 새 플래그를 만들지 않는다.

## 봉투 키

| 필드 | 필수 | 읽는 법 |
| --- | --- | --- |
| schemaVersion | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| source | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| format | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| pageCount | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| paragraphCount | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| tables | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| fields | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| footnoteCount | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| endnoteCount | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| encrypted | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |
| summary | 필수 | 결정론 한 줄 요약. LLM 판정 아님 |

`schemaVersion` 은 `"1.0"`. 필드 추가는 허용, 삭제·형 변경은 계약 테스트가 잡는다.

## 종료 코드

| 코드 | 의미 | 에이전트 행동 |
| --- | --- | --- |
| 0 | 성공. 0건 포함 | 봉투를 읽고 정지 규칙을 적용 |
| 1 | 런타임 (없음·파싱·암호 틀림) | stdout 비었는지 확인. 덤프 우회 금지 |
| 2 | 사용법 | 옵션을 고친다. 0 을 무제한으로 바꾸지 않는다 |

## summary 문장

형식·쪽수·문단 수·표 개수와 크기·병합·누름틀 이름·각주/미주·암호를 문장으로 조립한다.
표·누름틀 이름은 상위 N개로 자르지 않는다 (#3719 부분 목록 금지).
`tables[]` 는 셀 텍스트를 싣지 않는다. 내용은 `export-tables` 몫.

## 키 주의

`paragraphCount` 다. `info`/`digest` 의 `paraCount` 와 표기가 다르다.
`tables[].index` 는 0 기준, `table-to-csv --table` 과 같다.
사람 문장의 "표 1" 만 1 기준.

## 인계 신호

- `fields` 비어 있지 않음 + 채움 요청 → `rhwp-form-fill`
- `tables` 비어 있지 않음 + 표 작업 → `rhwp-table-exchange`
- `encrypted:true` → 비밀번호 규약. 내용을 추측하지 않는다

## 하지 않는 것

- summary 를 더 문학적으로 만들려고 전문을 읽기
- 표 셀을 explain 로 읽기
- 취지·논조를 explain 이 알려 준다고 말하기

## 운용 시나리오 C01~C60

1. 사용자가 '한 줄 요약' 라고 하면 이 명령을 쓴다 — `explain` 시나리오 C01.
2. 이 명령의 결과가 질문이 이미 충족 이면 즉시 정지 (S15) — `explain` 시나리오 C02.
3. 이 명령을 무제한 덤프 로 쓰면 컨텍스트 고갈 — `explain` 시나리오 C03.
4. 쪽수가 31~100쪽 일 때 이 명령의 예산은 메타만 — `explain` 시나리오 C04.
5. 이 명령 이후 프롬프트에 넣을 수 있는 것은 주소와 짧은 발췌 뿐이고 전문·전 셀 텍스트 은 버린다 — `explain` 시나리오 C05.
6. 사용자가 '표가 있는지' 라고 하면 이 명령을 쓴다 — `explain` 시나리오 C06.
7. 이 명령의 결과가 질문이 이미 충족 이면 즉시 정지 (S15) — `explain` 시나리오 C07.
8. 이 명령을 무제한 덤프 로 쓰면 컨텍스트 고갈 — `explain` 시나리오 C08.
9. 쪽수가 31~100쪽 일 때 이 명령의 예산은 메타만 — `explain` 시나리오 C09.
10. 이 명령 이후 프롬프트에 넣을 수 있는 것은 주소와 짧은 발췌 뿐이고 전문·전 셀 텍스트 은 버린다 — `explain` 시나리오 C10.
11. 사용자가 '사실 위치' 라고 하면 이 명령을 쓴다 — `explain` 시나리오 C11.
12. 이 명령의 결과가 질문이 이미 충족 이면 즉시 정지 (S15) — `explain` 시나리오 C12.

## 정지

이 단에서 질문이 답이면 [07_when_to_stop.md](07_when_to_stop.md) 로 간다.
다음 단은 필요 신호(표, 누름틀, 특정 어휘, 수치)가 있을 때만 연다.
