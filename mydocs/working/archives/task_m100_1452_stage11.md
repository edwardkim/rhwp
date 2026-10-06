# Task M100 #1452 Stage 11 작업 기록

## 배경

Stage 10에서 문자처럼 취급 그림 주변의 커서·문단부호 위치를 그림 bbox 기준으로 정리했다.
이후 작업지시자가 두 번째 그림 앞에서 Enter를 누를 때의 동작 차이를 확인했다.

한컴에서는 두 그림이 모두 문자처럼 취급되는 상태이므로, 두 번째 그림 앞 Enter는
문자 사이 문단 분할처럼 처리된다. 첫 번째 그림은 앞 문단에 남고 두 번째 그림은
다음 문단으로 이동해야 한다.

현재 rhwp에서는 같은 위치에서 Enter를 누르면 그림 사이에서 문단이 나뉘지 않고,
커서만 다음 줄로 이동하는 것처럼 보인다.

## 목표

- 본문 문단 분할 시 문자처럼 취급되는 inline 그림 control을 split offset 기준으로 앞/뒤 문단에 분배한다.
- `투명도0-50.hwp`처럼 텍스트 없이 inline 그림만 있는 문단에서도 offset 1 분할이 동작하게 한다.
- 두 번째 TAC 그림 끝에서 위쪽 방향키를 누르면 첫 번째 TAC 그림 끝으로 이동하게 한다.
- 기존 입력 메뉴 그림 삽입, 그림 뒤 Enter, 저장 커서 위치 보정 회귀를 막는다.

## 확인 예정

- `split_paragraph_native`가 텍스트만 분리하고 control 배열을 그대로 앞 문단에 남기는지 확인한다.
- `find_logical_control_positions` 또는 동등한 helper를 사용해 control의 논리 문자 위치를 계산한다.
- 구조 control과 inline control을 구분해 안전하게 분배한다.
- 세로 방향키 이동이 여러 줄 TAC control의 줄 끝 offset을 잃지 않는지 확인한다.

## 검증 예정

- `cargo fmt --check`
- `cargo test --test issue_1452_saved_caret -- --nocapture`
- 필요 시 관련 객체 입력 단위 테스트 추가
- WASM 갱신: `wasm-pack build --target web --out-dir pkg --no-pack`

## 구현 내용

- `Paragraph::split_at()`에서 TAC 그림/표/수식 등 문자처럼 움직이는 inline control을 logical offset 기준으로 앞/뒤 문단에 분배하도록 수정했다.
- control 분배 시 `ctrl_data_records`, `char_count`, `control_mask`, `has_para_text`를 함께 갱신하도록 했다.
- control-only 문단의 line segment 시작 위치를 raw UTF-16 control 위치에서 logical offset으로 복원했다.
- 세로 이동 후보 탐색에서 TextRun뿐 아니라 ImageNode의 좌/우 끝도 후보로 사용하도록 했다.

## 검증 결과

- `cargo fmt --check` 통과
- `cargo test --test issue_1452_saved_caret -- --nocapture` 통과
- `cargo test --lib paragraph::tests -- --nocapture` 통과
- `cargo test --lib issue1452 -- --nocapture` 통과
- `wasm-pack build --target web --out-dir pkg --no-pack` 통과

## 다음 스테이지 이월

- 작업지시자 수동 확인에서 실제 Studio 키보드 위쪽 이동이 아직 첫 번째 그림 뒤가 아니라 줄 시작 쪽으로 이동하는 현상이 남았다.
- 두 번째 그림 조판부호 뒤를 마우스로 클릭했을 때 캐럿이 해당 위치로 이동하지 않는 현상이 남았다.
