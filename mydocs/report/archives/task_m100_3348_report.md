# Task #3348 — 외부 연결 그림 dev 전용 fetch 가드 (최종 보고서)

- Issue: [#3348](https://github.com/edwardkim/rhwp/issues/3348) (#3313 서브 이슈)
- Branch: `task/3348-external-image-fetch-guard`
- 계획서: `mydocs/working/task_m100_3348_stage1.md` / `_stage2.md`

## 배경

v0.8.0 배포 점검(2026-07-26)에서 크롬 확장 Errors 패널에
`[WasmBridge] 외부 image ...: 00000000.OOO TypeError: Failed to fetch`
(`populateExternalImagesFromDevServer`)가 기록됨을 작업지시자가 발견했다. GitHub Pages
배포 사이트에서도 동일 fetch가 404로 실패한다(headless 실측, 콘솔 error 1건).

원인: 외부 연결 그림(HWP3 pic_type=0)의 `/samples/{basename}` fetch는 vite dev 서버
전용 경로(`server.fs.allow`)인데, 프로덕션 빌드(Pages)·확장(chrome-extension://)에서도
무조건 시도되어 문서를 열 때마다 실패 요청과 로그가 쌓였다.

## 수정

`rhwp-studio/src/core/wasm-bridge.ts` — `populateExternalImagesFromDevServer()` 진입부에
`if (!import.meta.env.DEV) return;` 가드 1줄(+근거 주석). dev 서버의 기존 동작
(#3313 1차 정정의 주입 + 뷰 갱신 배선)은 무변경.

## 검증 결과

| 단계 | 결과 |
|---|---|
| `tsc --noEmit` | 통과 |
| studio 단위 테스트 | 637 pass / 0 fail |
| dev 보존 (headless, `tmp-3313-sosueop-image.check.mjs`) | 주입 1건 발생·1쪽 이미지 시각 표시 확인. 스크립트의 "유채색 비율" FAIL은 대상 이미지가 흑백(붓글씨)이라 채도 휴리스틱이 0%를 내는 검사 자체의 한계 — 가드 유무 A/B 재실행으로 본 수정과 무관한 사전 존재 현상임을 실증 |
| 프로덕션 제거 (vite build + preview, headless) | `/samples/` 네트워크 요청 0건·`외부 image` 콘솔 로그 0건·문서 로드 정상. 대조군(가드 없는 현 배포 사이트)은 동일 절차에서 404 error 1건 |
| 확장 dist 로드 | 작업지시자 수동 게이트 (재빌드 제공) |

## 비범위 (유지)

프로덕션/확장의 사이드카 공급 UX(폴더 열기·다중 파일 드롭 → `inject_external_image`)는
부모 이슈 #3313 잔여 범위로 남는다. SO-SUEOP.hwp 1쪽 이미지가 배포판에서 보이게 하는
것은 그 설계 없이는 불가하며 본 가드의 목표가 아니다.

## 릴리즈

0.8.1 PATCH 대상.
