# task_m100_2717 처리결과 보고서 — 중첩 표 안쪽 셀 문단 시작 Backspace 무반응

- **이슈**: [#2717](https://github.com/edwardkim/rhwp/issues/2717)
- **브랜치**: `task/m100-2717-studio-nested-cell-backspace` (base `origin/devel` @ `49f38446`)
- **범위**: `rhwp-studio/src/engine/input-handler-text.ts`, `rhwp-studio/src/engine/command.ts`,
  `rhwp-studio/tests/nested-cell-backspace-merge.test.ts` (신규)
- **분류**: 결함 수정 (중첩 셀 인덱스 축 혼용 — outer/inner)
- **Rust 파일 변경 없음**

## 1. 문제

`input-handler-text.ts:260` 의 `handleBackspace` 가 "셀 문단 시작에서 이전 문단과 병합"
여부를 flat 필드 `pos.cellParaIndex` 로 판정했다.

```ts
} else if (pos.cellParaIndex! > 0) {
  // 셀 문단 시작에서 Backspace → 이전 셀 문단과 병합
  this.executeOperation({ kind: 'command', command: new MergeParagraphInCellCommand(pos) });
}
```

hit-test 는 이 flat 필드를 `cellPath[0]`, 즉 **최외곽** 셀 값으로 채운다
(`src/document_core/queries/cursor_rect.rs:1709,1727-1728` 의 `let outer = &ctx.path[0]`).
그리고 마우스 클릭 경로는 `cursor.moveToHit(hit)`(cursor.ts:275-283)가 hit 응답을
`this.position = { ...pos }` 로 **그대로** 복사한다. 따라서 중첩 표(depth ≥ 2) 안쪽 셀에서
저 판정은 바깥 셀의 문단 인덱스를 읽는다.

- **A. 무반응** — 안쪽 셀 2번째 이상 문단 시작에서 바깥이 0 이면(= 바깥 셀 문단 0 에 중첩
  표가 놓인 가장 흔한 배치) 가드가 거짓이 되어 **명령이 아예 생성되지 않는다**. Backspace 가
  먹통이 되고 undo 스택도 변하지 않는다.
- **B. 있어서는 안 될 병합** — 안쪽 셀 첫 문단(병합 대상 없음)인데 바깥이 ≥1 이면 병합이
  실행되고, `MergeParagraphInCellCommand.execute` 가 `cpi - 1 = -1` 을 넣은 cellPath 로
  `getCellParagraphLengthByPath` 를 호출한다. Rust `json_usize`
  (`src/document_core/helpers.rs:808-822`)는 ASCII 숫자만 취하므로 `-1` 에서 파싱 실패
  → `HwpError::RenderError`. `wasm-bridge.ts:966-969` 도 `CommandHistory.execute`
  (history.ts:78-80)도 이 예외를 감싸지 않아 keydown 까지 전파된다.

같은 파일의 형제 `handleDelete`(:319-320)는 커밋 `1e48274d "Fix nested table cursor paths
for PR 1291"` 에서 이미 `cellPath[last]` 축으로 교정됐고, `command.ts` 의 셀 문단 커맨드
3종도 `2442d949` 에서 `cellParaIndexOf()` 헬퍼로 통일됐다. 셀 문단 구조 편집 3키
(Enter / Delete / Backspace) 중 **Backspace 호출자 가드 한 곳만** 그 교정에서 누락됐다.

## 2. 분석

### 축 규약

`command.ts:190-205` 의 `cellParaIndexOf` 주석이 권위 근거다 —
"hit-test 는 flat 필드를 `cellPath[0]`, 즉 **최외곽** 엔트리에서 채운다 … 안쪽 셀의 값은
`cellPath[last].cellParaIndex` 다. 이를 섞으면 …ByPath API 에 바깥 축의 인덱스를 넘겨
엉뚱한 문단을 병합/분할한다." `tests/undo-nested-cell-merge-offset.test.ts:9-13,22` 가 같은
규약을 정적으로 핀하며, 정합 선례로 `input-handler-text.ts:307`(= `handleDelete`)을 지목한다.

### 의도된 동작이 아님을 확인한 근거

```
$ git log --oneline -S "pos.cellParaIndex! > 0" -- rhwp-studio/src/engine/input-handler-text.ts
f0f7f1a4 Initial commit: rhwp v0.5.0
```

이 판정은 depth 1 셀만 있던 초기 커밋 이후 한 번도 손대지 않았다. `1e48274d` 는 같은 파일의
`handleDelete`/`getTextAt`/`insertTextAtRaw`/`deleteTextAt` 네 곳의 축을 바꾸면서
`handleBackspace` 는 diff 에 한 줄도 포함하지 않았다. `2442d949` 는 커맨드 6지점을
`cellParaIndexOf` 로 통일하면서 호출자 가드는 훑지 않았다. 즉 의도적 예외가 아니라 누락이다.

호출되는 `MergeParagraphInCellCommand`(command.ts:1227-1274)는 `isNestedCell` 분기와
`...ByPath` 호출, `cellParaIndexOf` 를 이미 갖추고 있다 — "중첩은 지원 안 함"이 아니라
호출자 가드만 축이 다르다.

### 축 정합 전수 확인

`grep -rn "cellParaIndex!\? *[<>+]" rhwp-studio/src --include=*.ts` 로 flat 값을 비교
연산에 쓰는 지점은 `cursor.ts:202`(같은 축끼리의 정렬 비교 — 무해)와
`input-handler-text.ts:260` 둘뿐이었다. 후자만 결함이다.

## 3. 변경

1. `command.ts` — 단일 축 헬퍼 `cellParaIndexOf` 를 `export` (본문 로직 변경 없음).
   인라인 복제 대신 헬퍼를 재사용한 이유는 `undo-nested-cell-merge-offset.test.ts:44` 가
   "인덱스 축 유도가 여러 곳에 복제되면 한쪽만 고쳐지는 회귀가 재발한다"고 명시하기 때문이다.
2. `input-handler-text.ts` — `pos.cellParaIndex! > 0` → `cellParaIndexOf(pos) > 0`,
   `[#2717]` 주석으로 축과 두 오작동 방향 기록.

동작 변화 범위:

- depth 1(비중첩): `cellPath[0]` = 최외곽 = flat 이므로 **완전 무변화**.
- `cellPath` 부재(레거시 flat 위치): 헬퍼가 `pos.cellParaIndex!` 로 폴백 → **무변화**.
- depth ≥ 2: 안쪽 축으로 판정 → A(병합 수행)·B(무동작) 교정.

`export` 추가는 기존 가드 테스트에 영향 없다(`/function cellParaIndexOf\s*\(/` 는
`export function ...` 에도 매치). `wasm.<mutator>(` 호출을 늘리지 않아
`mutation-routing-guard.test.ts` 의 BASELINE 도 불변이다.

## 4. 검증

### 신규 테스트 — `rhwp-studio/tests/nested-cell-backspace-merge.test.ts`

소스 복제 없이 **실제** `input-handler-text.ts` 를 로드해 `executeOperation`/`wasm` 호출을
캡처하는 행위 테스트 4건.

- `command.ts` 의 parameter property 때문에 Node strip-only 로더로는 직접 import 가 불가하다
  (`TypeScript parameter property is not supported in strip-only mode` — 실측 확인).
  그래서 `process.execPath` 자식 프로세스를 `--experimental-transform-types` 로 띄우고
  `module.registerHooks` 로 `@/` 별칭만 매핑한다.
- `node_modules/.bin` 실행 파일에 의존하지 않는다. (기존 `cell-flow-boundary.test.ts` 는
  `.bin/tsc` 를 `spawnSync` 해 Windows 에서 `status=null` 로 실패한다 — 아래 미실행 항목 참조.)
- TypeScript 7(Go 포트)은 JS API `ts.transpileModule` 을 노출하지 않아
  (`typeof ts.transpileModule === 'undefined'` 실측) 라이브러리 트랜스파일 경로는 쓸 수 없었다.

검증 항목:

| 케이스 | 위치 | 기대 |
|---|---|---|
| A | 중첩 outer=0 / inner=1, charOffset 0 | `mergeParagraphInCell` 1건 |
| A-대조 | 같은 위치의 `handleDelete` | `getCellParagraphLengthByPath` 호출(안쪽 축) |
| B | 중첩 outer=2 / inner=0, charOffset 0 | 명령 0건, `cellParaIndex:-1` 경로 호출 없음 |
| C | depth 1 (cellParaIndex 1 / 0) | 각각 병합 / 무동작 — 현행 불변 |

### red→green 실증 (실제 캡처)

수정 2줄을 되돌린 devel 원본 상태:

```
✖ 중첩 셀 안쪽 2번째 문단 시작 Backspace 는 이전 문단과 병합한다 (2.8036ms)
✔ 같은 위치의 handleDelete 는 이미 안쪽 축(...ByPath)으로 조회한다(대조군) (0.2021ms)
✖ 중첩 셀 안쪽 첫 문단 시작 Backspace 는 셀 안에서 병합하지 않는다 (0.3833ms)
✔ 비중첩(depth 1) 셀 동작은 불변이다 (0.2454ms)
ℹ tests 4
ℹ pass 2
ℹ fail 2

✖ failing tests:

test at tests\nested-cell-backspace-merge.test.ts:174:1
✖ 중첩 셀 안쪽 2번째 문단 시작 Backspace 는 이전 문단과 병합한다 (2.8036ms)
  AssertionError [ERR_ASSERTION]: flat cellParaIndex(바깥 셀=0)로 판정하면 병합이 통째로 누락돼 Backspace 가 무반응이 된다
  + actual - expected

  + []
  - [
  -   'mergeParagraphInCell'
  - ]

test at tests\nested-cell-backspace-merge.test.ts:189:1
✖ 중첩 셀 안쪽 첫 문단 시작 Backspace 는 셀 안에서 병합하지 않는다 (0.3833ms)
  AssertionError [ERR_ASSERTION]: flat(바깥 셀=2)로 판정하면 병합 대상이 없는데도 병합이 실행된다
  + actual - expected

  + [
  +   'mergeParagraphInCell'
  + ]
  - []
```

수정 복원 후:

```
✔ 중첩 셀 안쪽 2번째 문단 시작 Backspace 는 이전 문단과 병합한다 (1.041ms)
✔ 같은 위치의 handleDelete 는 이미 안쪽 축(...ByPath)으로 조회한다(대조군) (0.1035ms)
✔ 중첩 셀 안쪽 첫 문단 시작 Backspace 는 셀 안에서 병합하지 않는다 (0.0759ms)
✔ 비중첩(depth 1) 셀 동작은 불변이다 (0.0609ms)
ℹ tests 4
ℹ pass 4
ℹ fail 0
```

RED 에서 대조군 2건(A-대조, C)이 통과한 것이 중요하다 — 실패가 축 혼용 때문임을 격리한다.

### CI 게이트

```
$ npm --prefix rhwp-studio run test
ℹ tests 466
ℹ pass 465
ℹ fail 1        ← tests/cell-flow-boundary.test.ts (기존 실패, 아래 참조)

수정 전 baseline: tests 462 / pass 461 / fail 1  (같은 파일)
```

```
$ ./node_modules/.bin/tsc --noEmit
src/core/wasm-bridge.ts(1,44): error TS2307: Cannot find module '@wasm/rhwp.js' ...
src/hwpctl/index.ts(417,57): error TS2307: Cannot find module '@wasm/rhwp.js' ...
```

이 2건은 로컬에 WASM 빌드 산출물(`pkg/`)이 없어서 나는 **기존** 오류다. 변경 전(stash)에도
동일한 2건만 나오는 것을 실측 대조했다 — 본 변경으로 새로 생긴 타입 오류는 0건.

`npm run lint` 는 `rhwp-studio/package.json` 에 존재하지 않는다(스크립트 없음).
Rust 파일을 한 줄도 건드리지 않았으므로 Rust CI 3종(fmt/clippy/test)은 대상 밖이다.

### 미실행 항목 (투명 고지)

- `tests/cell-flow-boundary.test.ts` — devel 원본에서도 실패하는 기존 실패다. 원인은
  `spawnSync(node_modules/.bin/tsc, …)` 가 Windows 에서 `status=null` 을 돌려주는 것
  (`cell-flow test runtime compile failed: undefinedundefined / null !== 0`). 본 변경과
  무관하며 손대지 않았다. 신규 테스트는 같은 함정을 피하려고 `process.execPath` 를 쓴다.
- `npm --prefix rhwp-studio run build`(`tsc && vite build`) 및
  `scripts/frontend-wasm-bindings.test.mjs` — 둘 다 WASM 빌드 산출물(`pkg/`, `@wasm/rhwp.js`)을
  요구하는데 로컬에 없어 실행 불가(`ENOENT … pkg/rhwp.d.ts`). WASM 바인딩은 건드리지 않았다.
- 브라우저 왕복 시각 검증(중첩 표 문서에서 A/B/C 손 확인) 미수행 — 로컬 WASM 빌드가 없어
  studio 를 띄울 수 없었다. 리뷰 단계에서 확인이 필요하면 요청 바란다.

## 5. 잔여 (범위 밖)

- `handleDelete`(:319-320)의 인라인 `useCellPath` 복제를 `cellParaIndexOf` 로 통일하는 정리.
  동작은 이미 옳고, 그쪽 인라인은 `pathJson`/`useCellPath` 를 `...ByPath` 조회에도 쓰므로
  단순 치환이 아니다.
- `cellParagraphPosition`(command.ts:208-224)이 편집 직후 위치의 flat `cellParaIndex` 를
  **안쪽** 값으로 채워, 같은 필드가 "hit-test 직후엔 바깥 / 편집 직후엔 안쪽" 두 의미를 갖는
  구조적 모호성. 이슈 재현 A 가 "다른 곳을 클릭했다가 다시 클릭"을 요구하는 이유이기도 하다.
  축 명시 필드 분리 또는 편집 후 위치 정규화가 근본 해법이나 파급이 커 별건.
- `applyNavResult`(cursor.ts:369-410)가 컨테이너 내부 위치를 재구성하면서 `cellPath` 를
  채우지 않아 중첩 정보가 유실되는 점(세로 이동 계열). 별개 경로.
- Rust `json_usize` 가 음수 입력을 "필드 파싱 실패"로만 알리는 점. Rust 범위라 별건.
