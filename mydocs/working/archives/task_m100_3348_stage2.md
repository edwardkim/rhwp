# Task #3348 Stage 2 — 구현계획서

## 수정 1곳 — `rhwp-studio/src/core/wasm-bridge.ts`

`populateExternalImagesFromDevServer()` 진입부(현재 `if (!this.doc) return;` 직후)에
가드 1줄 + 근거 주석:

```ts
private async populateExternalImagesFromDevServer(): Promise<void> {
  if (!this.doc) return;
  // [#3348] /samples/ fetch는 vite dev 서버 전용(server.fs.allow). 프로덕션 빌드
  // (Pages·확장)에는 경로가 없어 실패 로그만 쌓이므로 dev 외에는 시도하지 않는다.
  // 프로덕션 사이드카 공급 UX는 #3313 잔여 범위.
  if (!import.meta.env.DEV) return;
  ...
```

- 호출부(`loadDocument` 내 `void this.populateExternalImagesFromDevServer();`)는 무변경.
- `import.meta.env.DEV`는 vite 정적 치환 — 프로덕션 번들에서는 `if (true) return;` 꼴로
  상수 접힘되어 dead code 제거까지 기대 가능. wasm-bridge.ts 는 이미 vite 전용 모듈
  경로라 타입(`vite/client`) 문제 없음(주변 코드가 동일 관용구 사용, tsc 로 확정).

## 검증 절차 (순서대로)

1. `cd rhwp-studio && npx tsc --noEmit` — 타입 게이트.
2. `npm test` — 단위 테스트 전체(변경이 순수 가드라 회귀 없음 확인).
3. **dev 보존 실측**: dev 서버 기동 후 기존 `tmp-3313-sosueop-image.check.mjs`
   (headless) — 주입 후 유채색 픽셀 비율이 여전히 상승(≈18%)하는지.
4. **프로덕션 제거 실측**: `npx vite build` + `npx vite preview` 로 산출물 서빙,
   headless로 SO-SUEOP.hwp 업로드 → 네트워크에서 `/samples/` 요청 0건·해당 콘솔
   경고 0건 확인(요청 감시는 puppeteer request 이벤트).
5. 확장 재빌드(`rhwp-chrome npm run build`) 후 dist 로드 오류 소멸 확인 —
   작업지시자 수동 게이트.

## 커밋·PR

- 커밋 1개: `fix(studio): 외부 연결 그림 dev 전용 fetch를 프로덕션·확장에서 가드 (#3348)`
- working 문서(stage1/2) 동일 브랜치 커밋(보고서는 report/ 에 최종 1건).
- PR: base devel, `Closes #3348`, 본문 한국어 — 생성은 별도 승인 후.
