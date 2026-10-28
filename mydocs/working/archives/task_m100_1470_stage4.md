# Task M100 #1470 stage4 작업 기록

- 이슈: #1470 `스타일 적용/편집 불일치: 왼쪽 여백 배율, 줄간격 미반영, 표 캡션/생성 위치 문제`
- 기준 커밋: `7fb31cd6 task 1470: 스타일 직접 서식 보존`
- 작업 브랜치: `task_m100_1470`
- 작성일: 2026-06-22
- 상태: 구현/검증 완료. 표 캡션 번호 재정렬과 한컴식 `표 N` 접두어 보정 완료.

## 1. Stage 4 배경

Stage 1~3에서 #1470의 주요 보고 현상은 focused 테스트와 IAB 검증으로 대부분 닫았다.

- 스타일 왼쪽 여백 15pt 왕복 및 줄간격 reflow
- 실제 스타일 적용/편집 UI 경로
- 새 스타일의 현재 문단 기반 생성
- 표 생성 상세 옵션 전달
- TAC 표 중복 렌더 및 다음 페이지 이동 방지
- 표 캡션 AutoNumber 유지 및 삭제

다만 이슈 본문과 검증 아이디어에는 "표 캡션 생성/삭제/번호 재정렬" 및 "캡션 수정 흐름"까지 포함되어 있다.
현재 구현은 캡션 생성 시에만 `assign_auto_numbers`를 호출한다. 캡션 삭제나 여러 표 캡션이 섞인 상태에서 번호 재정렬이 항상 보장되는지는 아직 focused 테스트로 고정하지 않았다.

Stage 4는 이 잔여 항목을 별도 스테이지로 해결해 #1470 종료 판단의 빈틈을 없앤다.

추가 사용자 피드백:

- 표 캡션은 숫자만 `1`, `2`, `3`으로 표시되면 안 되고 한컴처럼 `표 1`, `표 2`, `표 3` 형태여야 한다.
- 그림 캡션은 이미 `그림 1`, `그림 2` 형태로 표시되므로, 표 캡션 생성 구조도 그림 캡션과 동일한 AutoNumber 앵커 패턴을 따라야 한다.

## 2. 잔여 항목

### 2.1 여러 표 캡션 삭제 후 번호 재정렬

현재 코드상 표 캡션 생성 시에는 `assign_auto_numbers`를 호출하지만, `hasCaption=false`로 캡션을 삭제하는 경로는 AutoNumber 재할당을 호출하지 않는다.

위험:

- 표 1, 표 2, 표 3 캡션을 만든 뒤 중간 캡션을 삭제하면 남은 표 캡션 번호가 한컴식 순번으로 재정렬되지 않을 수 있다.
- 내부 `AutoNumber.assigned_number`가 stale 상태로 남아 렌더/직렬화 시 이전 번호를 표시할 수 있다.

### 2.2 캡션 속성 수정 흐름 회귀 테스트 부족

현재 `captionDirection`, `captionWidth`, `captionSpacing`, `captionVertAlign` 수정 경로는 구현되어 있지만,
캡션 속성 수정 후 다음 조건을 focused 테스트로 고정하지 않았다.

- AutoNumber 컨트롤이 literal 텍스트로 바뀌지 않고 유지되는지
- 속성 수정 후 캡션 dirty/recompose가 일어나고 표 렌더가 중복되지 않는지
- 수정 후에도 페이지 수와 표 위치가 안정적인지

### 2.3 스타일 블록 적용의 테스트 고정 부족

Stage 1에서 Studio `InputHandler.applyStyle()`가 `getParaFormatTargetsAtCursor()` 결과를 순회하도록 바뀌어 블록 선택 다중 문단 경로는 코드상 반영되어 있다.
하지만 #1470 focused Rust 테스트에는 이 항목이 별도 테스트명으로 고정되어 있지 않다.

Stage 4에서는 소스 변경이 필요 없는 범위라면 IAB 검증 기록으로 충분한지 판단하고, 필요하면 최소 테스트를 추가한다.

## 3. 작업 범위

1. 표 캡션 삭제/수정 시 AutoNumber 재할당 조건을 정리한다.
2. 캡션 생성/삭제 후 남은 표 캡션 번호가 1부터 다시 순서대로 배정되는지 테스트한다.
3. 캡션 속성 수정 후 AutoNumber 컨트롤 유지와 속성 반영을 테스트한다.
4. TAC 표 + 캡션 on/off 후 렌더 노드가 한 번만 나오는 기존 보정을 유지한다.
5. 필요 시 `set_table_properties_native`에서 캡션 삭제/수정 후 `assign_auto_numbers` 호출 범위를 보정한다.
6. 스타일 블록 적용은 새 결함이 재현될 때만 소스 수정하고, 그렇지 않으면 IAB 또는 문서 검증으로 남긴다.

