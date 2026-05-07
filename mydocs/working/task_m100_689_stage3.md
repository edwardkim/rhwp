# Task #689 Stage 3 단계 보고서 — e2e strict assert 활성화 + 그리드 모드 모든 col click 정합 자동 회귀화

- **이슈**: [#689](https://github.com/edwardkim/rhwp/issues/689)
- **수행계획서**: [task_m100_689.md](../plans/task_m100_689.md)
- **구현계획서**: [task_m100_689_impl.md](../plans/task_m100_689_impl.md)
- **단계 위치**: 3 단계 중 3/3
- **변경 성격**: e2e 회귀 자동화 (Task #685 + #689 결합 효과 검증)
- **작성일**: 2026-05-08

---

## 변경 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/e2e/grid-mode-click-coord.test.mjs` | SKIP 분기 제거 → 모든 col strict assert (-12/+5 LOC). zoom=0.25 last-col only probe → 5 col 모두 probe (+3 LOC). |
| `mydocs/troubleshootings/grid_mode_click_coord.md` | "완전 정정 완료 (#685 + #689 결합)" 섹션 추가 (+18 LOC) |

총 코드 변경: ~+15 LOC.

---

## 1. e2e 변경 내용

### 1.1 `probeClickAtPage` — SKIP 분기 제거

기존 (Task #685 Stage 3 시점):
```ts
if (probe.isLastCol) {
  assert(afterCorrectClick.rectPageIdx === pageIdx, ...);
} else {
  console.log(`  SKIP: ... — Issue #689 후속`);
}
```

신규 (Task #689 Stage 3):
```ts
// Task #685 + #689 결합 정정 후: 모든 col CORRECT click → 의도한 페이지에 cursor 배치.
assert(
  afterCorrectClick.rectPageIdx === pageIdx,
  `[${label}] CORRECT click → cursor.rectPageIdx=${afterCorrectClick.rectPageIdx} (기대 ${pageIdx}, col=${probe.col}/columns=${probe.columns}${probe.isLastCol ? ' last' : ''})`
);
```

### 1.2 `[3b]` 블록 — zoom=0.25 모든 col probe 확장

기존: `zoom=0.25 last col` (col 4) 만 probe.

신규: `for (let c = 0; c < columns; c++)` 으로 col 0~4 모든 페이지 probe — 5-col 그리드의 모든 위치 정합 자동 회귀 차단.

---

## 2. e2e 검증 결과

### `grid-mode-click-coord.test.mjs --mode=headless`

```
$ exit=0
PASS: 21    FAIL: 0    SKIP: 0
```

세부 항목 (모든 PASS):

| # | 검증 항목 | 결과 |
|---|----------|------|
| 1 | `[zoom=0.5 그리드 상태]` getPageLeftResolved 동치성 (max delta=0.00 px) | PASS |
| 2 | `[zoom=0.25 그리드 상태]` 동치성 | PASS |
| 3-12 | `[page 0~4 (zoom=0.25 col 0~4)]` cursor.pos !== null + rectPageIdx 정합 (10 PASS) | PASS |
| 13 | `[zoom=1.0 단일 컬럼 baseline]` 동치성 | PASS |
| 14-15 | `[page 0 (zoom=1.0 single col)]` cursor.pos + rectPageIdx | PASS |
| 16-17 | `[page 0 (zoom=0.5 col 0)]` cursor.pos + rectPageIdx (이전 SKIP, 현재 PASS) | PASS |
| 18-19 | `[page 1 (zoom=0.5 col 1 last)]` cursor.pos + rectPageIdx | PASS |
| 20-21 | `[page 2 (zoom=0.5 col 0, row 1)]` cursor.pos + rectPageIdx (이전 SKIP, 현재 PASS) | PASS |

→ Task #685 (pageLeft 공식) + Task #689 (`getPageAtPoint`) 결합 효과 자동 회귀화 완성.

### 정량 비교 (Before vs After)

| 케이스 | Task #685 종결 시점 | Task #689 종결 시점 |
|--------|---------------------|---------------------|
| zoom=0.5 col 0 (page 0) | rectPageIdx=1 (어긋남) | rectPageIdx=0 ✅ |
| zoom=0.5 col 1 (page 1) | rectPageIdx=1 ✅ | rectPageIdx=1 ✅ |
| zoom=0.5 col 0 (page 2) | rectPageIdx=3 (어긋남) | rectPageIdx=2 ✅ |
| zoom=0.25 col 0 (page 0) | (미측정) | rectPageIdx=0 ✅ |
| zoom=0.25 col 1 (page 1) | (미측정) | rectPageIdx=1 ✅ |
| zoom=0.25 col 2 (page 2) | (미측정) | rectPageIdx=2 ✅ |
| zoom=0.25 col 3 (page 3) | (미측정) | rectPageIdx=3 ✅ |
| zoom=0.25 col 4 (page 4) | rectPageIdx=4 ✅ | rectPageIdx=4 ✅ |
| zoom=1.0 page 0 | rectPageIdx=0 ✅ | rectPageIdx=0 ✅ (무회귀) |

→ **이전 어긋남 모두 정합화 + 단일 컬럼 baseline 무회귀**.

---

## 3. 진단노트 갱신

[`mydocs/troubleshootings/grid_mode_click_coord.md`](../troubleshootings/grid_mode_click_coord.md) 끝부분에 "완전 정정 완료 (Task #685 + #689 결합)" 섹션 추가:
- `getPageAtPoint` 헬퍼 도입 + 18곳 치환 + 10곳 buggy pageLeft 동반 정정 명시
- 모든 col 정합 표 (zoom=1.0/0.5/0.25)
- e2e 결과 (PASS=21/FAIL=0/SKIP=0) 기록

→ Task #685 의 "부분 정정" 기록과 #689 의 "완전 정정" 기록이 진단노트 안에서 시간 순으로 명확히 분리됨.

---

## 4. 시각 검증 (작업지시자 권한)

자동 e2e 가 PASS=21 로 모든 col 정합 검증을 완료했으므로, 시각 검증은 작업지시자가 환경에서 1회 직접 확인 권장:
- `samples/hwpctl_action_table_v11.hwp` 그리드 모드 (zoom=0.5, 0.25)
- 페이지 1, 2, 4, 5 (이전 어긋남) 클릭 → 캐럿이 클릭한 페이지 안에 정확 배치
- zoom=1.0 일반 클릭 무회귀

---

## 5. 회귀 위험 점검

| 영역 | 위험 | 결과 |
|------|------|------|
| 기존 last-col 케이스 (Task #685 의 PASS 영역) | strict assert 활성화로 영향 | OK — 모두 PASS 유지 (col=last 이고 helperResolved 동치 보장) |
| zoom=1.0 단일 컬럼 baseline | `getPageAtPoint` Y-only fallback 정합 | OK — 명시 분기 + body-outside-click-fallback 무회귀 (Stage 1/2 검증) |
| 새 probe 추가로 e2e 시간 증가 | zoom=0.25 5-col probe | 무시할 수 있는 수준 (각 probe ~1초) |
| dumpGridState helperResolved assert | 임계값 |delta| < 0.01 px | OK — max delta=0.00 px 모든 모드 |

---

## 6. 다음 단계

최종 결과보고서 작성 (`mydocs/report/task_m100_689_report.md`) + `mydocs/orders/20260508.md` 갱신 → 승인 후 #689 close + `local/devel` 머지.

승인 요청 → 승인 시 최종 단계 진행.
