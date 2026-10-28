# Task M100 #1452 Stage 5 시작 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `c25aa7ee task 1452: 그림 삽입과 Shift+Tab 내어쓰기 개선`
  - `f5b2b035 task 1452: PNG 알파 BinData 보존 검증 추가`
  - `ab601c83 task 1452: 개체 속성 창 크기 고정`
  - `f12af6c9 task 1452: 그림 전체 투명도 구현`

## 1. 배경

한컴에서는 외부 파일 관리자의 그림 파일을 문서 위로 드래그해 놓으면 그림이 바로 삽입되며, 기본값은
`글자처럼 취급`이고 크기는 원래 그림 크기다. 현재 Studio의 외부 이미지 drop 경로는 입력 메뉴의 그림
삽입과 같은 배치 모드로 들어가 사용자가 다시 클릭/드래그해야 한다.

입력 메뉴 `그림` 명령은 기존처럼 사용자가 영역을 지정하는 동작을 유지한다.

## 2. 작업 범위

- 외부 이미지 파일 drop 경로만 즉시 삽입으로 바꾼다.
- drop 좌표 hit-test 결과 위치에 원본 그림 크기(`naturalWidth/Height * 75 HWPUNIT`)로 삽입한다.
- 원본 크기가 drop 대상 페이지의 본문 크기를 넘으면 비율을 유지해 자동 축소한다.
- 삽입 직후 `treatAsChar=true`를 적용해 한컴 drop 기본값과 맞춘다.
- drop 삽입 후 커서는 글자처럼 취급된 그림의 뒤쪽 logical offset으로 이동한다.
- 그림만 있는 TAC 문단의 문단부호는 그림 앞이 아니라 그림 뒤에 표시되게 한다.
- 그림 뒤에서 Enter를 눌렀을 때 새 빈 문단이 그림과 겹치지 않고 다음 줄에 배치되게 한다.
- 메뉴 `삽입 > 그림`의 `enterImagePlacementMode` 경로는 변경하지 않는다.

## 3. 검증 계획

- `cd rhwp-studio && npx tsc --noEmit`
- `git diff --check`
- 필요 시 외부 이미지 파일을 `http://localhost:7700/` 문서 본문에 drop해 즉시 삽입되는지 수동 확인

## 4. 구현 결과

- `InputHandler.insertDroppedImageAtClientPoint()`를 추가해 외부 이미지 drop 경로에서 내부 hit-test 위치에
  즉시 삽입하도록 했다.
- drop 삽입 크기는 원본 픽셀 크기 기준 `naturalWidth * 75`, `naturalHeight * 75` HWPUNIT으로 계산한다.
- 단, 원본 크기가 drop 대상 페이지의 본문 폭/높이를 넘으면 원본 비율을 유지하면서 본문 안에 들어가도록
  축소한다. 여러 단 문서에서는 drop 좌표가 속한 단 폭을 우선 기준으로 사용한다.
- 삽입 직후 `setPictureProperties({ treatAsChar: true })`를 호출해 한컴 drop 기본값인 `글자처럼 취급`을
  적용한다. 글상자 내부 경로는 by-path picture setter를 사용한다.
- `insert_picture_native()` 반환 JSON에 `logicalOffset`을 추가해 Studio가 삽입된 그림 뒤 커서 위치를
  정확히 받을 수 있게 했다. 외부 drop 경로는 삽입/속성 적용 직후 이 위치로 커서를 이동한다.
- `main.ts`의 이미지 drop 처리만 새 메서드로 연결했다. 입력 메뉴 `그림` 명령의 배치 모드 경로는 변경하지
  않았다.
- TAC 그림만 있는 줄의 빈 `TextRun` 문단 끝 표시 위치를 그림 시퀀스의 끝 좌표로 이동시켜 문단부호가
  그림 뒤에 보이게 했다.
- `reflow_line_segs()`가 텍스트 없는 문단이라도 TAC 그림/도형/표/수식/폼 컨트롤이 있으면 기본 줄 높이로
  축소하지 않고 인라인 개체 높이를 보존하도록 했다. 이로써 그림 뒤 Enter 후 새 빈 문단이 그림 아래 줄에
  배치된다.

## 5. 검증 결과

- `cd rhwp-studio && npx tsc --noEmit` 통과
- `git diff --check` 통과
- Browser smoke 확인: `http://localhost:7700/`
  - URL/title: `http://localhost:7700/`, `rhwp-studio`
  - DOM snapshot에서 리본/툴바가 렌더링됨
  - Vite/webpack류 프레임워크 오류 오버레이 없음
  - console warn/error 없음
  - 현 Browser 세션의 screenshot API는 `Page.captureScreenshot` 시간 초과로 캡처 증거를 남기지 못함
- 큰 이미지 자동 축소 보정 후 `cd rhwp-studio && npx tsc --noEmit` 재통과
- 큰 이미지 자동 축소 보정 후 `git diff --check` 재통과
- drop 후 커서 위치 보정 후 `cd rhwp-studio && npx tsc --noEmit` 재통과
- drop 후 커서 위치 보정 후 `cargo test --lib issue1452 -- --nocapture` 통과
  - 7개 테스트 통과
  - `issue1452_insert_picture_returns_logical_offset_after_picture` 추가
- drop 후 커서 위치 보정 후 `cargo fmt --check` 통과
- drop 후 커서 위치 보정 후 `git diff --check` 재통과
- 문단부호/Enter 후속 보정 후 `cargo fmt --check` 재통과
- 문단부호/Enter 후속 보정 후 `cargo test --lib issue1452 -- --nocapture` 통과
  - 8개 테스트 통과
  - `issue1452_enter_after_dropped_inline_picture_keeps_next_para_below_picture` 추가
- 문단부호/Enter 후속 보정 후 `cd rhwp-studio && npx tsc --noEmit` 재통과
- 문단부호/Enter 후속 보정 후 `wasm-pack build --target web --out-dir pkg` 통과
- 실제 외부 파일 관리자에서 그림 파일을 OS drag/drop하는 동작은 Browser 자동화로 재현하지 못해 수동 검증 필요
