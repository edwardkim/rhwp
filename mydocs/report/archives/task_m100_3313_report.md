# Task #3313 — studio 외부 연결 그림 첫 화면 미표시 (최종 보고서)

`#3313` 1차 정정 기록. #3302(CLI 축)와 같은 발견 계보의 studio(canvas2d) 축.

## 진단 (headless e2e 실측으로 확정)

- 당초 가설 "공급 배선 부재"는 기각 — dev 환경 자동 주입 경로(#741 후속:
  `getExternalImageBasenames` → dev 서버 `/samples/<basename>` fetch(vite 미들웨어) →
  `injectExternalImage`)는 정상 동작(SO-SUEOP 5660 bytes 주입, loaded:true).
- 성능 개선 PR #3272는 미머지 상태라 무관. canvas2d 백엔드도 정상(강제 재렌더 시
  즉시 표시 — 이미지 대역 비백색 0.0%→18.0%).
- **확정 결함**: 비동기 주입이 문서 첫 렌더 **이후** 완료되는데 완료 시점의 뷰 갱신
  이벤트가 없어, 페이지 트리 캐시만 무효화되고 화면은 이전 프레임(그림 없음)에 잔류.

## 수정

- `WasmBridge.onExternalImagesInjected` 훅 신설 — 주입 성공(>0)시에만 호출.
- `main.ts`에서 훅 → `canvasView.loadDocument()` 배선. `document-changed`는 dirty
  마킹(미저장 가드)까지 유발하므로 쓰지 않는다 — 뷰 전용 갱신.

## 검증

- headless e2e(CDP): 강제 재렌더 없이 자동 배선만으로 이미지 대역 0.0% → **18.0%**
  (강제 재렌더 기준치와 동일). tsc 통과, studio 테스트 **636/636**.
- **시각 판정(작업지시자) 통과** — `output/pr3036_judge/studio_p1_fixed.png`.

## 잔여 범위

- 프로덕션 빌드(비-dev 서버) 환경의 사이드카 공급 UX(폴더 열기 / 다중 파일 드롭 →
  `inject_external_image`)는 본 이슈에 남는다. dev 서버 의존 없는 공급 경로 설계 필요.
