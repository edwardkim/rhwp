# 완료 보고서 — Task M100-2857

- 이슈: #2857
- 제목: HWPX tabItem leader 속성 이중선/삼중선(SLIM_THICK/THICK_SLIM/SLIM_THICK_SLIM)
  파싱·직렬화가 OWPML 스펙 리터럴과 불일치
- 작성일: 2026-07-22
- 브랜치: `task/m100-2857-tabitem-leader-doubleslim`

## 1. 배경

`<hh:tabItem leader="...">`의 `fill_type` 8~11(이중선·삼중선 4종)은 과거 커밋
`5d0ac30a`에서 "9/10/11이 저장 시 NONE으로 유실"되는 문제를 고치면서 파서·직렬화를
내부적으로 짝지어 왕복되게는 했지만, 사용한 문자열(`THIN_THICK`/`THICK_THIN`/`TRIM`)이
`mydocs/manual/OWPML SCHEMA/Core XML schema.xml` 335~349행 `LineType3`의 실제 스펙
리터럴(`SLIM_THICK`/`THICK_SLIM`/`SLIM_THICK_SLIM`)과 달랐다. 그 결과:

- rhwp가 방출한 HWPX를 한/글 등 스펙 준수 구현체가 열면 정의되지 않은 leader
  값으로 처리될 위험.
- 한/글이 저장한 정상 HWPX 문서(스펙 리터럴 사용)를 rhwp가 열면 `leader`가
  매칭되지 않아 `NONE`(0)으로 조용히 유실.

`fill_type=8`(DOUBLE_SLIM)은 이미 스펙 리터럴과 일치했으므로 이번 수정 범위에서
제외했다.

## 2. 주요 변경

- `src/parser/hwpx/header.rs` (`parse_tab_item`)
  - `leader` 매칭에 스펙 리터럴 `"SLIM_THICK"`(9), `"THICK_SLIM"`(10),
    `"SLIM_THICK_SLIM"`(11)을 추가. 기존 비표준 이름(`THIN_THICK` 등)도
    하위호환을 위해 계속 허용.
- `src/serializer/hwpx/header.rs` (`tab_leader_str`)
  - 9/10/11 방출 문자열을 스펙 리터럴로 교체
    (`SLIM_THICK`/`THICK_SLIM`/`SLIM_THICK_SLIM`).

## 3. 테스트 (red → green)

`src/parser/hwpx/header.rs`에 추가:
`test_parse_tab_item_leader_accepts_owpml_spec_literal_slim_thick`

- 내용: `<hh:tabItem pos="1000" type="LEFT" leader="SLIM_THICK"/>`를 포함한
  `<hh:head>`를 파싱해 `doc_info.tab_defs[0].tabs[0].fill_type == 9` 확인.
- 수정 전: `"SLIM_THICK"`이 `match`에서 매칭되지 않아 `_ => 0`으로 떨어져
  `fill_type == 0` → 테스트 실패(red).
- 수정 후: `"SLIM_THICK" => 9` 매칭 → 테스트 통과(green).

## 4. 검증 결과

통과:

- `cargo build --lib`
- `cargo test --lib test_parse_tab_item_leader_accepts_owpml_spec_literal_slim_thick`
- `cargo clippy --all-targets --profile release-test -- -D warnings`
- `rustfmt --edition 2021 src/parser/hwpx/header.rs src/serializer/hwpx/header.rs`

## 5. 남은 항목

- `fill_type` 12(WAVE)/13(DOUBLEWAVE)는 IR(`TabItem::fill_type: u8`)이나 현재
  파서·직렬화에서 아예 다루지 않는다. 별도 조사·이슈 대상으로 남겨둔다(이번
  범위 밖).
