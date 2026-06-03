# Task #689 Stage 1 단계 보고서 — `getPageAtPoint` 헬퍼 추가 + 호출자 분류 확정

- **이슈**: [#689](https://github.com/edwardkim/rhwp/issues/689)
- **수행계획서**: [task_m100_689.md](../plans/task_m100_689.md)
- **구현계획서**: [task_m100_689_impl.md](../plans/task_m100_689_impl.md)
- **단계 위치**: 3 단계 중 1/3
- **변경 성격**: 헬퍼 도입 (동작 변경 없음 — 단일 컬럼은 `getPageAtY` 동치)
- **작성일**: 2026-05-08

---

## 변경 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/view/virtual-scroll.ts` | +33 LOC (`getPageAtPoint(docX, docY)` 메서드 신규) |

총 코드 변경: +33 LOC. 단일 컬럼 모드에서 `getPageAtY` 와 비트 단위 동치 보장.

---

## 1. 헬퍼 추가 (`getPageAtPoint`)

[`virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) 의 `getPageAtY` 메서드 직후에 추가:

```ts
/**
 * 문서 좌표 (X, Y) 가 속하는 페이지 인덱스를 반환한다.
 * 단일 컬럼 모드: getPageAtY 와 동치 (X 무관).
 * 그리드 모드: row(Y) 결정 후 같은 row 안에서 X 가 속하는 페이지 반환.
 *              gap 영역(페이지 사이 빈 공간) click 은 가장 가까운 페이지로 fallback.
 */
getPageAtPoint(docX: number, docY: number): number {
  const rowLastIdx = this.getPageAtY(docY);
  if (!this.gridMode) return rowLastIdx;

  // 같은 row 의 페이지 범위 (rowLastIdx 부터 row 시작까지)
  const rowOffset = this.pageOffsets[rowLastIdx];
  let rowFirst = rowLastIdx;
  while (rowFirst > 0 && this.pageOffsets[rowFirst - 1] === rowOffset) rowFirst--;

  // X 가 페이지 안에 속하는 첫 번째 페이지 반환
  for (let i = rowFirst; i <= rowLastIdx; i++) {
    const left = this.pageLefts[i] ?? 0;
    const right = left + (this.pageWidths[i] ?? 0);
    if (docX >= left && docX <= right) return i;
  }

  // gap / margin 영역 — 가장 가까운 페이지로 fallback
  let bestIdx = rowFirst;
  let bestDist = Infinity;
  for (let i = rowFirst; i <= rowLastIdx; i++) {
    const left = this.pageLefts[i] ?? 0;
    const right = left + (this.pageWidths[i] ?? 0);
    const dist = docX < left ? left - docX : (docX > right ? docX - right : 0);
    if (dist < bestDist) { bestDist = dist; bestIdx = i; }
  }
  return bestIdx;
}
```

기존 `getPageAtY(docY)` 는 그대로 보존 — viewport-center 호출자 (canvas-view.ts L120/L209, input-handler-keyboard.ts L798) + 새 헬퍼 자체 가 사용.

---

## 2. 호출자 분류 결과 (Stage 1.1)

### 마우스 컨텍스트 (Stage 2 치환 대상, 20곳 확정)

| 파일 | 라인 | 함수 / 컨텍스트 |
|------|------|----------------|
| input-handler-mouse.ts | 20, 126, 173, 354, 428, **470**, **807**, **886**, 928, 1143, 1193, **1240** | onClick / onDblClick / onContextMenu / onMouseMove 등 (14곳) |
| input-handler.ts | 612 | 그림 객체 중심 좌표 → 페이지 |
| input-handler.ts | 875 | 표 객체 중심 좌표 → 페이지 |
| input-handler.ts | 972 | e.client 마우스 이벤트 |
| input-handler.ts | 1542 | e.client 마우스 이벤트 |
| input-handler-table.ts | 400 | 표 이동 드래그 (e.client → cy) |
| input-handler-picture.ts | 594 | 그림 이동 드래그 (e.client → cy) |
| **소계** | | **20곳** |

### viewport-center / Y-only 영역 (미수정)

| 파일 | 라인 | 비고 |
|------|------|------|
| canvas-view.ts | 120, 209 | `vpCenter` viewport 중심 — X 의미 없음 |
| input-handler-keyboard.ts | 798 | 키보드 viewport 중심 — X 의미 없음 |

### `coordinate-system.ts:18` 분류 — **dead code**

조사 결과:
```
$ grep -rn "\.documentToPage\b" rhwp-studio/src/
(0 matches)
```

`CoordinateSystem` 인스턴스는 `canvas-view.ts:31` 에서 생성 + `getCoordinateSystem()` getter (L282) 로 노출되지만, **`documentToPage` 메서드를 직접 호출하는 코드 0건** — dead code path. 본 작업 범위 외 (별도 사이클에서 dead code 정리 가능).

→ Stage 2 변경 대상 확정: **20곳** (구현계획서 예상 17~22곳 중간값과 일치).

---

## 3. 검증 결과

### 1. typecheck
```
$ npx tsc --noEmit
(무에러)
```

### 2. vite build
```
$ npx vite build
✓ built in 389ms
PWA v1.2.0 — generateSW 정상
```

### 3. sanity e2e — `body-outside-click-fallback.test.mjs --mode=headless`
```
$ exit=0
- ERROR / FAIL / 에러 / 가설.yes: 0 매칭
```

→ 헬퍼 추가만으로는 동작 변경 없음 (단일 컬럼 모드 동치성 보장).

---

## 4. 회귀 위험 점검

| 영역 | 위험 | 결과 |
|------|------|------|
| 단일 컬럼 모드 (`!gridMode`) | `getPageAtPoint` 가 `getPageAtY` 와 비트 동치 안 되면 회귀 | OK — 명시 분기 `if (!this.gridMode) return rowLastIdx` |
| 그리드 모드 row 경계 인식 | `pageOffsets[i]` 동치 비교가 실제 row 그룹과 일치 안 하면 | OK — virtual-scroll.ts L80 `pageOffsets.push(rowTop)` 동일값 push 검증 완료 |
| gap 영역 fallback | "가장 가까운 페이지" 정책이 기존 동작과 다를 수 있음 | 의도된 변경 — 기존 동작은 항상 row last page 라 잘못된 결과. 새 정책이 사용자 기대에 더 가까움 |
| viewport-center 호출자 4곳 | 헬퍼 추가가 기존 메서드 영향 | OK — `getPageAtY` 미수정 |

---

## 5. 다음 단계

Stage 2 — 마우스 컨텍스트 호출자 20곳 `getPageAtY` → `getPageAtPoint` 치환 + Task #685 sweep 누락 6곳 buggy `pageLeft` 동반 정정 (`getPageLeftResolved` 적용).

승인 요청 → 승인 시 Stage 2 진행.
