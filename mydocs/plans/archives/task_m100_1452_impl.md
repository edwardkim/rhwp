# Task M100 #1452 구현계획서

- 대상 이슈: #1452
- 기준 문서: `mydocs/plans/task_m100_1452.md`
- 작성일: 2026-06-21

## 1. 구현 목표

사용자 피드백 중 현재 코드에서 직접 개선 가능한 두 영역을 먼저 수정한다.

1. 그림 속성 변경 후 HWP 저장 시 `textWrap=InFrontOfText` 같은 배치 속성이 보존되도록 `CommonObjAttr.attr`
   비트를 동기화한다.
2. Studio 그림 삽입 단계에서 decode 실패나 유효하지 않은 배치 클릭이 조용히 사라지지 않도록 사용자 안내를
   추가한다.
3. 여러 줄 문단에서 `Shift+Tab` 내어쓰기 기준점이 첫 줄 기준 기대 위치와 맞도록 정합성을 점검하고 보강한다.

## 2. 세부 구현

### 2.1 그림 속성 attr 동기화

- `src/document_core/commands/object_ops.rs`의 그림 속성 적용 경로에 공통 attr 비트 갱신 헬퍼를 추가한다.
- 기존 표 속성 적용 경로와 같은 비트 매핑을 사용한다.
- `textWrap` 변경 시 bit 21-23을 갱신한다.
- `vertRelTo`, `vertAlign`, `horzRelTo`, `horzAlign`, `treatAsChar`, `restrictInPage`, `allowOverlap`,
  `sizeProtect` 변경 시 이미 갱신 중인 비트와 enum 필드가 일관되는지 확인한다.

### 2.2 테스트

- 기존 `issue1151_v9_insert_picture_body_floating_default` 테스트는 유지한다.
- 그림 속성 변경 API로 `textWrap=InFrontOfText`를 적용한 뒤 `common.text_wrap`과 `common.attr`의 bit 21-23이
  모두 `3`인지 확인하는 테스트를 추가한다.
- 가능하면 `TopAndBottom`, `BehindText`도 같은 테스트에서 확인한다.

### 2.3 Studio 삽입 실패 안내

- `rhwp-studio/src/command/commands/insert.ts`
  - `Image` decode를 `load/error` 양쪽으로 처리한다.
  - 실패 시 기존 UI 알림 체계가 있으면 사용하고, 없으면 `alert`보다 덜 침습적인 기존 status/error 표출 경로를
    우선 탐색한다.
- `rhwp-studio/src/engine/input-handler-table.ts`
  - hit-test 실패 시 배치 모드를 즉시 조용히 취소하는 대신 안내를 표시한다.
  - WASM `insertPicture` 결과가 실패하거나 예외가 발생하면 사용자에게 표시한다.

### 2.4 Shift+Tab 내어쓰기 정합성

- `rhwp-studio/src/engine/input-handler-keyboard.ts`
  - `Tab` + `shiftKey` 호출 경로가 폼 필드 이동과 충돌하지 않는지 유지한다.
- `rhwp-studio/src/engine/input-handler.ts`
  - 현재 구현은 현재 줄의 `lineInfo.charStart` 위치를 기준으로 `cursorRect.x - lineStartRect.x`를 계산한다.
  - 여러 줄 문단에서는 첫 줄의 기준 위치를 별도로 구해 둘째 줄 이후의 hanging indent 목표와 비교한다.
  - 본문 문단과 1단계 표 셀 문단을 우선 대상으로 한다.
- 필요 시 WASM 쪽에 첫 줄 기준 커서 좌표를 얻는 보조 API를 추가하되, 기존 `getLineInfo`/`getCursorRect`
  조합으로 해결 가능한지 먼저 검토한다.

## 3. 검증

- Rust focused test:
  - `cargo test --lib issue1151_v9_insert_picture_body_floating_default -- --nocapture`
  - `cargo test --lib <신규 그림 attr 테스트명> -- --nocapture`
- Studio 타입 검사:
  - `cd rhwp-studio && npx tsc --noEmit`
- Shift+Tab focused 검증:
  - 여러 줄 본문 문단에서 첫 줄 기준 목표 좌표와 둘째 줄 이후 시작 좌표 비교
  - 표 셀 문단에서 동일 동작 확인
- 변경 범위가 커질 경우:
  - `cd rhwp-studio && npm test`

## 4. 리스크

- `common.attr`는 HWP 라운드트립 보존에 사용되므로, 변경한 속성의 비트만 갱신하고 나머지 비트는 보존해야 한다.
- 기존 문서에서 파싱된 미지원 attr 비트는 유지해야 한다.
- 삽입 실패 안내는 문서 변경 이벤트나 undo stack을 오염시키지 않아야 한다.
