# Task #1319 Stage 2 — 문단/글자 서식 Undo/Redo 구현 보고

## 범위

- 이슈: #1319 `문단 서식 변경 Undo/Redo 커맨드 체계화`
- 브랜치: `local/task1319`
- 선행 승인:
  - 수행계획서 승인
  - Stage 1 진단 승인
  - 구현 계획서 보강
- 후속 아키텍처 분리:
  - #1320 `편집 액션 라우터와 Undo/Redo 트랜잭션 아키텍처 정비`

## 구현 요약

문단 서식 변경을 기존 `CommandHistory`에 `EditCommand`로 편입했다.
동작 테스트 중 확인된 글자 서식 Redo 누락도 사용자 경험 기준에서 같은 Undo/Redo 품질 문제로 보아
함께 보강했다.

핵심 구현:

- Rust/WASM:
  - `setParaShapeId`
  - `setCellParaShapeId`
- TypeScript bridge:
  - `WasmBridge.setParaShapeId()`
  - `WasmBridge.setCellParaShapeId()`
- History command:
  - `ApplyParaFormatCommand`
  - 최초 실행: before `paraShapeId` 저장 → props 적용 → after `paraShapeId` 저장
  - undo: before `paraShapeId` 직접 복원
  - redo: props 재적용 없이 after `paraShapeId` 직접 복원
  - `mergeWith()`는 병합하지 않음
- InputHandler:
  - 본문 문단 target 산정
  - 일반 표 셀 문단 target 산정
  - 문단 모양 대화상자 경로 command 전환
  - toolbar/menu 정렬/줄 간격 경로 command 전환
  - #1318 Shift+Tab 내어쓰기 경로 command 전환
  - 문단 번호 모양 대화상자의 번호 해제 직접 WASM 호출 제거

추가 사용자 경험 보강:

- 작업지시자 동작 테스트 중 본문 텍스트 블록 선택 → 굵게 → Undo → Redo에서 Redo 적용 누락이 확인되었다.
- 이 문제는 문단 서식과 별도 도메인이지만, 사용자가 기대하는 Undo/Redo 품질 기준에서는 같은 완료 조건으로 보아야 한다.
- `ApplyCharFormatCommand`도 `charShapeId` before/after를 저장하고 Undo/Redo 때 직접 복원하도록 보강했다.
- Rust/WASM:
  - `setCharShapeId`
  - `setCharShapeIdInCell`
- TypeScript bridge:
  - `WasmBridge.setCharShapeId()`
  - `WasmBridge.setCharShapeIdInCell()`

## 기존 Undo/Redo와의 충돌 회피

- `recordWithoutExecute()`를 사용하지 않았다.
- `SnapshotCommand`를 사용하지 않았다.
- `redo()`에서 `execute()`가 재호출되는 기존 구조에 맞춰, redo 시 after ID 직접 복원 분기를 넣었다.
- 글자 서식 command도 redo 시 props 재적용 대신 after `charShapeId` 직접 복원 분기를 사용한다.
- `applyParaFormat` command는 page-local refresh 대상에 추가하지 않고 full `afterEdit()` 경로를 유지했다.
- 문단 서식 변경 후 selection 유지가 가능하도록 `executeOperation()`에서 `applyParaFormat`을 `applyCharFormat`과
  같은 문서 구조 불변 command로 취급했다.

## 1차 지원 문맥

지원:

- 본문 문단
- 일반 표 셀 문단
- 선택 범위 내 다중 본문 문단
- 같은 표 셀 내부의 다중 셀 문단

1차 제외:

- 머리말/꼬리말
- 각주/미주
- 글상자
- 중첩 표
- 문단 번호/글머리표 definition table 자체의 undo cleanup

제외 문맥에서는 직접 WASM mutation을 유지하지 않고 no-op/info 경로로 빠지도록 했다.

## 검증

실행:

```text
cargo fmt --all
npm run build
  - 실행 위치: rhwp-studio/
cargo test --lib
docker compose --env-file .env.docker run --rm wasm
npm run build
  - 실행 위치: rhwp-studio/
  - 새 WASM export 반영 후 재검증
```

결과:

- `rhwp-studio` build 통과
- `cargo test --lib` 통과
  - 1602 passed
  - 0 failed
  - 6 ignored
- WASM 빌드 통과
- 새 WASM 산출물 기준 `rhwp-studio` build 재통과

참고:

- 저장소 루트에는 `package.json`이 없어 루트 `npm run build`는 실행 위치 오류로 실패했다.
  이후 `rhwp-studio/`에서 다시 실행해 통과했다.

## 작업지시자 동작 테스트

rhwp-studio에서 다음 동작 테스트를 완료했다.

1. 본문 문단
   - 문단 모양 대화상자 적용 → Undo → Redo
   - toolbar 정렬 변경 → Undo → Redo
   - 줄 간격 변경 → Undo → Redo
   - Shift+Tab 내어쓰기 → Undo → Redo
2. 일반 표 셀
   - 셀 문단에서 문단 모양 적용 → Undo → Redo
   - 셀 문단에서 Shift+Tab 내어쓰기 → Undo → Redo
3. 기존 회귀
   - 텍스트 입력/삭제 Undo/Redo
   - 글자 서식 Undo/Redo
   - 본문 텍스트 블록 선택 → 굵게 → Undo → Redo
   - 이미지/도형 이동 또는 크기 조절 Undo/Redo

작업지시자 판정:

```text
2026-06-07 통과
```
