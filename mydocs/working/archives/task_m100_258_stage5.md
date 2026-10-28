# Task M100-258 Stage 5 — 저장 라운드트립 검증과 최종 보고

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `644a5ed4` (`task 258: 누름틀 삽입 대화상자 구현`)

## 1. 작업 목적

Stage 2~4에서 구현한 양식 모드, 누름틀 이동, 누름틀 삽입 기능을 저장/재파싱 관점에서
확인하고 최종 보고서를 작성한다.

## 2. 확인 범위

- 새로 삽입한 ClickHere 필드의 HWP/HWPX 저장 후 재파싱
- `editable` 속성, ClickHere command 안내문/메모/이름, CTRL_DATA 이름 보존
- 기존 샘플의 ClickHere editable 속성 회귀
- Stage 2~4 focused 테스트와 스튜디오 빌드 결과 정리

## 3. 진행 기록

- Stage 5 문서 작성 후 저장 라운드트립 검증 착수.
- 새로 삽입한 ClickHere 필드를 HWP/HWPX로 저장 후 재파싱하는 회귀 테스트를 추가했다.
- 1차 검증에서 HWP는 통과했으나 HWPX 재파싱 시 ClickHere command 안내문/메모가
  빠지는 문제를 확인했다.
- HWPX 직렬화에서 `raw_parameters_xml`이 없는 새 필드라도 `Field.command`가 있으면
  `<hp:parameters><hp:stringParam name="Command">…</hp:stringParam></hp:parameters>`를
  생성하도록 보정했다.
- 보정 후 HWP/HWPX 양쪽에서 안내문, 메모, 필드 이름, `editable` 속성 보존을 확인했다.

## 4. 검증 결과

- `cargo fmt --check` 통과
- `cargo test --test issue_258_clickhere_form_mode` 통과
- `cargo test --test issue_838_field_set_value` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `npm run build` 통과
