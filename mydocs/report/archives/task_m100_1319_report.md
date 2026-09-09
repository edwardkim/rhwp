# Task #1319 완료 보고서

## 요약

문단 서식 변경을 rhwp-studio의 기존 `CommandHistory` 기반 Undo/Redo 체계에 편입했다.

이번 작업의 핵심은 문단 모양 대화상자, toolbar/menu 정렬·줄 간격, #1318 `Shift+Tab` 내어쓰기처럼
문단 속성을 바꾸는 작업이 사용자 관점에서 `Ctrl+Z`/`Ctrl+Y`로 일관되게 복원되도록 하는 것이다.

작업지시자 동작 테스트 중 본문 텍스트 블록 선택 후 굵게 적용 시 Redo가 누락되는 문제가 추가로 확인되었다.
이는 문단 서식과 별도 도메인이지만, 사용자 경험 기준에서는 같은 Undo/Redo 품질 문제이므로 이번 범위에
포함해 보강했다.

## 변경 내용

### 문단 서식 Undo/Redo

- `src/document_core/commands/formatting.rs`
  - `set_para_shape_id_native()` 추가
  - `set_cell_para_shape_id_native()` 추가
  - 문단 서식 복원 시 `ParaProperties` JSON을 재적용하지 않고 `para_shape_id`를 직접 복원

- `src/wasm_api.rs`
  - `setParaShapeId`
  - `setCellParaShapeId`

- `rhwp-studio/src/core/wasm-bridge.ts`
  - `setParaShapeId()`
  - `setCellParaShapeId()`

- `rhwp-studio/src/engine/command.ts`
  - `ApplyParaFormatCommand` 추가
  - 최초 실행 시 before/after `paraShapeId` 저장
  - Undo/Redo 시 저장된 ID 직접 복원
  - `mergeWith()`는 병합하지 않음

- `rhwp-studio/src/engine/input-handler.ts`
  - 본문 문단 target 산정
  - 일반 표 셀 문단 target 산정
  - 문단 모양 대화상자 적용 경로를 command로 전환
  - toolbar/menu 정렬·줄 간격 경로를 command로 전환
  - #1318 `Shift+Tab` 내어쓰기 경로를 command로 전환

### 글자 서식 Undo/Redo 보강

- `src/document_core/commands/formatting.rs`
  - `set_char_shape_id_native()` 추가
  - `set_char_shape_id_in_cell_native()` 추가

- `src/wasm_api.rs`
  - `setCharShapeId`
  - `setCharShapeIdInCell`

- `rhwp-studio/src/core/wasm-bridge.ts`
  - `setCharShapeId()`
  - `setCharShapeIdInCell()`

- `rhwp-studio/src/engine/command.ts`
  - `ApplyCharFormatCommand`를 before/after `charShapeId` 복원 방식으로 보강
  - Redo 시 props 재적용 대신 after `charShapeId` 직접 복원

## 설계 판단

조회용 `ParaProperties`/`CharProperties` JSON을 Undo/Redo payload로 재사용하지 않았다.

문단 속성 조회값에는 UI 표시용 단위가 섞여 있고, 글자 서식도 `charShapeId` 직접 복원을 props parser가
처리하지 않는다. 따라서 복원 payload는 적용 전/후 shape ID로 제한했다.

이번 작업은 전술적 1차 구현이다. 편집 액션 라우터, WASM mutation transaction, dirty scope 최적화는 별도
이슈 #1320으로 분리했다.

## 지원 범위

성공 판정 범위:

- 본문 문단 문단 모양 Undo/Redo
- toolbar/menu 정렬 Undo/Redo
- toolbar/menu 줄 간격 Undo/Redo
- #1318 `Shift+Tab` 내어쓰기 Undo/Redo
- 일반 표 셀 문단의 문단 서식 Undo/Redo
- 본문 텍스트 블록 선택 후 굵게 Undo/Redo

1차 제외 범위:

- 머리말/꼬리말 문단
- 각주/미주 문단
- 글상자 문단
- 중첩 표 문단
- 문단 번호/글머리표 definition table 자체의 undo cleanup

## 후속 이슈

- #1320 `편집 액션 라우터와 Undo/Redo 트랜잭션 아키텍처 정비`

## 검증

| 항목 | 결과 |
|---|---|
| `cargo fmt --all` | 통과 |
| `cargo test --lib` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| `npm run build` (`rhwp-studio/`) | 통과 |
| `git diff --check` | 통과 |
| rhwp-studio 동작 테스트 | 통과 |

`cargo test --lib` 결과:

```text
test result: ok. 1602 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 131.38s
```

## 판정

작업지시자가 rhwp-studio에서 문단 서식 Undo/Redo와 글자 서식 Redo 보강 동작 테스트 성공을 확인했다.

이번 이슈는 성공으로 완료 판정한다.
