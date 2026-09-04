# Task M100 #1434 최종 보고서 — 누름틀 안내문(Direction) 한컴 바인딩 결함

- 이슈: #1434 "누름틀 안내문(Direction)이 한컴 에디터에서 바인딩되지 않음 — HWP 저장 호환 결함"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1434`
- 작성일: 2026-06-19

## 1. 개요

rhwp-studio에서 누름틀 + 안내문 지정 후 HWP 저장 시 한컴 편집기에서 안내문이 바인딩되지
않던 결함을 해소했다. 누름틀 command 문자열이 한컴 정답지 포맷과 3가지로 어긋난 것이 원인.

## 2. 근본 원인

누름틀 안내문은 `Field.command` 의 `Direction:wstring:{len}:{text}` 에 임베드된다.
한컴 정답지(`field-01.hwp`/`form-01.hwp`) 대조로 `build_clickhere_command` 의 어긋남 확정:

| # | 항목 | 한컴 정답 | rhwp(결함) |
|---|------|----------|-----------|
| ① | Name 키 | command 에 없음(CTRL_DATA 0x57 전담) | name 있으면 `Name:wstring:..` 추가 |
| ② | trailing 공백 | HelpState 뒤 2개 | 1개 |
| ③ | set 길이 | inner − 1 | inner 전체 |

set 길이가 1 크거나 Name 키가 끼면 한컴이 Direction 범위를 잘못 잘라 안내문 바인딩 실패.
(set 규칙은 한컴 2케이스 교차검증으로 inner−1 일관 확인.)

## 3. 해소

`build_clickhere_command` (`src/model/control.rs`) 정정 — 시그니처 `(guide, memo, name)`
→ `(guide, memo)`:
- Name 키 제거 (이름은 CTRL_DATA 전담).
- HelpState 뒤 trailing 공백 2개.
- set 길이 = inner − 1.

호출처 2곳 정합: `field_query.rs:1247`(삽입 — name 은 `ctrl_data_name` 별도 저장),
`wasm_api.rs:4045`(수정 — 이미 name="" 전달이라 동작 불변).

## 4. 검증

- 단위 테스트 4건(`model::control`): 한컴 2케이스 바이트 동형, Name 부재, guide/memo 왕복,
  set=inner−1 규칙.
- round-trip 통합 테스트 2건(`tests/issue_1434_clickhere_guide_hancom_command.rs`):
  삽입 직후 command 한컴 포맷, 저장→재파싱 후 guide/memo/이름 보존 + Name 부재.
- `model::control` 7 / `field_query` 4 passed, 한컴 샘플 field-01 dump 불변(파싱 회귀 0).
- CI급 `cargo test --profile release-test --tests` 전체 그린, fmt clean, clippy 0.
- **한컴 2020/2022 편집기 바인딩 판정 통과** (작업지시자 Windows 환경 — 산출 샘플
  `output/poc/task1434/clickhere-guide-여기에입력.hwp` 직접 열어 안내문 정상 바인딩 확인).

## 5. 의의

자기 정합(rhwp-studio 정상)으로 가려져 있던 한컴 호환 결함을, 한컴 정답지 command 바이트
대조로 근본 원인을 특정하고 정정. 누름틀 command 포맷이 한컴과 바이트 동형이 됐다.

## 6. 산출물

- 수행계획서: `mydocs/plans/task_m100_1434.md`
- 구현계획서: `mydocs/plans/task_m100_1434_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1434_stage{1,2}.md`
- 최종 보고서: 본 문서
- 트러블슈팅: `mydocs/troubleshootings/hwp_clickhere_guide_command_format.md`
- 테스트: `tests/issue_1434_clickhere_guide_hancom_command.rs`
- 한컴 판정 샘플: `output/poc/task1434/`
