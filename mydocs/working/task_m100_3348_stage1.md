# Task #3348 Stage 1 — 외부 연결 그림 dev 전용 fetch 가드 (수행계획서)

## 배경

v0.8.0 배포 점검(2026-07-26)에서 크롬 확장 Errors 패널에
`[WasmBridge] 외부 image ...: 00000000.OOO TypeError: Failed to fetch`가 기록되는 것을
작업지시자가 발견했다. GitHub Pages 배포 사이트에서도 같은 fetch가 404로 실패한다
(headless 실측 완료 — 콘솔 error 1건).

`WasmBridge.populateExternalImagesFromDevServer()`는 외부 연결 그림(HWP3 pic_type=0)의
basename을 `/samples/{name}`에서 fetch해 주입하는 **vite dev 서버 전용 경로**다
(vite `server.fs.allow`가 samples/ 를 서빙하는 것에 의존). 프로덕션 빌드(Pages)와
확장(chrome-extension://)에는 이 경로가 없어, 외부 연결 그림이 있는 문서를 열 때마다
문서당 그림 개수만큼 무의미한 네트워크 요청 + 실패 로그가 쌓인다.

## 방침

**dev 환경이 아니면 fetch를 시도하지 않는다.** 판별은 vite 표준 `import.meta.env.DEV`.

- dev 서버(`vite`/`vite --port 7700`): `DEV=true` → 기존 동작 유지
  (#3313 1차 정정의 주입 + `onExternalImagesInjected` 뷰 갱신 배선 그대로).
- 프로덕션 빌드(Pages `vite build`, 확장 `build.mjs`): `DEV=false` → 호출 자체 스킵.
  외부 연결 그림은 지금과 동일하게 placeholder 표시(회귀 아님 — 현재도 fetch가 전부
  실패해 placeholder로 귀결되며, 달라지는 것은 소음 제거뿐).

대안으로 "fetch는 시도하되 실패를 조용히 삼키기"를 검토했으나, 프로덕션에서 성공할 수
없는 요청을 문서마다 반복하는 것 자체가 낭비라 가드 방식을 택한다.

## 수정 범위

- `rhwp-studio/src/core/wasm-bridge.ts` — `populateExternalImagesFromDevServer()` 진입부
  1곳에 가드 추가 (호출부 유지, 함수 이름의 "DevServer" 의미와 일치).

## 비범위

- 프로덕션/확장의 사이드카 공급 UX(폴더 열기·다중 파일 드롭 → `inject_external_image`)
  — #3313 잔여 범위로 유지 (이 이슈는 #3313의 서브 이슈).
- SO-SUEOP.hwp 1쪽 이미지가 배포판에서 보이게 만드는 것 — 위 UX 설계 없이는 불가능,
  본 가드의 목표가 아니다.

## 검증

1. `tsc` + rhwp-studio 단위 테스트 전체.
2. dev 서버 headless e2e — SO-SUEOP.hwp 외부 그림 주입이 여전히 동작(유채색 픽셀 비율,
   기존 tmp-3313 체크 재사용).
3. 프로덕션 확인 — `vite build` 산출물(preview 서빙)에서 SO-SUEOP.hwp 로드 시
   `/samples/` fetch 요청·실패 로그 0건.
4. 확장 재빌드(`rhwp-chrome npm run build`) 후 dist 로드 확인은 작업지시자 수동 게이트.

## PR

- 브랜치 `task/3348-external-image-fetch-guard`, base `devel`, `Closes #3348`.
- 이 정정은 0.8.1 릴리즈 대상. PR 생성은 별도 승인 후 진행.

## 다음 단계

승인 시 Stage 2(구현계획서 — 가드 코드·검증 커맨드 확정) 후 구현.
