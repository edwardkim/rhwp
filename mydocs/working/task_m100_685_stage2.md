# Task #685 Stage 2 단계 보고서 — `input-handler-mouse.ts` 14곳 헬퍼 일괄 치환

- **이슈**: [#685](https://github.com/edwardkim/rhwp/issues/685)
- **수행계획서**: [task_m100_685.md](../plans/task_m100_685.md)
- **구현 계획서**: [task_m100_685_impl.md](../plans/task_m100_685_impl.md)
- **단계 위치**: 3 단계 중 2/3
- **변경 성격**: 본질 정정 (그리드 모드 click 좌표 정합)
- **작성일**: 2026-05-08

---

## 변경 요약

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 14 라인 수정 (insertion 14, deletion 14 — 1:1 표현식 치환) |

총 코드 변경: 14 LOC. 변수 선언 (`pw`, `pageDisplayWidth`) 및 페이지 인덱스 변수 (`pi`/`pageIdx`/`picBbox.pageIndex`) 모두 보존, `(... - X)/2` 표현식만 헬퍼 호출로 교체.

---

## 1. 14곳 라인별 치환 결과

| 라인(전) | 함수 | 페이지 idx 변수 | sc/pw 변수 | 치환 후 |
|---------|------|-----------------|------------|---------|
| 23 | onClick (연결선 모드) | `pi` | `sc` / `pw` | `getPageLeftResolved(pi, sc.clientWidth)` |
| 129 | onClick (선택된 표 이동) | `pi` | `sc` / `pw` | `getPageLeftResolved(pi, sc.clientWidth)` |
| 176 | onClick (다중 그림 선택 BBOX) | `pi` | `sc` / `pw` | `getPageLeftResolved(pi, sc.clientWidth)` |
| 279 | onClick (직선 끝점 핸들) | `picBbox.pageIndex` | `sc` / `pw` | `getPageLeftResolved(picBbox.pageIndex, sc.clientWidth)` |
| 296 | onClick (회전 핸들) | `picBbox.pageIndex` | `sc` / `pw` | `getPageLeftResolved(picBbox.pageIndex, sc.clientWidth)` |
| 357 | onClick (단일 그림 본체) | `pi` | `sc` / `pw` | `getPageLeftResolved(pi, sc.clientWidth)` |
| 431 | onClick (셀 선택 표 리사이즈) | `pageIdx` | `scrollContent` / `pageDisplayWidth` | `getPageLeftResolved(pageIdx, scrollContent.clientWidth)` |
| 475 | onClick (일반 click main path) | `pageIdx` | `scrollContent` / `pageDisplayWidth` | `getPageLeftResolved(pageIdx, scrollContent.clientWidth)` |
| 811 | onDblClick (머리말/꼬리말 진입) | `pageIdx` | `(sc as HTMLElement)` / `pageDisplayWidth` | `getPageLeftResolved(pageIdx, (sc as HTMLElement).clientWidth)` |
| 889 | onContextMenu | `pageIdx` | `scrollContent` / `pageDisplayWidth` | `getPageLeftResolved(pageIdx, scrollContent.clientWidth)` |
| 931 | onMouseMove (연결선 미리보기) | `pi` | `sc` / `pw` | `getPageLeftResolved(pi, sc.clientWidth)` |
| 1146 | onMouseMove (그림 hover) | `pi` | `scrollContent` / `pw` | `getPageLeftResolved(pi, scrollContent.clientWidth)` |
| 1196 | onMouseMove (표 hover) | `pi` | `scrollContent` / `pw` | `getPageLeftResolved(pi, scrollContent.clientWidth)` |
| 1243 | handleResizeHover (표 경계선 hover) | `pageIdx` | `scrollContent` / `pageDisplayWidth` | `getPageLeftResolved(pageIdx, scrollContent.clientWidth)` |

치환 방식: 6 회 Edit 호출 (그룹별 `replace_all` + 컨텍스트 단건 Edit).

```
Edit 1: L279 (line endpoint, picBbox.pageIndex) — 컨텍스트 단건
Edit 2: L296 (rotate, picBbox.pageIndex) — 컨텍스트 단건
Edit 3: L811 (sc as HTMLElement) — 단일 occurrence
Edit 4: replace_all "const pl = (sc.clientWidth - pw) / 2;" (4-space prefix) → 5곳 일괄 (L23, 129, 176, 357, 931 — 모두 pi)
Edit 5: replace_all "const pl = (scrollContent.clientWidth - pw) / 2;" → 2곳 일괄 (L1146, 1196 — 모두 pi)
Edit 6: replace_all "const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;" → 4곳 일괄 (L431, 475, 889, 1243 — 모두 pageIdx; 4-space + 2-space prefix 두 번 호출)
```

각 시점의 `pw`/`pageDisplayWidth` 변수 선언은 보존 (hit test bbox 계산 등 다른 용도 잠재 사용).

---

## 검증 결과

### 1. 잔여 buggy 패턴 sweep

```
$ grep -nE "clientWidth\s*-\s*\w+\)\s*/\s*2" src/engine/input-handler-mouse.ts
(0 건 — exit=1 grep 매칭 없음)
```

### 2. 헬퍼 호출 수 카운트

```
$ grep -c "getPageLeftResolved" src/engine/input-handler-mouse.ts
14
```

→ 정확히 14곳 모두 헬퍼 호출로 교체됨.

### 3. typecheck

```
$ npx tsc --noEmit
(무에러)
```

### 4. vite build

```
$ npx vite build
✓ 85 modules transformed.
✓ built in (단축, 정상)
PWA v1.2.0 — generateSW 정상
```

### 5. e2e 무회귀 — `body-outside-click-fallback.test.mjs --mode=headless`

```
$ exit=0
- ERROR/FAIL/에러: 0 매칭
- 가설 (a) hit invalid: no
- 가설 (b) isTextBox=true: no
- 가설 (c) rect.pageIdx mismatch: no
- scroll 점프: no
```

→ 단일 컬럼 모드에서 click 좌표 산출 무회귀 (Stage 1 의 동치성과 일관 — 헬퍼 sentinel fallback 경로 정상).

---

## 회귀 위험 점검

| 영역 | 위험 | 결과 |
|------|------|------|
| 단일 컬럼 모드 click 좌표 (zoom > 0.5) | 헬퍼 fallback 식이 기존과 다르면 회귀 | OK — body-outside-click-fallback e2e 무회귀 |
| `pw` / `pageDisplayWidth` 변수 잔여 사용처 | 변수 제거 시 hit test bbox 깨짐 | OK — 변수 보존 |
| 페이지 인덱스 변수 mismatch | `pi` vs `picBbox.pageIndex` 잘못 적용 시 좌표 오인 | OK — Edit 단건 컨텍스트로 명시적 분리 (L279/296) |
| L811 의 `sc as HTMLElement` 캐스트 보존 | 캐스트 누락 시 `clientWidth` 접근 타입 에러 | OK — typecheck 통과 |
| Stage 3 그리드 e2e | 본 변경의 본질 검증 (zoom=0.5/0.25 click → cursor) | 다음 단계 |

---

## 다음 단계

Stage 3 — `grid-mode-click-coord.test.mjs` 의 측정 로깅에 회귀 assert 추가 + 호스트 모드 시각 검증. 본 정정의 본질 효과 (그리드 모드 click 좌표 정합) 자동 회귀화.

승인 요청 → 승인 시 Stage 3 진행.
