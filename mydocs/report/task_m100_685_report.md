# Task #685 최종 결과 보고서

**Issue**: [#685](https://github.com/edwardkim/rhwp/issues/685) — rhwp-studio: zoom ≤ 0.5 그리드 모드 click 좌표 단일 컬럼 가정 — 14곳 분기 일괄 어긋남
**Milestone**: M100 (v1.0.0)
**브랜치**: `local/task685` → `local/devel` 머지 영역 (작업지시자 권한)
**완료일**: 2026-05-08

---

## 1. 본질 요약

[`virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) 는 `zoom ≤ 0.5 + pages > 1 + viewport > 0` 조건에서 **다중 컬럼 그리드 배치** (`pageLefts[i] = marginLeft + col * (pw + gap)`) 로 페이지를 열별 분리 저장하지만, [`input-handler-mouse.ts`](../../rhwp-studio/src/engine/input-handler-mouse.ts) 의 마우스 → 페이지 좌표 변환 14곳 모두 **단일 컬럼 가정 공식** `(scrollContent.clientWidth - pageDisplayWidth) / 2` 사용 → 그리드 모드 click 좌표 ±수백 px 어긋남.

**한컴 호환 결함**: 한컴 오피스 그리드 모드는 정상 동작 (사용자 직접 시연, 2026-05-07). RHWP 만 어긋남.

**Task #595 후속 sweep 으로 발견** — `feedback_process_must_follow` 정합으로 별도 task 사이클 분리 (PR #685/#686 분리 등록 사이클).

## 2. 정정 영역

**Stage 1 — 헬퍼 도입 + verbose 패턴 정리**:
- [`src/view/virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) — `getPageLeftResolved(pageIdx, containerWidth)` 헬퍼 신규 (+12 LOC). 그리드 모드는 `pageLefts[i]`, 단일 컬럼은 `(containerWidth - pageWidth) / 2` fallback (sentinel −1 해소).
- [`src/engine/input-handler.ts`](../../rhwp-studio/src/engine/input-handler.ts) `formBboxToOverlayRect` — 기존 verbose sentinel 패턴 (`getPageLeft(pageIdx) >= 0 ? : ...`) 을 헬퍼 호출 한 줄로 단순화 (-3/+1 LOC). 동치 refactor 로 헬퍼의 두 모드 동작 자연스럽게 검증.
- 기존 `getPageLeft(pageIdx)` raw accessor 는 보존 (`canvas-view.ts`, `field-marker-renderer.ts`, `caret-renderer.ts` 4 호출자 무회귀).

**Stage 2 — `input-handler-mouse.ts` 14곳 헬퍼 일괄 치환** (총 +14/-14 LOC, 1:1 표현식 치환):
- 라인 23, 129, 176, 279, 296, 357, 431, **475**, **811**, **889**, 931, 1146, 1196, 1243 — 모두 `this.virtualScroll.getPageLeftResolved(pageIdx, containerWidth)` 한 줄로 교체.
- 변수 (`pw` / `pageDisplayWidth` / `pi` / `pageIdx` / `picBbox.pageIndex`) 모두 보존 — hit test bbox 등 다른 사용처 잠재 보호.
- 6 회 Edit (`replace_all` 3 회 + 컨텍스트 단건 3 회) — `pi` vs `picBbox.pageIndex` 그룹 분리, 4-space/2-space prefix 분리.

**Stage 3 — e2e 회귀 자동화**:
- [`e2e/grid-mode-click-coord.test.mjs`](../../rhwp-studio/e2e/grid-mode-click-coord.test.mjs) — 측정 로깅에 `assert` 추가 (helpers.mjs 의 `assert` 활용).
  - `dumpGridState`: 모든 페이지 `getPageLeftResolved == 기대값` (max |delta| < 0.01 px).
  - `probeClickAtPage`: CORRECT click → `cursor.pos !== null` (전 케이스), `rectPageIdx === pageIdx` (last-col only — non-last col 은 후속 결함 #689 로 SKIP 안내).
- 추가 probe: zoom=0.25 last col (page 4) + zoom=1.0 baseline (page 0).
- [`mydocs/troubleshootings/grid_mode_click_coord.md`](../troubleshootings/grid_mode_click_coord.md) 끝부분 — "정정 완료 — 부분 정정 (Task #685)" + 후속 결함 #689 안내 추가.

**총 변경**: 4 src 파일 + 1 e2e 파일 + 1 진단노트 + 4 plan/working/report 문서. 코드 변경 ~+30 LOC / -17 LOC.

## 3. 검증 결과 (정량)

### 자동 검증 (모두 PASS)

| 검증 항목 | 결과 |
|-----------|------|
| `npx tsc --noEmit` | ✅ 무에러 |
| `npx vite build` | ✅ 85 modules transformed, 정상 dist 생성 |
| `body-outside-click-fallback.test.mjs --mode=headless` | ✅ exit 0, 단일 컬럼 click 무회귀 (가설 a/b/c 모두 negative) |
| `grid-mode-click-coord.test.mjs --mode=headless` | ✅ exit 0, **PASS=11 / FAIL=0 / SKIP=2** (의도된 non-last col 스킵) |

### Last-col 정합 검증 (자동 회귀)

| 케이스 | helperResolved 동치성 | CORRECT click → rectPageIdx 정합 |
|--------|----------------------|----------------------------------|
| zoom=1.0 page 0 (single col, col=0=last) | ✅ max |delta| = 0.00 px | ✅ rectPageIdx=0 (기대 0) |
| zoom=0.5 page 1 (col=1=last, columns=2) | ✅ max |delta| = 0.00 px | ✅ rectPageIdx=1 (기대 1) |
| zoom=0.25 page 4 (col=4=last, columns=5) | ✅ max |delta| = 0.00 px | ✅ rectPageIdx=4 (기대 4) |

→ Task #685 의 본질 효과 (그리드 모드 last-col 에서 `pageLefts[i]` 적용 → cursor.rectPageIdx 정합) 완전 검증.

### 시각 검증 (작업지시자 직접 확인)

작업지시자가 `samples/hwpctl_action_table_v11.hwp` 로 vite dev server 환경 직접 검증 (2026-05-08):
- non-last col 페이지 (1, 2, 4, 5 등) 클릭 어긋남 — **Issue #689 로 분리 등록된 후속 결함의 시각 재현**.
- last col 페이지 (3, 6, 9 등) 정상 클릭 — Task #685 정정 효과 시각 확인.

## 4. 후속 결함 — Issue #689 분리 등록

Stage 3 의 e2e assert 강화로 추가 결함 노출:

[`virtual-scroll.ts:133-140`](../../rhwp-studio/src/view/virtual-scroll.ts#L133-L140) `getPageAtY(docY)` 가 Y 좌표만 보고 row 의 last page idx 만 반환 → non-last col 페이지 click 시 row 의 last col 페이지로 cursor 처리됨.

| zoom | columns | last col 정합 | non-last col 정합 |
|------|---------|--------------|-------------------|
| 1.0 | 1 | ✅ (col 0 = last) | n/a |
| 0.5 | 2 | ✅ col 1 | ❌ col 0 |
| 0.25 | 5 | ✅ col 4 | ❌ col 0~3 |
| ≤0.5 (3+ columns) | 3+ | ✅ last col | ❌ 그 외 col |

**작업지시자 결정 (2026-05-08, scope 엄격 준수)**: Task #685 본문 명시 범위 (pageLeft 공식 14곳) 그대로 유지. 후속 결함 (`getPageAtY` X 무시) 는 별도 [Issue #689](https://github.com/edwardkim/rhwp/issues/689) 으로 등록 + 즉시 다음 사이클 시작.

#689 의 정정 방향: `getPageAtPoint(docX, docY)` 헬퍼 도입 + input-handler-mouse 14곳의 `getPageAtY(contentY)` 호출을 `getPageAtPoint(contentX, contentY)` 로 일괄 치환.

## 5. 회귀 위험 영역

| 영역 | 위험 | 결과 |
|------|------|------|
| 단일 컬럼 모드 click 좌표 (zoom > 0.5) | 헬퍼 fallback 식이 기존과 다르면 회귀 | ✅ OK — body-outside-click-fallback e2e 무회귀 |
| 그리드 모드 last-col click | pageLefts[i] 적용으로 정합 | ✅ OK — 자동 회귀 assert 통과 |
| `pw` / `pageDisplayWidth` 변수 | hit test bbox 사용처 보호 | ✅ OK — 변수 보존 |
| 페이지 인덱스 변수 mismatch (`pi` vs `picBbox.pageIndex`) | 잘못된 변수 적용 시 좌표 오인 | ✅ OK — Edit 단건 컨텍스트로 명시적 분리 |
| canvas-view / field-marker / caret renderer 4 호출자 | 헬퍼 미사용 영역 | ✅ OK — 본 작업 미수정 |
| `formBboxToOverlayRect` 양식 오버레이 위치 | 헬퍼 동치성 | ✅ OK — body-outside-click-fallback e2e 무회귀로 검증 |

## 6. 정합 영역

- **하이퍼-워터폴 절차 정합**: 수행계획서 → 구현계획서 → Stage 1/2/3 → 최종보고서 모두 승인 게이트 거침. 각 단계 완료 보고서 + 커밋 (`769f534`, `d982d50`, `7fdf01d`).
- **`feedback_process_must_follow` 정합**: Stage 3 에서 추가 결함 발견 시 scope 확장 충동 억제, 작업지시자 결정 받아 #689 로 분리 등록.
- **DRY 정합**: `getPageLeftResolved` 헬퍼 도입 + 기존 input-handler.ts:2579 의 verbose 패턴까지 정리하여 총 15곳 통일.
- **회귀 위험 영역 좁힘**: `canvas-view` / `field-marker-renderer` / `caret-renderer` 4 호출자 미수정 — 그리드 인프라 정상 동작 보존.
- **HWP IR 표준 직접 사용**: virtual-scroll 의 `pageLefts[i]` (이미 그리드 인프라가 채우는 정합 데이터) 를 그대로 적용 — 신규 좌표 계산 코드 0줄.
- **회귀 차단 가드**: e2e `grid-mode-click-coord.test.mjs` assert 강화로 last-col 정합 영구 회귀 차단.
- **부분 정정 명시**: 진단노트 (`grid_mode_click_coord.md`) 끝부분 + 본 결과보고서 모두 "Task #685 = 부분 정정, #689 후속" 명시 → 후속 영역 추적 가능성 확보.

## 7. 변경 파일 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/view/virtual-scroll.ts` | +12 LOC (헬퍼 추가) |
| `rhwp-studio/src/engine/input-handler.ts` | -3/+1 LOC (formBboxToOverlayRect 단순화) |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | +14/-14 LOC (14곳 헬퍼 치환) |
| `rhwp-studio/e2e/grid-mode-click-coord.test.mjs` | +54 LOC (assert + 추가 probe) |
| `mydocs/troubleshootings/grid_mode_click_coord.md` | +20 LOC ("정정 완료 — 부분 정정" + #689 안내) |
| `mydocs/plans/task_m100_685.md` | 신규 (수행계획서) |
| `mydocs/plans/task_m100_685_impl.md` | 신규 (구현계획서) |
| `mydocs/working/task_m100_685_stage{1,2,3}.md` | 신규 (단계 보고서 3종) |
| `mydocs/report/task_m100_685_report.md` | 신규 (본 보고서) |
| `mydocs/orders/20260508.md` | 신규 (5/8 오늘 할일) |

## 8. 커밋 이력 (`local/task685`)

```
7fdf01d Task #685 Stage 3: 그리드 모드 click 좌표 e2e assert 강화 + 후속 결함 #689 분리
d982d50 Task #685 Stage 2: input-handler-mouse 14곳 헬퍼 치환 — 그리드 모드 click 좌표 정정
769f534 Task #685 Stage 1: getPageLeftResolved 헬퍼 추가 + formBboxToOverlayRect 단순화
da7461d Merge local/devel: Task #595 본 작업 + 후속 sweep (Issue #685/#686 진단 노트 + e2e 정량 측정 등록)  ← 분기 기준점
```

## 9. 후속 영역

- **Issue #689 즉시 시작** (작업지시자 결정, 2026-05-08): `getPageAtPoint(docX, docY)` 헬퍼 + 14곳 일괄 치환. Task #685 와 동일한 input-handler-mouse cluster 영역.
- **`local/task685` → `local/devel` 머지** (작업지시자 권한): `git merge local/task685 --no-ff -m "Merge local/devel: Task #685 — 그리드 모드 click 좌표 일괄 정정 (closes #685, partial — #689 후속)"`.
- **이슈 #685 close**: merge 후 `gh issue close 685` 또는 merge 커밋의 `closes #685` 키워드 (cherry-pick 이 아닌 일반 merge 이므로 자동 처리 가능). 단, 부분 정정임을 close 코멘트에 명시 권장.

## 10. 검증된 권위 영역

- **last-col 정합 (자동)**: zoom=1.0 page 0, zoom=0.5 page 1, zoom=0.25 page 4 모두 e2e PASS.
- **단일 컬럼 무회귀 (자동)**: `body-outside-click-fallback.test.mjs` 무회귀.
- **양식 오버레이 무회귀 (sanity)**: `formBboxToOverlayRect` 동치성 → 양식 개체 좌표 영역 무영향.
- **부분 정정 한계 명시 (수동)**: 작업지시자 직접 시각 검증 (`hwpctl_action_table_v11.hwp` 그리드 모드) — non-last col 페이지 어긋남 = #689 시각 재현. last col 페이지 정상 = #685 정정 효과 시각 확인.
