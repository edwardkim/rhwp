---
kind: report
status: done
task: m100-3001
issue: 3001
---

# task-m100-3001: 하이퍼링크 fieldBegin 직렬화 command 속성 오사용 수정 보고서

## 이슈

edwardkim/rhwp#3001 — `src/serializer/hwpx/field.rs::write_hyperlink_begin()` 이
하이퍼링크 URL을 `<hp:fieldBegin command="...">` 속성으로 방출하는데, HWPX
스키마는 `fieldBegin` 에 `command` 속성을 정의하지 않는다. 실제 파서
(`src/parser/hwpx/section.rs::parse_field_begin_attrs`)는 `type`/`name`/`id`/
`fieldid`/`editable` 만 인식하고 그 외 속성은 조용히 버리므로, 이 함수가 만든
XML을 다시 읽으면 URL이 완전히 소실된다. 필드 명령 문자열은
`<hp:parameters><hp:stringParam name="Command">` 자식 요소로만 전달되어야
한다(같은 파일 `parse_field_parameters()`가 명시적으로 구현).

## 근본 원인

`write_hyperlink_begin()` 이 파서가 읽지 않는 위치(존재하지 않는 속성)에 값을
써서, 직렬화 → 파싱 왕복에서 하이퍼링크 URL이 무조건 소실되는 계약 불일치.

## 수정

`src/serializer/hwpx/field.rs`

- `write_hyperlink_begin()` 을 `<hp:fieldBegin>` 자기닫힘 + `command` 속성
  대신, `<hp:fieldBegin>…<hp:parameters><hp:stringParam
  name="Command">{url}</hp:stringParam></hp:parameters></hp:fieldBegin>` 구조로
  방출하도록 변경. URL 텍스트는 `super::utils::text()` (자동 이스케이프)로
  방출해 `&`/`<`/`>` 를 포함한 URL도 안전.
- 기존 테스트 `hyperlink_begin_uses_url_command` (잘못된 계약을 검증하던 테스트)
  를 `hyperlink_begin_uses_string_param_command` 로 교체 — `command` 속성이
  없음과 `stringParam` 자식이 있음을 함께 검증.

diff 규모: `src/serializer/hwpx/field.rs` 순수 수정부(함수 본문 교체 + import
1줄) 약 15줄 + 테스트 교체(순증감 상쇄) — 요구된 1–15줄 범위 내.

## 검증 (Red → Green)

1. **Red**: 수정 전 코드에서 `write_hyperlink_begin()` 출력은
   `<hp:fieldBegin id="7" type="HYPERLINK" name="" editable="0"
   command="https://example.com"/>` 이었다. 이를 `parse_field_begin_attrs` +
   `parse_field_parameters` 경로로 재파싱하면 `command` 속성은 인식되지 않는
   속성이라 무시되고 `<hp:parameters>` 자식이 없으므로 `Field::command` 는
   빈 문자열이 된다(코드 리딩으로 확인 — 두 함수의 속성/자식 매칭 분기에
   `command` 속성 처리가 전혀 없음).
2. **Green**: 수정 후 `cargo test --lib
   serializer::hwpx::field::tests::hyperlink_begin_uses_string_param_command`
   실행 결과:
   ```
   test serializer::hwpx::field::tests::hyperlink_begin_uses_string_param_command ... ok
   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2521 filtered out
   ```
   출력 XML에 `command=` 속성이 없고
   `<hp:stringParam name="Command">https://example.com</hp:stringParam>` 이
   포함됨을 확인.
3. `cargo check --lib` 통과(경고 없음).

## 참고

- `write_hyperlink_begin` 은 `#[allow(dead_code)]` 로 표시된 스캐폴드 함수로
  현재 section.rs 직렬화 dispatcher 에서 호출되지 않는다. 그러나 향후 신규
  하이퍼링크 필드 생성 경로가 이 함수를 재사용할 가능성이 높고, 기존
  유닛 테스트가 잘못된 계약(`command` 속성)을 "정답"으로 고정하고 있어 지금
  바로잡았다.
- 로컬 `cargo test` 전체 실행 중 `dbghelp.lib` 손상으로 인한 링커 오류
  (`LNK1123`)가 무관한 빌드 스크립트(`serde_core`, `zerocopy`, `rustversion`
  등)에서 발생해 전체 스위트 실행이 막혔다(환경 문제, 이번 변경과 무관).
  대상 테스트는 `cargo test --lib serializer::hwpx::field::tests::…` 단독
  실행으로 통과를 확인했고, `cargo check --lib` 는 정상 완료됐다.
