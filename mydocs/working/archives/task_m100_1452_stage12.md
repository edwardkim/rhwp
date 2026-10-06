# Task M100 #1452 Stage 12 작업 기록

## 배경

Stage 11에서 TAC 그림 사이 Enter 분할과 control-only 문단의 native 세로 이동 후보를 보정했다.
작업지시자 수동 확인 결과, 실제 Studio 조작에서는 다음 문제가 아직 남아 있다.

## 목표

- 두 번째 그림 끝에서 키보드 위쪽 방향키를 누르면 첫 번째 그림 뒤로 이동해야 한다.
- 두 번째 그림 뒤 조판부호 위치를 마우스로 클릭하면 캐럿이 두 번째 그림 뒤로 이동해야 한다.
- 임의 좌표 보정이 아니라 그림 bbox와 logical control offset을 기준으로 hit-test와 커서 이동을 맞춘다.
- 일반 텍스트에서도 `End`로 줄 끝에 놓인 뒤 위/아래 방향키 이동이 줄 끝 affinity를 유지해야 한다.
- 첫 번째 그림 앞에서 Enter로 빈 문단을 만든 뒤에도, 두 번째 그림 끝에서 ↑ 이동하면 첫 번째 그림 뒤로 이동해야 한다.
- 임시 `1111` wrap 샘플의 텍스트 영역에서 `Home`/`End`/마우스 클릭/위아래 이동이 현재 시각 줄 기준으로 동작해야 한다.

## 확인할 내용

- `moveVertical`이 같은 logical offset이라도 목표 시각 줄의 bbox 기반 rect를 반환하도록 보정했다.
- 마우스 hit-test는 `inline_shape_positions` 대신 실제 `ImageNode` bbox를 우선 사용해 TAC 그림 좌우/뒤 조판부호 위치를 매핑한다.
- Studio `CursorState`는 `End` 직후 수직 이동 시 이전 글자 기준 줄 정보를 사용하고, preferred X를 줄 끝으로 전달한다.
- control-only TAC 문단의 재계산 line segment가 모두 같은 `text_start`를 가질 때, 줄 시작 logical offset을 순차 보정해 각 줄이 최소 한 개의 inline control을 소비하도록 했다.
- soft-wrap 경계 offset은 이전 줄 끝과 다음 줄 시작을 동시에 뜻할 수 있으므로 `getCursorRectOnLine`으로 시각 줄 affinity를 명시한다.
- Studio `CursorState`는 Home/End 직후 해당 줄의 rect를 보존하고, 수직 이동 결과가 줄 끝 경계이면 `atLineEnd` affinity를 계속 유지한다.

## IAB 확인

- `localhost:7700`에서 임시 `1111` wrap 샘플을 열고 텍스트 영역을 직접 키보드로 확인했다.
- 1행 중간 클릭 후 `End`는 1행 끝, `Home`은 1행 시작으로 동작했다.
- 2행 중간 클릭 후 `End`는 2행 끝으로 이동하지만, 이어서 `Home`을 누르면 2행 시작이 아니라 1행 끝 위치에 캐럿이 그려지는 문제가 재현됐다.
- 2행 끝에서 `ArrowUp`/`ArrowDown`, 3행 끝에서 `End`/`ArrowUp`도 soft-wrap 경계 offset의 시각 줄 affinity가 불안정한 증상을 보였다.
- 1행 `End` 후 다시 `End`를 누르면 1행 끝에 유지되어야 하나, 같은 경계 offset을 2행 시작으로 재해석해 2행 끝으로 이동하는 문제가 추가로 재현됐다.
- 수정 후 동일 샘플을 새 WASM으로 새로고침해 2행 `End -> Home`이 2행 시작에 캐럿을 표시하는 것을 확인했다.
- 2행 `End -> ArrowUp -> ArrowDown`도 첫 줄 끝과 두 번째 줄 끝 사이에서 이동하며, 이전처럼 다음 줄로 건너뛰지 않는 것을 확인했다.
- `moveToLineEnd`도 `atLineEnd` 상태에서는 이전 줄 정보를 유지하도록 보정했고, IAB에서 1행 `End -> End`가 1행 끝에 유지되는 것을 확인했다.

## 검증

- `cargo test --test issue_1452_saved_caret -- --nocapture` 통과
- `cargo fmt --check` 통과
- `git diff --check` 통과
- `npm run build` 통과
- `npm test` 통과
- `wasm-pack build --target web --out-dir pkg --no-pack` 통과
