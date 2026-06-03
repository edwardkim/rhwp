# Task 283 수행계획서: 중첩 구조 hitTest/커서 이동 정교화

## 목표

복잡한 중첩 HWP 문서(표>표, 표>표>표, 표>셀>글상자)에서:
1. hitTest가 최심(가장 안쪽) 자식 요소까지 정확히 경로 추적
2. 커서 좌→우 이동 시 줄 끝에서 다음 줄 시작으로 이동
3. F5 셀 선택 모드에서 병합 셀을 단일 셀로 처리하여 방향키 이동

## 테스트 파일

- `samples/table-path-bug.hwp`: 표(5×3) > 셀 > 내부표(3×3) + 내부표(5×5) + 인라인 Shape
- `samples/group-box.hwp`: 개체묶기(14개 글상자)

## 구현 단계

### 1단계: hitTest 최심 경로 추적 (WASM + Rust)

**현상**: 표>셀>내부표>셀 클릭 시 외부 표 셀에 커서가 위치하거나, 에러 발생
**원인**: hitTest가 중첩 표/글상자까지 재귀 탐색하지 않음

수정 대상:
- `src/document_core/queries/cursor_rect.rs`: hitTest 재귀 탐색
- `src/document_core/queries/rendering.rs`: 셀 내 중첩 컨트롤 경로 반환

### 2단계: 커서 좌→우 이동 (줄 끝 → 다음 줄 시작)

**현상**: 셀 내에서 오른쪽 방향키를 눌러 줄 끝에 도달하면 커서가 멈춤
**기대 동작**: 줄 끝에서 다음 줄의 가장 왼쪽으로 이동

수정 대상:
- `rhwp-studio/src/engine/cursor.ts`: `moveHorizontal` 줄 경계 처리
- `rhwp-studio/src/engine/input-handler-keyboard.ts`: 방향키 핸들러

### 3단계: F5 셀 선택 모드 병합 셀 이동

**현상**: F5 모드에서 방향키로 이동 시 병합된 셀의 각 행/열을 개별 셀로 인식
**기대 동작**: 병합 셀은 한 개의 셀로 취급하여 건너뜀

수정 대상:
- `rhwp-studio/src/engine/cursor.ts`: `moveCellSelection` 병합 셀 건너뛰기
- `src/wasm_api.rs`: 셀 병합 정보 조회 API (필요시)

## 검증 방법

- `samples/table-path-bug.hwp` 로드 후:
  - 내부표 셀 클릭 → 정확한 셀에 캐럿 생성
  - 셀 내 텍스트에서 좌/우 방향키 → 줄 경계 넘기
  - F5 → 방향키 → 병합 셀 단위 이동
- `samples/group-box.hwp` 로드 후:
  - 글상자 클릭 → 글상자 안에 캐럿 생성
- 기존 E2E 테스트 통과
