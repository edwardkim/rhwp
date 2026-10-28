---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3377.md
last_verified: 2026-07-30
---

# Task #3377 Stage 1 보고 — 재현 계측

## 수행 내용

1. **트러블슈팅 사전 검색**: 66건 중 첫 로딩/줌/재페인트 선례 0건.
2. **코드 정찰** (`rhwp-studio/src/view/`):
   - 페이지 캔버스의 위치·크기는 `renderCanvas()` 안에서만 설정된다
     (`canvas-view.ts:420~431, 481~484`). `updateVisiblePages()` 는 풀에 있는 페이지를
     재렌더하지 않는다.
   - `onViewportResize()` 는 그리드 전환(`wasGrid || isGrid`)일 때만 전체 재렌더 —
     단일 열에서는 재배치 없이 `updateVisiblePages()` 만 호출 (`:566~574`).
   - 줌 변경은 완료 시 전체 재렌더하나, **애니메이션 중에는 CSS preview 변환만**
     적용(`:618~623`) — 이 창이 닫히지 않으면 stale 캔버스가 남는 구조.
   - SO-SUEOP 는 광폭 페이지 보유로 content 폭(3593.7px) > 뷰포트 → 모든 페이지가
     명시 `left`(중앙 계산값) + 컨테이너 scrollLeft 센터링으로 배치된다. 따라서
     **페이지별 `left`/`width` 계산 시점의 상태 차이가 그대로 지속 어긋남이 된다**
     (CSS 50% 센터링이 아니므로 자기 교정 없음).
3. **확장 프로덕션 빌드 headless 계측** (`rhwp-chrome/dist` 실빌드 + puppeteer-core
   + chrome 146, `viewer.html?url=` 경로, 스크롤·이동 입력 없음):
   - 시계열(0.3s×25): 1·2쪽 x=313 / w=794 완전 정렬 — 이슈의 dev 관측과 동일.
   - 조건 매트릭스 6종: DPR {1, 1.5} × 로딩 중 뷰포트 리사이즈(300~500ms 시점,
     900/1000→1400) × CPU 4배 스로틀 — **전부 MISALIGNED=false**.
   - 하니스: scratchpad `task3377_stage1_timeline.mjs` / `task3377_stage1_matrix.mjs`.

## 판정 — 미재현 (정직 기록)

headless 재현 실패. 이슈의 "dev headless 미재현" 관측과 합쳐, 재현 인자는 실환경
전용 변수로 좁혀진다: ①실창(OS) 리사이즈·DPR 변동(puppeteer setViewport 와 이벤트
경로가 다름) ②GPU 백엔드(headless=SwiftShader vs 실 GPU 의 CanvasKit 선택·fallback
경로) ③확장 실사용 파일 열기 경로(content-script/navigation 인터셉트 — 하니스는
`?url=` 경유) ④줌 애니메이션 중단 창.

## 다음 수 — 실환경 어긋남 상태의 직접 판독 (작업지시자 협조 요청)

작업지시자가 현재 재현 가능하므로, 어긋난 화면에서 DevTools 콘솔 스니펫 1회 실행으로
가설을 즉시 판별할 수 있다. 판독 항목: 1·2쪽 캔버스의 rect(x/w)·style.left·
`rhwpRenderedZoom`·물리 폭, 컨테이너 clientWidth/scrollLeft, content 폭, grid 클래스.

| 판독 결과 | 귀속 |
|---|---|
| p1.rz ≠ p2.rz | 줌 확정 전 렌더 잔존 (H2 — 줌/애니메이션 경합) |
| rz 동일, style.left 상이 | 레이아웃 재계산 후 기렌더 페이지 left 미갱신 (H1 — 배치 stale) |
| left 동일, rect.x 상이 | transform/preview 잔존 (H2 변형) |
| 캔버스 정상, 컨테이너 폭 이상 | 웹뷰 컨테이너 축 문제 (H1 원형) |
