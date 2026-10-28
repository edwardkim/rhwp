# Task m100-2914 처리결과 — 표 셀 안 Ctrl+↑ 가 현재 문단 시작을 건너뜀

- 이슈: https://github.com/edwardkim/rhwp/issues/2914
- 브랜치: `task/m100-2914-cell-ctrl-up-para-start` (origin/devel 기준)
- 범위: `rhwp-studio/src/engine/cursor.ts` TypeScript 한정, cargo 빌드 없음

## 1. 배경 및 탐색 경로

당초 목표는 `cursor.ts` 의 잔여 flat-axis 독자(`getLineInfoForOffset`, `getCursorRectOnLine`)를
경로 기반 API 로 전환하는 것이었다. 그러나 현 devel(a4d0486e) 기준으로 확인한 결과:

```
git grep -n "line_info_in_cell_by_path\|LineInfoInCellByPath" origin/devel -- src rhwp-studio
→ (결과 없음)
```

Rust 측 `line_info_in_cell_by_path` / `cursor_rect_on_line_by_path` 계열 API 는 여전히
존재하지 않아 해당 전환은 TS 단독으로 불가능하다(#2792 의 "경로 기반 선택 rect·줄정보
코어 부재" 지적과 일치). 따라서 `cursor.ts` 안의 다른 독립적·증명 가능한 결함을 탐색했고,
`moveToParagraphBoundary` 의 본문/셀 분기 비대칭을 확정했다.

## 2. 결함 분석

### 2.1 함수 계약

`moveToParagraphBoundary(direction: -1 | 1)` 은 Ctrl+↑/↓ 문단 단위 이동이며, doc 주석이
"한컴 표준 정합" 을 명시한다. 한컴 표준에서 Ctrl+↑ 는:

1. 커서가 문단 **중간**이면 → 현재 문단 시작으로.
2. 커서가 문단 **시작**이면 → 이전 문단 시작으로.

### 2.2 본문 분기 (정상)

```ts
} else {
  if (pos.charOffset > 0) {
    // 현재 문단 시작으로 (한컴 표준)
    this.position = { ...pos, charOffset: 0 };
  } else if (pos.paragraphIndex > 0) {
    this.position = { ...pos, paragraphIndex: pos.paragraphIndex - 1, charOffset: 0 };
  } else if (sec > 0) { ... }
}
```

`charOffset > 0` 를 최우선 검사하여 규칙 1 을 구현한다.

### 2.3 셀 분기 (결함)

```ts
const target = cpi + direction;
if (target >= 0 && target < cellParaCount) {
  this.position = { ...pos, paragraphIndex: target, cellParaIndex: target, ..., charOffset: 0 };
  this.updateRect();
}
```

`charOffset` 을 전혀 보지 않고 곧바로 `cpi + direction` 으로 셀 문단 축만 이동한다.
그 결과 두 가지 오동작이 생긴다:

| 상황 | 기대 (본문·한컴) | 실제 |
|---|---|---|
| cpi > 0, charOffset > 0, Ctrl+↑ | 현재 문단 시작 정지 | 이전 셀 문단 시작으로 점프 (한 문단 건너뜀) |
| cpi == 0, charOffset > 0, Ctrl+↑ | 현재 문단 시작 정지 | `target = -1` 가드에 걸려 **완전 무반응** |

### 2.4 재현

2×2 표의 한 셀에 "가나다" / "라마바" 두 문단 입력 →
"라마바" 중간에서 Ctrl+↑ → "가나다" 시작으로 점프(건너뜀).
"가나다" 중간에서 Ctrl+↑ → 무반응. 같은 조작을 본문 문단에서 하면 둘 다 현재 문단
시작에 멈춘다.

## 3. 수정

셀 분기 진입 직후, 본문 분기와 동일한 가드를 추가했다 (+6줄, 순수 위치 갱신):

```ts
// [#2914] 본문 분기와 동일한 한컴 표준 — 문단 중간(Ctrl+↑)이면 먼저 현재 문단 시작에서 멈춘다.
if (direction === -1 && pos.charOffset > 0) {
  this.position = { ...pos, charOffset: 0 };
  this.updateRect();
  return;
}
```

설계 근거:

- `...pos` 스프레드라 `cellPath`/`cellParaIndex`/`parentParaIndex` 등 셀 좌표 축이 그대로
  보존되고 `charOffset` 만 0 이 된다. WASM 조회가 필요 없어 중첩 표(cellPath 존재)에서도
  안전하며, #2756 이 지적한 flat 축 오용 문제를 새로 만들지 않는다.
- 가드가 `getCellParagraphCount*` WASM 호출보다 앞서므로 cpi==0 무반응 케이스도 같은
  가드 하나로 해소된다.
- Ctrl+↓(direction=+1) 및 문단 시작에서의 cpi±1 이동 경로는 변경 없음.

## 4. 검증

- 신규 테스트 `rhwp-studio/tests/cell-ctrl-up-para-start.test.ts` (source-guard 방식,
  기존 `table-keyboard-navigation.test.ts` 관례 준수):
  - 가드 패턴 존재 + WASM 카운트 조회보다 선행함을 검사.
  - **red→green 확인**: origin/devel 원본 cursor.ts 로 되돌리면 fail 1, 수정본이면 pass 1.
- `npm test`: **pass 499 / fail 1** — 실패는 기존 허용된 `cell-flow-boundary.test.ts` 뿐.
- `npx tsc --noEmit`: 기존 baseline 과 동일한 TS2307 2건(`@wasm/rhwp.js`)만, 신규 0건.

## 5. 남은 문제

- `getLineInfoForOffset` / `getCursorRectOnLine` 의 flat-axis 사용은 Rust 경로 기반
  줄정보/rect API 가 추가되기 전까지 전환 불가 (#2792 추적).
- 셀 분기의 Ctrl+↑ cpi==0·charOffset==0 케이스(셀 첫 문단 시작에서 표 밖/이전 셀로의
  탈출)는 본 수정 범위 밖이며 현행 동작(무이동)을 유지했다.