## 4. 제외 범위

- 그림/수식/글상자 캡션 전체 재설계
- 캡션 내용 편집 UX 전체 구현
- 표 캡션 스타일 UI 전체 재현
- 모든 AutoNumber/NewNumber 정책 재설계
- Stage 2에서 별도 후보로 남긴 TAC 표 뒤 키보드 입력 순서 E2E 수정
- 오래된 E2E 스크립트의 `getParaText` API 갱신

## 5. 구현 방향

우선 후보:

- `src/document_core/commands/table_ops.rs`의 표 캡션 처리에서 `caption_created || caption_changed`인 경우 AutoNumber를 재할당한다.
- 단, 단순 캡션 폭/간격 수정도 전체 번호를 다시 배정해도 의미상 문제는 없으므로, 조건을 단순하게 두는 방향을 검토한다.
- 캡션 삭제는 `caption_changed=true`가 되므로 같은 후처리로 처리한다.
- 재할당 후 캡션 문단 reflow는 기존 캡션 생성 경로와 같은 폭 기준을 사용한다.

대안:

- 삭제/생성처럼 번호 집합이 바뀌는 경우에만 재할당하고, 방향/폭/간격 수정은 reflow만 수행한다.
- 이 방식은 불필요한 번호 재할당을 줄이지만, 분기와 테스트가 늘어난다.

## 6. 테스트 계획

Focused Rust 테스트:

- `issue_1470_table_caption_renumbers_after_delete`
  - 표 3개를 만들고 모두 캡션을 켠다.
  - AutoNumber assigned number가 1, 2, 3인지 확인한다.
  - 첫 번째 또는 중간 표 캡션을 삭제한다.
  - 남은 표 캡션 번호가 1, 2로 재배정되는지 확인한다.
  - 렌더링 결과가 `표 1`, `표 2`만 포함하고 stale `표 3`을 남기지 않는지 확인한다.

- `issue_1470_table_caption_edit_keeps_autonumber`
  - 표 캡션을 만든 뒤 방향, 폭, 간격, 세로 정렬을 수정한다.
  - AutoNumber 컨트롤이 유지되고 속성 값이 반영되는지 확인한다.

- 기존 회귀 유지:
  - `issue_1470_table_caption_keeps_autonumber_and_can_be_removed`
  - `issue_1470_create_table_ex_tac_caption_renders_once`
  - `cargo test --release issue_1470 --lib`

브라우저/IAB 검증:

- `http://localhost:7700/`에서 표 여러 개 + 캡션 on/off 후 pageCount=1, 렌더 표 노드 중복 없음, console error 없음 확인.
- 필요하면 블록 선택 스타일 적용을 IAB로 확인해 stage 문서에 결과를 기록한다.

빌드/포맷:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --release issue_1470 --lib`
- Rust/WASM 변경이 Studio 표시에도 영향을 주면 `wasm-pack build --target web --out-dir pkg`
- Studio UI 검증이 필요하면 `cd rhwp-studio && npm run build`

`cargo clippy --all-targets -- -D warnings`는 이전 작업지시자의 중지 지시에 따라 별도 지시가 있을 때만 실행한다.

## 7. 진행 결과

구현:

- `src/document_core/commands/table_ops.rs`
  - 캡션 생성/수정/삭제 후 `assign_auto_numbers`를 호출하도록 보정했다.
  - 캡션 방향이 왼쪽/오른쪽일 때는 `captionWidth`, 위/아래일 때는 `max_width` 기준으로 캡션 문단을 reflow한다.
  - 표 캡션 기본 문단을 `"표  "` + AutoNumber 앵커 구조로 생성하도록 바꿨다.
- `src/parser/mod.rs`
  - AutoNumber 재배정 시 `assigned_number`뿐 아니라 legacy/렌더 참조용 `number`도 같이 갱신한다.
- `src/wasm_api/tests.rs`
  - 표 캡션 삭제 후 번호 재정렬 테스트를 추가했다.
  - 캡션 속성 수정 후 AutoNumber 유지 테스트를 추가했다.
  - 표 캡션 문단이 `"표  "` 접두어와 그림 캡션과 같은 offset 구조를 유지하는지 확인한다.

검증:

- `cargo fmt && cargo test --release issue_1470_table_caption --lib`
  - 통과: 3 passed
- `cargo test --release issue_1470 --lib`
  - 통과: 10 passed
- `wasm-pack build --target web --out-dir pkg`
  - 통과

## 8. 승인 게이트

Stage 4 구현은 표 캡션 번호 재정렬/수정 흐름의 focused 보정과 검증으로 제한했다.
추가로 발견된 PR #1446 모양복사 회귀 후보는 Stage 4 커밋 뒤 별도 Stage 5 문서와 변경으로 분리한다.
