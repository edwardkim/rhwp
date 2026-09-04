# task m100-2756 — 중첩 표 셀 flat 좌표 축 오용 수정 (studio)

- **이슈**: [#2756](https://github.com/edwardkim/rhwp/issues/2756)
- **브랜치**: `task/m100-2756-nested-cell-axis` (base: `origin/devel`)
- **분류**: 결함 수정 (중첩 셀 좌표 축 혼용 — outer/inner). 결함 2건, 뿌리 1개.
- **Rust 파일 변경 없음** — 전부 `rhwp-studio/` TypeScript.

## 1. 문제

`DocumentPosition` 은 flat 필드(`controlIndex`/`cellIndex`/`cellParaIndex`)와 `cellPath` 배열을 동시에 들고 다닌다. hit-test 는 flat 필드를 `cellPath[0]`, 즉 **최외곽** 셀에서 채운다(`src/document_core/queries/cursor_rect.rs:2462-2491` 의 `let outer = &ctx.path[0];`; `ctx.path[0]` 참조는 이 파일에만 18곳). 따라서 중첩 표(depth ≥ 2)에서 flat 필드는 **바깥 셀의 축**이고, 안쪽 셀 값은 `cellPath[last]` 다.

이 축 전환은 `command.ts` 의 커맨드 계층에서는 `cellParaIndexOf()` 헬퍼로 이미 정리됐으나, 호출부 두 곳이 누락됐다.

- **결함 1 (HIGH, 데이터 손실)** — `cursor.ts:186` `CursorState.comparePositions` 가 flat `cellIndex`/`cellParaIndex` 를 맨몸으로 비교. 중첩 셀에서 선택 양끝이 뒤바뀐다. 소비자 `DeleteSelectionCommand`(`command.ts:555`)는 `cellParaIndexOf`(안쪽 축)로 start/end 를 읽으므로 `startPara > endPara` → `savedTexts=[]`, Rust `delete_range_in_cell_by_path`(`text_editing.rs:3497`)는 `start_para != end_para` 분기를 타 **선택 범위의 여집합**을 삭제. `multiPara=true` + 빈 `savedTexts` 라 `undo` 는 아무것도 복원하지 못한다(영구 손실).
  - **입력 방식 의존**: 좌/우 화살표 선택은 `cursor.ts:435` `updateCellParaInPath` 가 flat 을 안쪽 값으로 덮어써 우연히 정상. 마우스 클릭·드래그 / Shift+상하 화살표만 발현.
- **결함 2 (MEDIUM)** — `input-handler.ts:1822` `getCharPropertiesAtCursor()` 가 `getCellCharPropertiesAt`(flat) 로 **바깥 셀** 서식을 조회. `applyToggleFormat`(`:1789`)이 이 값에서 `!current[prop]` 로 토글 **방향**을 정하고(실제 적용 `ApplyCharFormatCommand` 는 이미 `...ByPath`), `emitCursorFormatState`(`:2028`)의 툴바 표시도 이 조회를 쓴다. 소비자 8곳 전부 오답.

## 2. 검증 근거 (소스·이력)

- `git log -S "comparePositions" -- rhwp-studio/src/engine/cursor.ts` → `f0f7f1a4 Initial commit` 만. 중첩 셀 축으로 한 번도 개정된 적 없음.
- `grep -rn "comparePositions|getSelectionOrdered" rhwp-studio/tests/` → **0건**. 정렬 로직 테스트 부재.
- 기존 두 가드(`delete-selection-nested-cell.test.ts`, `apply-charformat-nested-cell.test.ts`)는 `classBlock`으로 **`command.ts` 커맨드 본문만** 검사 → 결함 2(입력 핸들러)와 start/end 생산자(결함 1)는 시야 밖.

## 3. 변경

| 파일 | 변경 |
|---|---|
| `src/engine/command.ts` | `cellAxisPath(pos)` 헬퍼 신설·export (축 유도 단일 정의 공유; `cellParaIndexOf` 와 동일 규약의 경로 전체 버전). `CellPathEntry` 타입 import 추가. |
| `src/engine/cursor.ts` | `comparePositions` 의 셀-내부 분기를 `cellAxisPath` 기반 깊이별 비교로 교체. `command.ts` 에서 `cellAxisPath` import. |
| `src/engine/input-handler.ts` | `getCharPropertiesAtCursor` 에 `cellPath` 유무 분기 추가 — 있으면 `getCellCharPropertiesAtByPath`(안쪽 축), 없으면 기존 flat 폴백. |

**동작 변화 범위 (수학적 등가 확인)**: `cellPath` 가 없는 두 위치(레거시 flat/`applyNavResult` 산출물)를 비교할 때 `cellAxisPath` 는 flat 필드로 1-depth 경로를 합성하므로, 새 비교 순서(parentParaIndex → controlIndex → cellIndex → cellParaIndex → charOffset)가 **기존 로직과 완전히 동치**다. depth 1 은 `cellPath[0]=최내곽=flat` 이라 무변화. **depth ≥ 2 에서만** 안쪽 축으로 판정이 바뀐다. 결함 2 도 `cellPath` 부재 시 flat 폴백이라 depth 1/레거시 무변화.

축 유도를 인라인 복제하지 않고 헬퍼를 공유한 이유는 `tests/undo-nested-cell-merge-offset.test.ts:44`("인덱스 축 유도가 여러 곳에 복제되면 한쪽만 고쳐지는 회귀가 재발한다")와 선행 PR #2720 의 방침을 따른 것.

## 4. 검증

### 4-1. 신규 테스트

- `tests/selection-ordering-nested-cell.test.ts` — **행위 테스트**. `cursor.ts` 는 `constructor(private wasm)` parameter property 때문에 Node strip-only 로더로 직접 import 불가 → `process.execPath` 자식 프로세스를 `--experimental-transform-types` 로 띄우고 `module.registerHooks` 로 `@/` 별칭·확장자 없는 상대 import 를 매핑(선행 PR #2720 기법). 실제 `CursorState.comparePositions` 호출 + `moveToHit`/`setAnchor`/`getSelectionOrdered` 실경로로 정렬 결과를 검증. 8건.
- `tests/char-properties-cursor-nested-cell.test.ts` — **소스 가드**. `getCharPropertiesAtCursor` 메서드 본문을 추출해 `getCellCharPropertiesAtByPath` 사용·`cellPath` 분기·폴백 순서를 핀. 결함 2 는 조회 결과를 그대로 반환할 뿐 분기 로직이 없어(경로 유무만 판단) "어느 API 를 호출하는가"로 충분. 2건.

행위 red→green 은 결함 1 쪽에서 실경로로 확보했고, 결함 2 는 정적 가드를 선택했다(이유: 위 참조).

### 4-2. red→green 실증 (실제 캡처)

**결함 1** — `cursor.ts` `comparePositions` 를 devel 원본(flat 비교)으로 되돌린 상태:

```
✖ 중첩 안쪽 셀의 문단 0 끝 → 문단 1 시작 선택이 뒤바뀌지 않는다 (데이터 손실 방지)
  AssertionError: flat cellParaIndex(바깥=0) 로 비교하면 두 문단이 같아 보여 charOffset(5>0)으로 낙하, 양끝이 뒤바뀐다
  1 !== -1
✖ 서로 다른 안쪽 셀은 경로의 안쪽 cellIndex 로 정렬한다
  AssertionError: flat cellIndex 는 둘 다 바깥 셀(0)이라 같다 — 안쪽 cellPath[last].cellIndex(2<4)로 정렬해야 한다
  1 !== -1
✔ depth 1 (비중첩) 셀 정렬은 불변이다 (회귀 대조군)
✔ 본문 정렬은 불변이다 (회귀 대조군)
✔ 본문↔셀 혼합 정렬은 불변이다 (회귀 대조군)
✔ comparePositions 는 반대칭·반사적이다
ℹ pass 4 / fail 2
```

RED 에서 **대조군 4건이 통과**한 것이 핵심 — 실패가 중첩 셀 축 혼용 때문임을 격리한다(depth 1·본문·본문↔셀 혼합 무변화 증명).

**결함 2** — `getCharPropertiesAtCursor` 를 devel 원본(flat 전용)으로 되돌린 상태:

```
✖ getCharPropertiesAtCursor 는 cellPath 가 있으면 ...ByPath 로 최내곽 셀을 조회한다
  AssertionError: 중첩 셀 서식 조회는 getCellCharPropertiesAtByPath(안쪽 축)를 써야 한다
✖ getCharPropertiesAtCursor 의 flat 조회는 cellPath 부재 폴백으로만 남는다
  AssertionError: ByPath 분기가 있어야 한다
ℹ pass 0 / fail 2
```

두 수정 복원 후 신규 10건 전부 GREEN.

### 4-3. CI 게이트 (기준선 대조)

| | clean `devel` 기준선 | 수정 후 |
|---|---|---|
| `npm test` | tests 465 / pass 464 / **fail 1** | tests 473 / pass 472 / **fail 1** |
| `npx tsc --noEmit` | error 2 (둘 다 `TS2307 @wasm/rhwp.js`) | error 2 (**baseline 과 byte-identical**) |

- 신규 통과 +8(행위 6 + 가드 2 = 8건), 총계 465→473. 신규 8건 전부 통과. 유일한 실패 `tests/cell-flow-boundary.test.ts` 는 **기준선에도 있던 선재 실패**(`spawnSync(node_modules/.bin/tsc)` 가 Windows 에서 `status=null`)로 본 변경과 무관, 손대지 않음.
- `tsc` 2건은 로컬 WASM 빌드 산출물 부재로 나는 **선재** 오류. `diff baseline after` → **완전 동일**(신규 타입 오류 0).
- `npm run lint` 는 `package.json` 에 없음(스크립트 부재). Rust 무변경으로 Rust CI 3종 대상 밖.

### 4-4. 미실행 항목 (투명 고지)

- 브라우저 왕복 시각 검증(중첩 표 문서에서 손 확인) 미수행 — 로컬에 빌드된 `rhwp_bg.wasm` 이 없어 studio 를 띄울 수 없음. 그래서 결함 1 은 실경로 행위 테스트로 런타임 증명을 대신했다.
- `npm run build`(`tsc && vite build`) — WASM 바인딩(`@wasm/rhwp.js`) 요구로 로컬 실행 불가. 해당 바인딩 무변경.

## 5. 잔여 (범위 밖)

1. `applyNavResult`(`cursor.ts:396-407`)가 컨테이너 내부 위치 재구성 시 `cellPath` 를 싣지 않아 수직 이동 산출물이 중첩 경로를 유실. 이번 수정은 폴백으로 일관성만 확보. 별건.
2. flat 필드 자체의 폐기(`cellPath` 상위 집합) — Rust 직렬화(`cursor_rect.rs` 18곳)와 다수 호출부 연동으로 범위 밖.
3. 셀 내 다중 문단 삭제 undo 의 구조적 한계(`command.ts:603-613` 자체 주석) — 이번 건은 `savedTexts` 가 비는 문제만 제거.
4. Rust `cursor_rect.rs` 의 flat 채움 규칙 — 무수정. TS 소비자가 축을 올바로 고르는 것으로 충분.
