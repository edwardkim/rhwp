# task_m100_3281 처리결과 보고서 — `fields` 누름틀 조사 (읽기 전용)

- **이슈**: [#3281](https://github.com/edwardkim/rhwp/issues/3281)
- **브랜치**: `pr/task-fields-json` (**upstream/devel 직분기 — 열린 PR 과 공유 커밋 없음**)
- **범위**: `src/main.rs`(명령 1개·디스패치 1행·help), `queries/mod.rs`(가시성 1행),
  `tests/fields_json_contract.rs`(신규), `mydocs/manual/cli_commands.md`
- **분류**: 기능 추가 (읽기 전용 질의)

## 1. 문제

rhwp 는 이미 필드에 값을 **쓸 수** 있다(`set_field_value_by_name`). 그런데 **읽을 CLI 입구가
없었다** — 조회 API(`collect_all_fields`, `get_field_list_json`)가 WASM/스튜디오 경로에만
노출되어 브라우저 밖 에이전트의 접근 경로가 0이었다. 그래서 에이전트는 서식을 받아도 "이
템플릿이 무슨 값을 요구하는지" 알 수 없고, 필드 이름을 추측해 쓰기 API 를 때려보는 수밖에
없었다.

## 2. 분석 — 설계 결정

- **기존 질의를 그대로 노출한다.** 새 순회 로직을 쓰지 않고 `collect_all_fields()` 를 부른다.
  `HwpDocument` → `DocumentCore` Deref 로 CLI 에서 바로 호출된다.
- **지시문(memo)을 포함한다.** 기존 `get_field_list_json` 스키마(fieldId/fieldType/name/
  guide/command/value/location/editableInForm)를 따르되, `Field::memo_text()` 의 HelpState
  지시문을 더했다 — "이 칸에 무엇을 어떻게 쓰라"는 안내는 에이전트에게 가장 값진 신호다.
- **중첩 위치를 구조화한다.** `NestedEntry` 를 `{kind:"tableCell"|"textBox", control, cell,
  paragraph}` 로 방출해, 표 셀·글상자 안의 필드도 후속 편집이 좌표를 찾을 수 있게 했다.
- **읽기 전용이다.** 편집(쓰기)은 범위 밖이므로 편집 관문(Stage 3)을 건드리지 않는다.
- **정정**: 이슈에는 "라이브러리 변경 0"으로 적었으나, 실제로는 `queries/mod.rs` 의
  `field_query` 가 `pub(crate)` 라 `NestedEntry` 를 CLI 에서 매칭할 수 없었다.
  `structure`·`rendering` 과 같은 `pub mod` 로 올렸다(**1행**). 읽기 전용 질의 모듈이므로
  가시성 선례와 일치한다.

## 3. 변경

- `show_fields()` — `--json` 봉투 / 기본은 사람용 요약
- 디스패치 1행, help 등재, `queries/mod.rs` 가시성 1행
- `cli_commands.md` 신설 항목 (범위 한계 명시)

## 4. 검증

- **계약 테스트 8종 red→green**: 봉투 스키마 / 이름 노출(폼 채우기 가능성) / **지시문 추출** /
  중첩 위치 배열 / **필드 없는 문서는 오류가 아니라 빈 목록** / 기본 출력 비-JSON 가드 /
  종료 코드(없는 파일 1·인자 없음 2)
- `cargo clippy --release --bin rhwp -- -D warnings` 0, `rustfmt` clean, 문서 검사 2종 clean
- 실측: `samples/field-01-memo.hwp` → 필드 11개, `회사명`(guide="여기에 입력",
  memo="회사명은 회사이름입니다. 반드시 거래업체 등록된 정식 명칭을…") 정확 추출

## 5. 남긴 것

- **머리말/꼬리말·각주/미주 안의 필드**는 `collect_fields_from_paragraph` 재귀가 표 셀·글상자
  두 갈래뿐이라 잡히지 않는다. 스튜디오는 머리말 필드 편집을 지원하므로 실재하는 사각지대다.
  본 PR 은 현행 범위를 그대로 노출하고 문서에 명시했다 — 재귀 확장은 편집 API 좌표계와 함께
  봐야 하므로 별도 이슈가 맞다.
- MCP 도구 등록(`hwp_fields`)은 `capabilities --mcp`(#3263, PR #3264)가 머지된 뒤 후속.
