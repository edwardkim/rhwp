# Task #2903 — MoveTableCommand undo가 moveTableOffset 반환값을 버리는 결함 수정

## 이슈

https://github.com/edwardkim/rhwp/issues/2903

## 버그 요약

`rhwp-studio/src/engine/command.ts` 의 `MoveTableCommand`(표 이동 Undo/Redo 명령)에서
`execute()` 와 `undo()` 가 동일한 WASM API(`moveTableOffset`)를 호출하면서도 그 반환값을
다루는 방식이 비대칭이었다.

- `execute()`: `wasm.moveTableOffset(...)` 의 반환값 `{ ppi, ci }` 를 `this.resultPpi`/
  `this.resultCi` 에 캡처해 저장하고, 반환하는 `DocumentPosition` 도 이 값을 사용한다 —
  표는 문단에 걸쳐 있는 anchor 객체라 오프셋 이동으로 귀속 문단(ppi)이 바뀔 수 있고,
  WASM 이 알려준 값을 신뢰해야 한다는 전제가 이미 코드에 반영돼 있다.
- `undo()`: 정확히 대칭인 역이동(`-deltaH, -deltaV`)을 호출하지만 그 **반환값을 완전히
  버리고**, 커맨드 생성 시점에 캡처해 둔 stale `this.ppi`/`this.ci` 를 그대로 반환했다.

표 이동과 문단 구조 변경(삽입/삭제/병합 등으로 문단 인덱스가 shift)이 같은 히스토리
세션에서 섞이면, undo 후 반환되는 `DocumentPosition` 이 실제로 표가 재배치된 문단이
아니라 stale 문단을 가리켜 커서/선택 복원이 잘못될 수 있었다.

## 수정

`undo()` 도 `execute()` 와 대칭으로 `wasm.moveTableOffset(...)` 의 반환값을 캡처해
`this.ppi`/`this.ci` 를 갱신하고, 반환 `DocumentPosition` 도 그 갱신된 값을 사용하도록
수정했다.

```ts
undo(wasm: WasmBridge): DocumentPosition {
  const result = wasm.moveTableOffset(this.sec, this.resultPpi, this.resultCi, -this.deltaH, -this.deltaV);
  this.ppi = result.ppi;
  this.ci = result.ci;
  return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
}
```

- 파일: `rhwp-studio/src/engine/command.ts` (`MoveTableCommand.undo`)
- `.rs` 변경 없음, TypeScript 전용 수정.

## 테스트

신규 파일: `rhwp-studio/tests/undo-move-table-ppi.test.ts`

기존 `undo-delete-char-count.test.ts` 와 동일한 스타일의 소스 가드 테스트로, `command.ts`
소스에서 `MoveTableCommand.undo()` 블록을 추출해:

1. `undo()` 가 `moveTableOffset` 반환값을 변수(`result`)로 캡처하는지
2. 그 값으로 `this.ppi`/`this.ci` 를 갱신하는지
3. 반환 `DocumentPosition` 이 갱신된 `this.ppi` 를 쓰는지

를 검증한다. 회귀 가드로 `execute()` 가 여전히 `resultPpi`/`resultCi` 를 갱신하는지도
함께 확인한다.

### Red (수정 전)

```
git stash push -- src/engine/command.ts 후 실행:

✖ MoveTableCommand.undo 는 moveTableOffset 반환값을 캡처해 this.ppi/this.ci 를 갱신한다 (1.4737ms)
  AssertionError: undo() 가 moveTableOffset 반환값을 변수로 캡처해야 함(execute() 와 대칭)
tests 2, pass 1, fail 1
```

### Green (수정 후)

```
✔ MoveTableCommand.undo 는 moveTableOffset 반환값을 캡처해 this.ppi/this.ci 를 갱신한다 (1.2706ms)
✔ MoveTableCommand.execute 도 여전히 moveTableOffset 반환값으로 resultPpi/resultCi 를 갱신한다 (회귀 가드) (0.2953ms)
tests 2, pass 2, fail 0
```

### 전체 테스트 (`node --test tests/*.test.ts`)

수정 후 전체 실행 결과 — 총 477개 중 476개 통과, `cell-flow-boundary.test.ts` 1건만 실패
(작업 지침에 따라 이 파일만 실패가 허용됨, 본 변경과 무관한 기존 결함).

```
ℹ tests 477
ℹ pass 476
ℹ fail 1  (cell-flow-boundary.test.ts — 허용된 기존 실패)
```

### `npx tsc --noEmit`

베이스라인과 동일하게 정확히 2건의 기존 TS2307 오류만 존재(`@wasm/rhwp.js` 모듈 타입
선언 누락 — WASM 빌드 산출물 미생성으로 인한 환경적 오류, 본 변경 이전부터 존재).
본 변경으로 추가된 신규 오류 없음.

```
src/core/wasm-bridge.ts(1,44): error TS2307: Cannot find module '@wasm/rhwp.js' or its corresponding type declarations.
src/hwpctl/index.ts(417,57): error TS2307: Cannot find module '@wasm/rhwp.js' or its corresponding type declarations.
```

## 참고: 환경 비고

이 워크트리(`rhwp-wt-cc`)에는 `node_modules` 가 설치돼 있지 않아 `npx tsc` 및 일부
third-party 의존성을 import 하는 테스트(`canvaskit-resource-key.test.ts` 등)가 실행 전
단계에서 실패했다. 검증을 위해 `npm install` 을 실행해 로컬 devDependencies 를 설치한
뒤 위 결과를 얻었다(문서화된 `dev_environment_guide.md` 표준 절차, 프로젝트 코드 변경
아님).
