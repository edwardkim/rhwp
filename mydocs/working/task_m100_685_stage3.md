# Task #685 Stage 3 단계 보고서 — e2e 회귀 assert 강화 + 후속 결함 분리 등록

- **이슈**: [#685](https://github.com/edwardkim/rhwp/issues/685)
- **수행계획서**: [task_m100_685.md](../plans/task_m100_685.md)
- **구현 계획서**: [task_m100_685_impl.md](../plans/task_m100_685_impl.md)
- **단계 위치**: 3 단계 중 3/3
- **변경 성격**: e2e 회귀 자동화 + scope 발견사항 분리
- **작성일**: 2026-05-08

---

## 변경 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/e2e/grid-mode-click-coord.test.mjs` | +30 LOC: import assert, dumpGridState 에 helperResolved 동치성 assert, probeClickAtPage 에 last-col only 정합 assert, zoom=0.25 last-col probe + zoom=1.0 baseline probe 추가 |
| `mydocs/troubleshootings/grid_mode_click_coord.md` | +20 LOC: "정정 완료 — 부분 정정" + 후속 결함 (#689) 안내 |

---

## 1. e2e assert 추가

### dumpGridState — 헬퍼 동치성 검증

각 페이지마다 `getPageLeftResolved(i, clientWidth)` 호출값을 측정하고, 기대값(`pageLefts[i] ?? buggy`)과 비교. 모든 페이지에서 |delta| < 0.01 px 임을 assert.

```js
const expectedHelper = correct >= 0 ? correct : buggy;
const helperDelta = helperResolved - expectedHelper;
// ...
assert(maxHelperDelta < 0.01, `[${label}] getPageLeftResolved == 기대값 ...`);
```

### probeClickAtPage — last-col only 정합 검증

CORRECT click @(correctDocX, correctDocY) → `cursor.rectPageIdx === pageIdx` 를 assert. 단, **last-col 케이스에서만 strict** 하게 검증. non-last col 은 후속 결함 (Issue #689) 으로 항상 row 의 last page 로 떨어지므로 SKIP 로깅.

```js
if (probe.isLastCol) {
  assert(afterCorrectClick.rectPageIdx === pageIdx, `[${label}] CORRECT click → cursor.rectPageIdx=...`);
} else {
  console.log(`  SKIP: [${label}] non-last col rectPageIdx strict assert ... — Issue #689 후속`);
}
```

### 추가 probe

- **`[3b] zoom=0.25 last col (page 4)`** — 5-column 그리드의 마지막 컬럼 click 정합 검증
- **`[4b] zoom=1.0 page 0`** — 단일 컬럼 baseline click 무회귀 검증

---

## 2. 검증 결과

### e2e 실행 — `grid-mode-click-coord.test.mjs --mode=headless`

```
$ node e2e/grid-mode-click-coord.test.mjs --mode=headless
exit=0

PASS: 11    FAIL: 0    SKIP: 2 (의도된 non-last col)
```

세부 항목:

| 검증 항목 | 결과 |
|-----------|------|
| `[zoom=0.5 그리드 상태]` getPageLeftResolved == 기대값 (max delta=0.00 px) | PASS |
| `[zoom=0.25 그리드 상태]` getPageLeftResolved == 기대값 (max delta=0.00 px) | PASS |
| `[zoom=1.0 단일 컬럼 baseline]` getPageLeftResolved == 기대값 (max delta=0.00 px) | PASS |
| `[page 4 (zoom=0.25 last col)]` cursor.pos !== null + rectPageIdx=4 | PASS |
| `[page 0 (zoom=1.0 single col)]` cursor.pos !== null + rectPageIdx=0 | PASS |
| `[page 0 (col 0)]` cursor.pos !== null | PASS |
| `[page 0 (col 0)]` rectPageIdx strict (col=0/columns=2) | SKIP (#689) |
| `[page 1 (col 1)]` cursor.pos !== null + rectPageIdx=1 | PASS |
| `[page 2 (col=2 % columns)]` cursor.pos !== null | PASS |
| `[page 2 (col 0/columns=2)]` rectPageIdx strict | SKIP (#689) |

→ Task #685 의 본질 효과 (last col 에서 `pageLefts[i]` 적용 → cursor.rectPageIdx 정합) 완전 검증.

---

## 3. Stage 3 진행 중 발견된 후속 결함 — Issue #689 등록

본 Stage 의 assert 강화로 새로운 결함이 노출됨:

[`virtual-scroll.ts:133-140 getPageAtY`](../../rhwp-studio/src/view/virtual-scroll.ts#L133-L140) 는 Y 좌표만 보고 페이지 인덱스를 결정. 그리드 모드에서 한 row 의 모든 페이지가 동일한 `pageOffsets[i] = rowTop` 을 가지므로 **항상 row 의 last page idx 만 반환**.

| 의도 페이지 | col | rectPageIdx 결과 | 정합 여부 |
|------------|-----|------------------|----------|
| 0 (zoom=0.5) | 0 | 1 | ❌ |
| 1 (zoom=0.5) | 1 (last) | 1 | ✅ |
| 2 (zoom=0.5) | 0 | 3 | ❌ |
| 4 (zoom=0.25) | 4 (last) | 4 | ✅ |

→ **Task #685 의 pageLeft 정정만으로는 last-col 케이스만 해결됨**. non-last col 정정은 별도 결함 영역 (`getPageAtY` 의 X 무시) 이므로 [Issue #689](https://github.com/edwardkim/rhwp/issues/689) 으로 분리 등록.

### 작업지시자 결정 (2026-05-08)

scope 엄격 준수 — Task #685 의 정정 범위는 본문 명시 그대로 (pageLeft 공식 14곳) 유지. 후속 결함 (#689) 은 별도 타스크에서 진행.

---

## 4. 시각 검증 (작업지시자 검토 단계로 위임)

`--mode=host` (호스트 Chrome CDP) 로 시각 검증 1회는 작업지시자가 환경에서 직접 실행하시기를 권장. 본 단계의 자동 검증은 headless 모드로 PASS 확인 완료.

---

## 5. 완료 기준 점검

- [x] dumpGridState 헬퍼 동치성 assert 추가 (3 모드 × 페이지수 만큼 검증)
- [x] probeClickAtPage 정합 assert 추가 (last-col strict, non-last skip with 안내)
- [x] zoom=0.25 last col probe 추가
- [x] zoom=1.0 baseline probe 추가
- [x] e2e headless PASS=11 / FAIL=0 / SKIP=2 (의도)
- [x] 진단노트 (`grid_mode_click_coord.md`) "정정 완료 — 부분 정정" + #689 안내 추가
- [x] Issue #689 등록 ([https://github.com/edwardkim/rhwp/issues/689](https://github.com/edwardkim/rhwp/issues/689))

---

## 다음 단계

최종 결과보고서 (`mydocs/report/task_m100_685_report.md`) 작성 + `mydocs/orders/2026-05-07.md` (또는 작업 시작일) 갱신 → 승인 후 이슈 #685 close 절차.

승인 요청 → 승인 시 최종 단계 진행.
