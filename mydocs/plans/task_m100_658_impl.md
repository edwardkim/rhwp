# Task M100 #658 구현계획서

## 타이틀

rhwp-studio 마우스 드래그 텍스트 선택 rect 정합화 및 드래그 렌더링 성능 개선

## 구현 원칙

본 작업은 선택 하이라이트가 페이지 밖으로 튀는 정확도 결함을 먼저 정정하고, 이후 드래그 중 DOM 갱신 비용을 줄인다.

진행 순서는 다음 원칙을 따른다.

- native `get_selection_rects_native()`가 반환하는 rect의 정확도를 우선 확보한다.
- frontend 최적화는 native 오류를 숨기는 방향이 아니라, 정확한 rect를 더 가볍게 렌더링하는 방향으로만 적용한다.
- 기존 본문/다단/표 셀 선택 동작에 회귀를 만들지 않는다.
- 각 단계 완료 후 완료보고서 작성과 승인 요청을 거친다.

## 변경 후보 파일

| 파일 | 목적 |
|------|------|
| `src/document_core/queries/cursor_nav.rs` | 선택 rect 계산 정합화, 페이지/컨테이너 경계 클램프 |
| `tests/issue_658_text_selection_rects.rs` | native rect 회귀 테스트 후보 |
| `rhwp-studio/src/engine/selection-renderer.ts` | 하이라이트 DOM 재사용 또는 동일 rect 업데이트 생략 |
| `rhwp-studio/src/engine/input-handler.ts` | 필요 시 드래그 중 선택 갱신 경량 경로 분리 |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 필요 시 드래그 루프에서 경량 선택 갱신 호출 |
| `rhwp-studio/e2e/text-selection-drag.test.mjs` | web 선택 하이라이트 bbox 검증 후보 |
| `mydocs/working/task_m100_658_stage{N}.md` | 단계별 완료보고서 |
| `mydocs/report/task_m100_658_report.md` | 최종 결과보고서 |

## 단계 1 — 재현 계측 및 native rect 회귀 가드

### 목표

첨부 영상의 증상을 코드 레벨에서 재현할 수 있는 관찰 지점을 확보한다. 가능하면 native rect가 페이지/컨테이너 밖으로 나가는 조건을 테스트로 고정한다.

### 작업

1. `get_selection_rects_native()`의 현재 계산 경로를 정리한다.
2. 오른쪽 자료 박스 선택에 해당하는 문단/컨텍스트를 확인한다.
3. 선택 rect가 페이지 폭을 초과하는지 확인하는 native 테스트 또는 진단 코드를 작성한다.
4. 필요하면 `rhwp-studio` e2e에서 `.selection-layer` 하이라이트 bbox를 수집하는 최소 계측을 작성한다.

### 검증

- 진단 결과에 다음 값이 포함된다.
  - 선택 시작/끝 위치
  - 반환된 rect 배열
  - page width 대비 `x + width`
  - 가능하면 대상 컨테이너 bbox 대비 `x + width`
- 기존 코드에서 실패하거나 문제를 드러내는 재현 근거를 확보한다.

### 산출물

- `mydocs/working/task_m100_658_stage1.md`
- 회귀 테스트 또는 진단 도구

## 단계 2 — native selection rect 정합화

### 목표

`get_selection_rects_native()`가 페이지 바깥 또는 실제 줄/컨테이너 영역 밖으로 확장된 rect를 반환하지 않도록 정정한다.

### 작업

1. `CursorHit`에 필요한 경우 줄/텍스트 런 bbox 또는 컨테이너 경계 정보를 추가한다.
2. `lh`와 `rh`가 다른 줄/다른 컨테이너에서 온 경우를 명시적으로 처리한다.
3. `selection_continues` 분기에서 column/page 끝을 무조건 사용하지 않고 현재 줄의 실제 렌더링 경계를 우선한다.
4. 최종 rect를 페이지 영역 안으로 방어 클램프한다.
5. 셀 내부 선택은 셀 bbox 또는 TextRun bbox 기준으로 폭이 튀지 않도록 별도 검증한다.

### 검증

- 단계 1의 재현 테스트가 통과한다.
- `cargo test --test issue_658_text_selection_rects` 후보 통과
- `cargo test --lib --release` 통과

### 산출물

- `mydocs/working/task_m100_658_stage2.md`
- native rect 정정 커밋 후보

## 단계 3 — rhwp-studio 선택 하이라이트 렌더링 비용 완화

### 목표

정확한 rect를 기준으로 드래그 중 선택 하이라이트를 더 가볍게 갱신한다.

### 작업

1. `SelectionRenderer.render()`의 전량 삭제/재생성 방식을 점검한다.
2. 기존 div pool 재사용 또는 동일 rect 배열 업데이트 생략을 적용한다.
3. 필요 시 `clear()`가 실제로 필요한 경우와 render 업데이트 경로를 분리한다.
4. 드래그 중 `updateCaret()` 전체 호출이 과도한지 확인하고, 필요하면 선택 갱신 전용 경량 경로를 분리한다.
5. 드래그 중 `scrollCaretIntoView`, `emitCursorFormatState`, `updateFieldMarkers` 호출 축소 여부를 검토한다.

### 검증

- `cd rhwp-studio && npm run build` 통과
- 선택 하이라이트가 드래그 중 깜빡이거나 사라지지 않는다.
- DOM 하이라이트 개수가 선택 rect 수와 일치하고 누수되지 않는다.

### 산출물

- `mydocs/working/task_m100_658_stage3.md`
- frontend 성능 개선 커밋 후보

## 단계 4 — 통합 검증, 시각 판정, 최종 보고

### 목표

native rect 정합과 frontend 렌더링 개선을 통합 검증하고 PR 제출 가능한 상태로 정리한다.

### 작업

1. 전체 검증 명령을 실행한다.
2. 첨부 영상과 같은 화면에서 오른쪽 자료 박스 내부 텍스트 드래그를 시각 확인한다.
3. 오늘할일 상태를 갱신한다.
4. 최종 결과보고서를 작성한다.
5. 작업 단위 커밋을 정리하고 PR 본문 초안을 준비한다.

### 검증

- `cargo test --lib --release`
- `cargo test --test issue_658_text_selection_rects` 후보
- `cd rhwp-studio && npm run build`
- 가능하면 `rhwp-studio` e2e 선택 드래그 테스트
- 작업지시자 시각 판정

### 산출물

- `mydocs/working/task_m100_658_stage4.md`
- `mydocs/report/task_m100_658_report.md`
- 갱신된 `mydocs/orders/20260507.md`
- PR 제출 가능한 커밋 세트

## 검증 명령 후보

```bash
cargo test --lib --release
cargo test --test issue_658_text_selection_rects
cd rhwp-studio && npm run build
```

E2E를 추가하는 경우:

```bash
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700
node e2e/text-selection-drag.test.mjs
```

## 승인 요청

이 구현계획서 승인 후 단계 1부터 진행한다.
