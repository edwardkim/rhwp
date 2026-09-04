# 완료 보고서 — Task M100-2948

- 이슈: #2948
- 제목: HWPX `<hp:tc dirty>` 셀 속성이 항상 0으로 하드코딩되어 라운드트립 소실
- 작성일: 2026-07-22
- 브랜치: `task/m100-2948-tc-dirty`

## 0. 작업 재개 메모 (디스크 공간 이슈)

이번 세션 시작 시 디스크 공간이 임계 수준으로 부족한 상태였다. 사용자가 여유 공간을
54GB+ 확보하고 `target/*/incremental` 및 오래된 `deps` 산출물을 정리한 뒤 작업을
재개했다. 정리 후 `cargo check --lib` 은 링커 오류 없이 정상 동작했다 — 이번 작업에서
`LNK1123`/`CVT1107`/`dbghelp.lib` 류의 Windows SDK 링커 손상은 재현되지 않았다.

## 1. 완료 내용

HWPX 표 셀(`<hp:tc>`)의 `dirty` 속성(편집기 캐시 무효화 표시 플래그)이 파서에서
아예 읽히지 않고, 직렬화기에서는 값과 무관하게 항상 `dirty="0"` 으로 하드코딩되어
방출되던 문제를 수정했다. "parsed but hardcoded" 계열(lock, reverse, dropCapStyle,
groupLevel, numberingType, fieldid 등)에 속하는 반복 패턴이다.

## 2. 주요 변경

- `src/model/table.rs`
  - `Cell` 구조체에 `pub dirty_flag: bool` 필드 추가.
  - 템플릿 기반 신규 셀 생성 경로(`Cell::from_template` 계열)는 `dirty_flag: false` 로
    초기화 — dirty는 편집기 캐시 상태이므로 템플릿에서 상속하지 않음.
- `src/parser/hwpx/section.rs`
  - `parse_table_cell` 에 `b"dirty" => cell.dirty_flag = parse_bool(&attr),` 추가.
- `src/serializer/hwpx/table.rs`
  - `write_cell` 에서 하드코딩된 `("dirty", "0")` 을 `("dirty", dirty)`
    (`dirty = bool01(cell.dirty_flag)`) 로 교체.
- `src/diagnostics/ir_field_sweep.rs`
  - `sweep_cell` 의 `Cell` 전수 구조 분해(`..` 없이 모든 필드 나열)에 새 필드
    `dirty_flag` 를 추가하고 `f!(dirty_flag);` 비교 호출을 추가. (devel 최신본과 병합
    과정에서 이 파일이 새 필드를 인식하지 못해 컴파일이 깨졌던 부분을 별도 커밋으로 수정.)

## 3. red→green 테스트

`src/serializer/hwpx/table.rs::tests`

- `tc_dirty_flag_preserved_when_set` — `dirty_flag=true` 인 셀이 `dirty="1"` 로
  직렬화되는지 확인 (수정 전에는 항상 `dirty="0"` 하드코딩이라 실패).
- `tc_dirty_flag_zero_when_unset` — 기본값(false)에서는 기존 동작대로 `dirty="0"`
  유지 확인.

## 4. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib tc_dirty_flag` — 2 passed
- `rustfmt --edition 2021` (변경 4개 파일: `src/model/table.rs`,
  `src/parser/hwpx/section.rs`, `src/serializer/hwpx/table.rs`,
  `src/diagnostics/ir_field_sweep.rs`) — 포맷 위반 없음

## 5. 특이사항

- `origin/devel` 에 브랜치를 새로 딴 뒤 기존 커밋을 cherry-pick 하는 과정에서
  `src/serializer/hwpx/table.rs` 테스트 삽입 지점에서 다른 병렬 작업(`allowOverlap`
  보존 테스트, PR 이력상 이미 devel에 병합됨)과 충돌이 발생했다. 두 테스트 블록을
  모두 보존하는 방식으로 수동 해결했다.
- 동일하게 devel 최신본에서 `src/diagnostics/ir_field_sweep.rs::sweep_cell` 이
  `Cell` 을 `..` 없이 전수 구조 분해하는 방어적 패턴을 쓰고 있어, 새 필드 추가만으로
  컴파일이 깨졌다. 해당 파일에 `dirty_flag` 를 반영하는 후속 커밋으로 수정했다.

## 6. 결론

Task M100-2948 구현과 검증을 완료했다. PR 생성 대기 상태다.
