# Task M100 #1443 Stage 5 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-19
- 선행 커밋: `8001b6bf task 1443: 셀 크기 균등화 선택 범위 적용`

## 1. Stage 5 목표

사용자 피드백의 `Alt+C로 글꼴&문단모양 복사하기/붙여넣기 [모양복사]` 항목을 구현한다.

현재 `edit:format-copy`는 메뉴 항목만 있고 `canExecute: () => false`인 미구현 상태다. 이번 단계에서는 rhwp-studio의
기존 글자/문단 모양 적용 API가 지원하는 범위 안에서 다음 동작을 1차 목표로 삼는다.

- 현재 커서 위치 또는 선택 시작 위치의 글자/문단 모양을 복사한다.
- 복사된 모양이 있으면 `Alt+C` 또는 메뉴의 `모양 복사` 실행 시 현재 선택 영역/현재 위치에 붙여넣는다.
- 표 셀 안에서도 같은 방식으로 동작한다.

## 2. 현재 판정

- `edit:format-copy` 명령은 비활성 상태.
- 표시 단축키는 `Ctrl+Alt+C`로 되어 있어 한컴 피드백의 `Alt+C`와 다르다.
- 글자 모양/문단 모양 적용 명령은 `format:*` 계열과 `InputHandler`에 흩어져 있어 재사용 가능한 API를 먼저 확인해야 한다.

## 3. 구현 방향

- 전역/서비스 레벨 저장소를 새로 크게 만들기보다 `InputHandler` 안에 모양복사 상태를 둔다.
- 첫 실행은 복사, 복사 상태가 있는 다음 실행은 붙여넣기로 처리한다.
- 최소 1차 지원 범위는 현재 엔진에서 읽고 쓸 수 있는 글자 모양/문단 모양 속성으로 제한한다.
- 단축키는 `Alt+C`를 우선 연결하고, 기존 표시 문자열도 한컴 피드백에 맞춘다.

공식 도움말 기준:

- 한컴 도움말 `모양 복사 <Alt+C>`:
  https://help.hancom.com/hoffice/multi/ko_kr/hwp/format/quick_format/quick_format.htm
- 보통 상태에서는 커서 위치의 모양을 임시 저장소에 기억하고, 블록 상태에서는 기억된 모양을 블록에 덮어쓴다.
- 표 안에서는 글자/문단 모양 외에 셀 속성, 셀 테두리, 셀 배경까지 복사할 수 있다.

이번 Stage 5는 1차 구현으로 글자 모양과 문단 모양을 지원한다. 셀 속성/셀 테두리/셀 배경 복사는 별도 확장 범위로 남긴다.

## 4. 구현 내용

- `edit:format-copy`를 활성화하고 표시 단축키를 `Alt+C`로 변경했다.
- `shortcut-map.ts`에 `Alt+C`, 한글 입력 상태의 `Alt+ㅊ` 매핑을 추가했다.
- `InputHandler`에 모양복사 상태를 추가했다.
  - 선택이 없으면 현재 커서의 글자/문단 모양을 기억한다.
  - 선택이 있고 기억된 모양이 있으면 선택 범위에 글자/문단 모양을 적용한다.
  - 표 셀 내부 선택은 기존 `ApplyCharFormatCommand`, `ApplyParaFormatCommand` 경로를 재사용한다.
- 새로 생성된 표 셀 문단처럼 `char_shapes`가 비어 있어도 범위 글자 모양 적용이 가능하도록 `Paragraph::apply_char_shape_range`에서 기본 ref를 보강했다.
- 해당 Rust 보정에 단위 테스트를 추가했다.

## 5. 회귀 방지 기준

- 일반 `Ctrl+C` 복사와 충돌하지 않는다.
- macOS Option+C 입력이 텍스트 입력 경로를 망가뜨리지 않는다.
- 기존 `format:*` 명령과 undo/dirty 흐름을 깨지 않는다.
- 문서가 없거나 커서가 유효하지 않은 상태에서는 조용히 비활성/무시된다.

## 6. 검증 결과

- `wasm-pack build --target web --out-dir pkg` 통과.
- `node /private/tmp/rhwp_1443_format_copy_check.mjs` 통과.
  - 본문: 커서 위치 글자/문단 모양을 `Alt+C`로 기억한 뒤 대상 텍스트 선택 영역에 적용됨.
  - 표 셀: 셀 내부 source 글자/문단 모양을 `Alt+C`로 기억한 뒤 다른 셀 텍스트 선택 영역에 적용됨.
- `cargo test --lib test_apply_char_shape_range_seeds_empty_shape_refs` 통과.
- `cd rhwp-studio && npm test` 통과.
- `node /private/tmp/rhwp_1443_cell_drag_check.mjs` 통과.
- `node /private/tmp/rhwp_1443_protected_cell_drag_check.mjs` 통과.
- `cd rhwp-studio && npm run build` 통과.
- `cargo fmt --check` 통과.
- `git diff --check` 통과.

참고:

- 인앱 Browser는 이번 세션에서 `Browser is not available: iab`로 연결되지 않아, 지침에 따라 headless Chrome/Puppeteer 검증으로 대체했다.
- `cargo test --lib test_apply_char_shape_range_seeds_empty_shape_refs -- --exact`는 Rust 테스트 필터가 전체 모듈 경로 exact match를 요구해 0개 실행으로 끝났고, `--exact` 없이 재실행해 1개 테스트 통과를 확인했다.
- 중간에 실행한 `cargo test --lib apply_char_shape_range`는 rustc가 CPU 0% 상태로 장시간 멈춰, 해당 테스트 프로세스만 종료했다. 이후 정확한 단일 테스트는 정상 통과했다.
