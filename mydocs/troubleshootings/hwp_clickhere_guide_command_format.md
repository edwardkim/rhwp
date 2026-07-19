---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 누름틀(ClickHere) 안내문이 한컴에서 바인딩 안 됨 — command 포맷 어긋남 (Task #1434)

## 증상

rhwp-studio에서 누름틀 + 안내문 지정 후 HWP 저장 → rhwp-studio는 정상 표시되나 **한컴
2020/2022 편집기에서 안내문이 바인딩되지 않음**(빈 누름틀). 자기 정합 OK = 자기 검증 ≠
한컴 호환.

## 근본 원인 — command 문자열이 한컴 포맷과 3가지로 어긋남

누름틀 안내문은 `Field.command` 의 `Direction:wstring:{len}:{text}` 에 임베드된다.
한컴 정답지(`samples/field-01.hwp`, `form-01.hwp`) command:

```
Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:␣␣
Clickhere:set:47:Direction:wstring:5:제목 입력 HelpState:wstring:0:␣␣
```

`Field::build_clickhere_command` (`src/model/control.rs`) 의 어긋남:

| # | 항목 | 한컴 정답 | rhwp(결함) |
|---|------|----------|-----------|
| ① | Name 키 | command 에 **없음** (이름은 CTRL_DATA 0x57 전담) | name 있으면 `Name:wstring:{n}:{name}` **추가** |
| ② | trailing 공백 | HelpState 값 뒤 **2개** | **1개** |
| ③ | set 길이 | inner 글자수 **− 1** | inner 글자수 전체 |

set 길이는 한컴이 command 를 파싱할 때 범위를 자르는 기준이다. 길이가 1 크거나 Name 키가
끼면 한컴이 **Direction 범위를 잘못 잘라 안내문 바인딩 실패**. 이름은 이미 CTRL_DATA
레코드(`src/serializer/control.rs:140~`)에 저장되므로 command 의 Name 은 중복·유해.

(set 길이 규칙은 한컴 2케이스 교차검증: inner−1 일관. inner = `Direction:wstring:{gl}:
{guide} HelpState:wstring:{ml}:{memo}  ` — HelpState 뒤 공백 2개.)

## 해소

`build_clickhere_command` 정정 (시그니처 `(guide, memo, name)` → `(guide, memo)`):
- Name 키 제거 (CTRL_DATA 전담).
- HelpState 뒤 trailing 공백 2개.
- set 길이 = inner − 1.

호출처 2곳 정합: `field_query.rs`(삽입 — name 은 ctrl_data_name 으로 별도 저장),
`wasm_api.rs`(수정 — 이미 name="" 전달).

검증: 한컴 2케이스 바이트 동형 단위 테스트 + 저장→재파싱 guide/memo/name 보존 round-trip
+ **한컴 2020/2022 편집기 바인딩 판정 통과**(작업지시자 Windows 환경).

## 재발 방지 체크리스트

- [ ] 누름틀 command 생성 시 Name 키를 넣지 말 것 (이름은 CTRL_DATA 0x57 전담).
- [ ] HelpState 값 뒤 공백 2개, set 길이 = inner − 1 (한컴 정답지 규칙).
- [ ] command 포맷 변경 시 한컴 정답지 샘플(field-01/form-01) command 와 바이트 대조.
- [ ] **반드시 한컴 편집기 바인딩 판정** (rhwp 자기 정합으로 충분하다고 판단 금지).

## 관련

- Task #1434, 한컴 정답지 `samples/field-01.hwp`·`form-01.hwp`
- `src/model/control.rs` (build_clickhere_command/guide_text/memo_text)
- `src/serializer/control.rs` (CTRL_DATA 이름 레코드)
- 테스트 `tests/issue_1434_clickhere_guide_hancom_command.rs`
