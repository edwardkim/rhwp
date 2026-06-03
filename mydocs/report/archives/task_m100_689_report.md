# Task #689 최종 결과 보고서

**Issue**: [#689](https://github.com/edwardkim/rhwp/issues/689) — rhwp-studio 그리드 모드 `getPageAtY` X 좌표 무시 — non-last col 페이지 click 어긋남
**Milestone**: M100 (v1.0.0)
**브랜치**: `local/task689` → `local/devel` 머저 영역 (작업지시자 권한)
**선행 타스크**: [#685](https://github.com/edwardkim/rhwp/issues/685) (closed, partial — pageLeft 공식 14곳 정정)
**완료일**: 2026-05-08

---

## 1. 본질 요약

[`virtual-scroll.ts:133-140 getPageAtY(docY)`](../../rhwp-studio/src/view/virtual-scroll.ts#L133-L140) 가 Y 좌표만 보고 페이지를 결정. 그리드 모드에서 같은 row 의 모든 페이지가 동일 `pageOffsets[i] = rowTop` 을 가지므로 loop 가 highest index 부터 내려가며 첫 매치 반환 → **항상 row 의 last col 페이지** 만 반환.

→ non-last col 페이지(좌측/중간 컬럼) click 시 row 의 last col 페이지로 cursor 처리. 사용자 직접 시각 확인 (`hwpctl_action_table_v11.hwp` 그리드 모드, 2026-05-08): "1, 2 페이지가 클릭이 안 됨".

**Task #685 와의 관계**: #685 는 `pageLeft` 공식 정정으로 **last col 정합** 만 달성. non-last col 정정은 별도 결함 영역 → 본 #689 에서 정정.

## 2. 정정 영역

### 2.1 Stage 1 — `getPageAtPoint` 헬퍼 도입

[`virtual-scroll.ts`](../../rhwp-studio/src/view/virtual-scroll.ts) 에 `getPageAtPoint(docX, docY)` 신규 (+33 LOC):

- 단일 컬럼 모드: `getPageAtY(docY)` 동치 (X 무관).
- 그리드 모드: `getPageAtY` 로 row 의 last page idx 찾고, 같은 row 의 페이지 범위 (`pageOffsets[i] === rowOffset` 조건) 안에서 X 가 속하는 페이지 반환.
- Gap 영역 (페이지 사이 빈 공간) click → 가장 가까운 페이지로 fallback.

기존 `getPageAtY` 는 미수정 — viewport-center 호출자 (canvas-view 2곳, input-handler-keyboard 1곳) + 새 헬퍼 자체가 사용.

### 2.2 Stage 2 — 마우스 컨텍스트 일괄 치환

**`getPageAtY` → `getPageAtPoint` 치환 (18곳)**:
- `input-handler-mouse.ts` 12곳 (3 회 `replace_all`: cy/y/contentY)
- `input-handler.ts` 4곳 (L612/875/972/1542, 2 회 `replace_all`: cY/contentY)
- `input-handler-table.ts` 1곳 (L400)
- `input-handler-picture.ts` 1곳 (L594)

**Task #685 sweep 누락 buggy `pageLeft` 동반 정정 (10곳)** — 작업지시자 scope 확장 결정 (2026-05-08):

| 분류 | 파일 / 라인 | pageIdx 출처 |
|------|-------------|--------------|
| `getPageAtY` 와 동일 함수 (6곳) | input-handler.ts L612/875/972/1542 + table:400 + picture:594 | `getPageAtPoint(...)` (Stage 2 에서 치환) |
| `getPageAtY` 호출 없음 (4곳, 추가 발견) | input-handler-table.ts L74/L111 + input-handler-connector.ts L85/L152 | `state.edge.pageIndex` 또는 함수 매개변수 |

후자 4곳은 본 작업 sweep 중 추가 발견 → 작업지시자 추가 승인으로 동반 정정. 그리드 모드에서 표 리사이즈 드래그 / connector 좌표 어긋남도 함께 해소.

### 2.3 Stage 3 — e2e strict assert 활성화

[`grid-mode-click-coord.test.mjs`](../../rhwp-studio/e2e/grid-mode-click-coord.test.mjs):
- `probeClickAtPage` 의 `isLastCol` 분기 + SKIP 로깅 제거 → 모든 col strict assert 일원화.
- `[3b]` zoom=0.25 last-col only probe → for 루프로 col 0~4 5건 모두 probe.
- 진단노트 [`grid_mode_click_coord.md`](../troubleshootings/grid_mode_click_coord.md) 끝부분에 "완전 정정 완료 (Task #685 + #689 결합)" 섹션 추가.

### 2.4 코드 품질 결정 (작업지시자 review, 2026-05-08)

`getPageLeftResolved(pageIdx, containerWidth)` API 의 `containerWidth` 인자 leaky abstraction 검토 — Option A (현재, stateless) vs Option B (`setPageDimensions` 시점 단일 컬럼도 실좌표 저장, 호출자 단순) 비교.

**결정**: **Option A 유지**. 이유:
- A 는 stateless — click 시점 항상 최신 `clientWidth` 사용
- B 는 stateful — pageLefts[i] cache 가 setPageDimensions 호출에 의존, race condition / 미래 코드 변경 시 fragile
- verbose 는 trade-off 비용, robustness 가 우선

→ 본 작업은 추가 리팩터 없이 #689 종결.

### 총 변경

- 신규 헬퍼 1 (`getPageAtPoint`, +33 LOC)
- 호출 치환: **`getPageAtY` 18곳** + **buggy `pageLeft` 10곳** = 28 LOC delta
- e2e: ~+15 LOC (SKIP→strict, 추가 probe)
- 진단노트: +18 LOC ("완전 정정 완료" 섹션)
- 5 src 파일 + 1 e2e + 1 진단노트 + 4 plan/working/report 문서

## 3. 검증 결과 (정량)

### 자동 검증 (모두 PASS)

| 검증 항목 | 결과 |
|-----------|------|
| `npx tsc --noEmit` | ✅ 무에러 |
| `npx vite build` | ✅ 성공 |
| `body-outside-click-fallback.test.mjs --mode=headless` | ✅ exit 0, 단일 컬럼 무회귀 |
| `grid-mode-click-coord.test.mjs --mode=headless` | ✅ **PASS=21 / FAIL=0 / SKIP=0** |

### Before vs After 비교

| 케이스 | #685 종결 시점 | #689 종결 시점 |
|--------|---------------|---------------|
| zoom=1.0 page 0 | rectPageIdx=0 ✅ | rectPageIdx=0 ✅ (무회귀) |
| zoom=0.5 col 0 (page 0) | rectPageIdx=1 ❌ | **rectPageIdx=0 ✅** |
| zoom=0.5 col 1 last (page 1) | rectPageIdx=1 ✅ | rectPageIdx=1 ✅ |
| zoom=0.5 col 0 (page 2) | rectPageIdx=3 ❌ | **rectPageIdx=2 ✅** |
| zoom=0.25 col 0 (page 0) | (미측정/SKIP) | **rectPageIdx=0 ✅** |
| zoom=0.25 col 1 (page 1) | (미측정/SKIP) | **rectPageIdx=1 ✅** |
| zoom=0.25 col 2 mid (page 2) | (미측정/SKIP) | **rectPageIdx=2 ✅** |
| zoom=0.25 col 3 (page 3) | (미측정/SKIP) | **rectPageIdx=3 ✅** |
| zoom=0.25 col 4 last (page 4) | rectPageIdx=4 ✅ | rectPageIdx=4 ✅ |

→ **이전 어긋남 모두 정합화** + 단일 컬럼 baseline 무회귀.

### 시각 검증 (작업지시자, 2026-05-08)

`hwpctl_action_table_v11.hwp` 그리드 모드 (zoom=0.5, columns=3) — 모든 col 페이지 정상 클릭 확인:
- 작업지시자 평가: "오 정상적으로 잘 된다."

→ 이전 단계 (#685 종결 시점) "1, 2 페이지가 클릭이 안 됨" 보고에서 모든 페이지 click 정합 달성.

## 4. 회귀 위험 영역

| 영역 | 위험 | 결과 |
|------|------|------|
| 단일 컬럼 모드 click (zoom > 0.5) | `getPageAtPoint` 가 `getPageAtY` 동치 안 되면 회귀 | ✅ OK — 명시 분기 + body-outside-click-fallback 무회귀 |
| 그리드 모드 last-col click | #685 정합 영역 무회귀 | ✅ OK — Stage 3 e2e PASS (zoom=0.5 col 1, zoom=0.25 col 4) |
| 그리드 모드 non-last col click | 본 작업의 본질 효과 | ✅ OK — Stage 3 e2e PASS (zoom=0.5 col 0/0row1, zoom=0.25 col 0~3) |
| `pw` / `pageDisplayWidth` 변수 보존 | hit test bbox 등 다른 사용처 | ✅ OK — 모두 보존 |
| 추가 4 사이트 (table L74/111, connector L85/152) | 신뢰 pageIdx 인 곳에 헬퍼 적용 | ✅ OK — `getPageLeftResolved` 가 신뢰값 받아도 정상 |
| viewport-center 영역 (canvas-view, keyboard) | 헬퍼 통일 욕심 시 회귀 | ✅ OK — 미수정 |
| `coordinate-system.ts:18` | dead code 점검 | ✅ OK — 미수정 (호출자 0건) |
| Container resize 처리 | Option A stateless 설계 | ✅ OK — click 시점 `clientWidth` 직접 사용으로 robust |

## 5. 정합 영역

- **하이퍼-워터폴 절차 정합**: 수행계획서 → 구현계획서 → Stage 1/2/3 → 최종보고서 모두 작업지시자 승인 게이트 거침. 단계별 보고서 (`_stage{N}.md`) + 단계별 커밋 (`a22cbf0`, `8369ebd`, `5db03a8`).
- **`feedback_process_must_follow` 정합**: Stage 2 sweep 중 추가 buggy pageLeft 4곳 발견 시 즉시 작업지시자 승인 받아 scope 확장. 무단 확장 없음.
- **DRY 정합**: `getPageLeftResolved` 헬퍼는 #685 도입분 재사용. 신규 헬퍼는 `getPageAtPoint` 1개. 신규 좌표 계산 코드 0줄 (기존 데이터 `pageOffsets`/`pageLefts`/`pageWidths` 재사용).
- **HWP IR 표준 직접 사용**: virtualScroll 의 그리드 인프라 (이미 채워져 있는 `pageOffsets`, `pageLefts`, `pageWidths`) 그대로 적용.
- **회귀 위험 영역 좁힘**: viewport-center 호출자 (canvas-view, input-handler-keyboard) 미수정. coordinate-system.ts dead code 미수정.
- **회귀 차단 가드 영구 보존**: e2e strict assert 활성화로 모든 col 정합 자동 회귀 차단.
- **#685 누락분 동반 정정**: 같은 영역 (input-handler 클러스터 buggy pageLeft) 의 누락분을 본 작업에서 함께 정정 → 향후 같은 결함 카테고리에 대한 추가 후속 이슈 불필요.
- **사용자 보고 정합**: 작업지시자 시각 검증 시점에서 정확히 보고된 결함 (`hwpctl_action_table_v11.hwp` 그리드 모드 페이지 1, 2 클릭 안 됨) 이 #689 의 본질 결함 (`getPageAtY` X 무시) 임을 진단 노트로 명시 + 정정 후 작업지시자 직접 정합 확인.
- **코드 품질 결정 명시**: Option A vs B trade-off 검토 + 작업지시자 robustness 우선 결정 → 결과보고서 기록으로 향후 동일 결정 추적 가능.

## 6. 변경 파일 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/view/virtual-scroll.ts` | +33 LOC (`getPageAtPoint` 헬퍼 추가) |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 12곳 치환 (`getPageAtY` → `getPageAtPoint`) |
| `rhwp-studio/src/engine/input-handler.ts` | 4곳 `getPageAtY` + 4곳 buggy `pageLeft` 치환 |
| `rhwp-studio/src/engine/input-handler-table.ts` | 1곳 `getPageAtY` + 3곳 buggy `pageLeft` (L400 + L74/L111) |
| `rhwp-studio/src/engine/input-handler-picture.ts` | 1곳 `getPageAtY` + 1곳 buggy `pageLeft` |
| `rhwp-studio/src/engine/input-handler-connector.ts` | 2곳 buggy `pageLeft` (L85/L152) |
| `rhwp-studio/e2e/grid-mode-click-coord.test.mjs` | SKIP→strict + 추가 probe |
| `mydocs/troubleshootings/grid_mode_click_coord.md` | "완전 정정 완료 (#685 + #689 결합)" +18 LOC |
| `mydocs/plans/task_m100_689.md` | 신규 (수행계획서) |
| `mydocs/plans/task_m100_689_impl.md` | 신규 (구현계획서) |
| `mydocs/working/task_m100_689_stage{1,2,3}.md` | 신규 (단계 보고서 3종) |
| `mydocs/report/task_m100_689_report.md` | 신규 (본 보고서) |
| `mydocs/orders/20260508.md` | 갱신 (#689 [완료] + 추가 발견 4곳 메모) |

## 7. 커밋 이력 (`local/task689`)

```
5db03a8 Task #689 Stage 3: e2e strict assert 활성화 + 그리드 모드 모든 col click 정합 자동 회귀화
8369ebd Task #689 Stage 2: getPageAtY 18곳 → getPageAtPoint 치환 + Task #685 누락 buggy pageLeft 10곳 동반 정정
a22cbf0 Task #689 Stage 1: getPageAtPoint 헬퍼 도입 + 호출자 분류 확정
7651d91 Merge local/devel: Task #685 — 그리드 모드 click 좌표 일괄 정정 (closes #685, partial — #689 후속)  ← 분기 기준점
```

## 8. 후속 영역

- **Task #685 + #689 결합으로 그리드 모드 click 한컴 호환 완성**. 추가 후속 이슈 없음.
- 키보드/IME/Touch 입력 경로의 그리드 모드 좌표 처리 (별도 후속 조사) — 본 작업 범위 외.
- `coordinate-system.ts` dead code 정리 (`documentToPage` 호출자 0건) — 별도 cleanup 사이클.
- 헬퍼 이름 단축 (`getPageLeftResolved` → 짧은 이름) 등 추가 리팩터 — 코드 품질 사이클 후보.

## 9. 검증된 권위 영역

- **그리드 모드 모든 col click 정합 (자동)**: zoom=0.5 col 0/1, zoom=0.25 col 0~4 모두 e2e PASS.
- **단일 컬럼 무회귀 (자동)**: `body-outside-click-fallback.test.mjs` 무회귀 + zoom=1.0 baseline strict assert PASS.
- **양식 오버레이 무회귀 (sanity)**: `formBboxToOverlayRect` 동치성 (#685 stage 1) + 본 작업에서 `pageLeft` 헬퍼 추가 사용처 확장 무회귀.
- **사용자 시각 검증 (수동)**: `hwpctl_action_table_v11.hwp` 그리드 모드 모든 col 정상 클릭 — 작업지시자 직접 확인.
