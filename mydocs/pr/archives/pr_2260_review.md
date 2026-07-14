# PR #2260 검토 — rhwp-vscode 배율 메뉴 통합 (planet6897, closes #2259)

- 검토일: 2026-07-14 / base: devel / 9파일 +766/−28 (소스 3: provider·viewer·CHANGELOG,
  나머지는 내부 규약 문서) / CI 12 green / BEHIND (merge 시 #2257 방식 적용 가능)
- 요지: 상태 표시줄의 보기 배치+배율을 단일 드롭다운으로 통합
  (폭 맞춤 / 쪽 맞춤 / 두 쪽 맞춤 / % 프리셋), ResizeObserver 맞춤 재계산,
  구현 중 결함 2건(레이아웃 재구성 조기 반환, padding 이중 차감) 동반 수정.

## 구조 검토

- `zoomMode(manual/fitWidth/fitPage)` 분리 + `currentZoom` 은 실제 배율
  유지 — 렌더·썸네일 하위 경로 무수정 (파급 최소화 설계 적절).
- 맞춤 기준 = 문서 전체 최대 폭·높이 (쪽 크기 혼합 문서의 배율 요동 방지)
  + 스크롤바 진동 2중 차단(contentRect 기준 + 1% 히스테리시스) — 설계
  요점이 코드와 일치함을 확인.
- CSP 인라인 핸들러 0건, VSCode 테마 변수 사용, `stb-` 접두어 규약 준수.
- 범위 격리: rhwp-vscode 만. studio/브라우저 확장 비접촉.

## 게이트 (로컬)

| 게이트 | 결과 |
|--------|------|
| `npm ci` + `tsc --noEmit` (extension) | 클린 |
| `npm run compile` (webpack production) | 성공 (webview+ext) |
| 인라인 script/핸들러 스캔 | 0건 |

## 잔여 — 실사용 판정 (컨트리뷰터 명시 요청)

rhwp-vscode 는 자동화 하네스가 없어 시각 검증 미완을 컨트리뷰터가 명시.
확장 개발 호스트(F5)에서 시나리오 7건 확인 필요:
①두 쪽 맞춤 가로 스크롤 없음 ②폭 맞춤 세로 스크롤만 ③쪽 맞춤 한 쪽
전체 표시 ④패널 크기 변화 시 배율 추종 ⑤사이드바 토글 재계산
⑥−/+·Ctrl+휠 시 맞춤 해제 ⑦혼합 쪽 크기 문서 배율 진동 없음.

## 판단

구조·게이트 결격 없음. **실사용 판정(F5) 통과 시 merge 수용 권고.**
merge 시 "closes #2259" 로 이슈 자동 close 됨.

---

## E2E 검증 결과 (v2, 호스트 Chrome CDP — 작업지시자 지시)

매뉴얼 `manual/e2e-cdp.md` 준수 (mirrored 모드 `CHROME_CDP=http://localhost:19222`,
runTest + skipLoadApp). provider webview HTML 추출 + acquireVsCodeApi 스텁
하네스로 standalone 구동 (`rhwp-studio/e2e/pr2260-vscode-zoom-menu.test.mjs`).

| 시나리오 | 1쪽 문서 | 6쪽 문서(biz_plan) |
|----------|---------|-----|
| ①두 쪽 맞춤 가로 스크롤 없음 | PASS (64%) | PASS (63%) |
| ②폭 맞춤 세로 스크롤만 | PASS (126%) | PASS (126%, vScroll ✓) |
| ③쪽 맞춤 한 쪽 뷰포트 이내 | PASS (76%) | PASS (76%) |
| ④리사이즈 추종 | fitWidth 126→78%, fitPage(높이) 76→49% PASS | 동일 PASS |
| ⑥수동 전환 후 배율 고정 | PASS (66% 유지) | PASS |
| ⑦배율 진동 (2초 감시) | PASS (단일값) | PASS |
| 메뉴 UX (체크 표시 + Esc 닫힘) | PASS | PASS |

- ⑤(사이드바 토글)만 F5 전용으로 미자동화 — 단, 사이드바 토글도
  scrollContainer 크기 변화 → 동일 ResizeObserver 경로이므로 ④가 같은
  코드 경로를 커버.
- 검증 중 하네스 측 교정 2건 기록: fitPage 는 세로 문서에서 높이
  제약이므로 폭 변화 불변이 정답(초기 시나리오 설계 오류), − 버튼은
  `#stb-zoom-out` id 직접 사용.
- HTML 보고서: `output/e2e/pr2260-vscode-zoom-menu-report.html`.

## 판단 (갱신)

**E2E 판정 통과 — approve → merge 수용 권고** (BEHIND 시 #2257 방식:
merged tree 선검증 + admin merge).
