# Task M100 #1443 Stage 6 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `6cb59962 task 1443: Alt+C 모양복사 1차 구현`

## 1. Stage 6 목표

Stage 5에서 남긴 표 안 모양복사 확장 범위를 처리한다.

한컴 도움말 기준으로 표 안에서 [모양 복사]는 글자/문단 모양뿐 아니라 셀 속성, 셀 테두리, 셀 배경도 복사할 수 있다. Stage 5는 글자/문단 모양까지만 1차 구현했으므로, 이번 단계에서는 rhwp-studio가 현재 노출하는 셀 속성 API 범위 안에서 셀 모양 복사를 보강한다.

## 2. 현재 판정

- `Alt+C` 글자/문단 모양 복사: Stage 5에서 구현 완료.
- 표 셀 안 글자/문단 모양 복사: Stage 5에서 검증 완료.
- 셀 속성/셀 테두리/셀 배경 복사: 아직 미구현.

## 3. 구현 방향

- 커서가 표 셀 안에 있고 선택이 없을 때 `Alt+C`를 누르면 글자/문단 모양과 함께 현재 셀의 셀 모양도 기억한다.
- 표 셀 선택 범위가 있는 상태에서 `Alt+C`를 누르면 기억된 셀 모양을 선택 셀 범위에 적용한다.
- 일반 텍스트 선택 상태에서는 Stage 5와 같이 글자/문단 모양만 적용한다.
- 셀 속성 복사는 현재 `getCellProperties`/`setCellProperties`가 지원하는 속성으로 제한한다.
- border/fill은 같은 문서 안에서는 `borderFillId`를 복사해 적용한다. 이렇게 해야 셀 고유 border/fill 참조가 보존되고, border 객체 안의 `width` 같은 중첩 필드가 셀 폭으로 오인되는 문제를 피할 수 있다.

## 4. 구현 내용

- `FormatCopyState`에 `cellProps`를 추가했다.
- 셀 안에서 선택 없이 `Alt+C`를 실행할 때 현재 셀의 다음 속성을 함께 기억한다.
  - 안 여백
  - 세로 정렬
  - 텍스트 방향
  - 제목 셀 여부
  - 셀 보호
  - 필드 이름
  - 양식 모드에서 편집 가능
  - `borderFillId` 기반 셀 테두리/배경
- 셀 선택 모드에서 `Alt+C`를 실행하면 셀 선택을 해제하지 않고 `edit:format-copy`를 먼저 dispatch하도록 키보드 분기를 보강했다.
- 셀 블록 선택 상태에서 기억된 `cellProps`를 선택 범위의 각 셀에 snapshot 작업으로 적용한다.
- `set_cell_properties_native`가 중첩 border 객체의 `"width"`를 최상위 셀 `width`로 오인하지 않도록 최상위 셀 속성 파싱을 `serde_json::Value` 기반으로 보정했다.
- `set_cell_properties_native`가 최상위 `borderFillId`를 직접 적용할 수 있게 했다.
- border/fill JSON 설정 시 셀 폭/높이가 바뀌지 않는 Rust 회귀 테스트를 추가했다.

## 5. 회귀 방지 기준

- 일반 본문 `Alt+C` 모양복사 동작을 유지한다.
- 셀 내부 텍스트 선택에 `Alt+C`를 누른 경우 셀 전체 속성이 아니라 텍스트 글자/문단 모양만 적용한다.
- 셀 블록 선택 상태에서만 여러 셀의 셀 모양을 덮어쓴다.
- 셀 보호 입력 차단, 보호 셀 드래그 선택, 셀 크기 균등화 동작을 깨지 않는다.

## 6. 검증 결과

- `wasm-pack build --target web --out-dir pkg` 통과.
- `cd rhwp-studio && npx tsc --noEmit` 통과.
- `node /private/tmp/rhwp_1443_cell_shape_format_copy_check.mjs` 통과.
  - source 셀의 여백/세로 정렬/텍스트 방향/제목 셀/셀 보호/필드 이름/양식 편집 가능/테두리/배경을 `Alt+C`로 복사.
  - 2x2 셀 블록 선택 상태에서 `Alt+C`를 다시 실행해 선택된 4개 셀에 동일 속성 적용 확인.
  - 선택 범위 밖 셀은 고유 여백/속성 유지 확인.
- `node /private/tmp/rhwp_1443_format_copy_check.mjs` 통과.
  - Stage 5 본문/셀 글자·문단 모양복사 회귀 없음.
- `cd rhwp-studio && npm test` 통과.
- `node /private/tmp/rhwp_1443_cell_drag_check.mjs` 통과.
- `node /private/tmp/rhwp_1443_protected_cell_drag_check.mjs` 통과.
- `cargo test --test issue_493_cell_attrs set_cell_border_properties_do_not_overwrite_cell_size` 통과.
- `cargo test --test issue_493_cell_attrs` 통과.
- `cd rhwp-studio && npm run build` 통과.
- `cargo fmt --check` 통과.
- `git diff --check` 통과.
