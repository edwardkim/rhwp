---
kind: pr-review
status: local-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4637 검토 - 개발용 Subsecond 런타임의 production bundle 격리

## 판정

로컬 수용. 개발 전용 런타임은 `import.meta.env.DEV` 아래 동적 import로 제한하고, production bundle은
벤더 이름과 hot-patch marker를 포함하지 않도록 한다. 원 PR은 최신 `devel`과 충돌하지만 누적 검토
브랜치에서 현재 WASM memory typing과 도메인 이름 변경을 함께 보존해 해소했다.

## 검토 기준

- 원격 head: `0d1b7d929e8ec38a549a58e81d5482a6226afd96`
- 로컬 누적 검토 브랜치: `review/humdrum00001010-20260812`
- 적용 순서: #4621 다음에 #4637의 13개 commit을 적용했다.
- 충돌 해소: `wasm-bridge.ts`의 `init()` 결과 `memory?: WebAssembly.Memory` 계약을 유지하면서
  `startRenderCodeReload()` 도메인 경로를 적용했다.

## 확인

- `npm --prefix rhwp-studio test`: 870 passed.
- `npm --prefix rhwp-studio run build`: TypeScript와 Vite production build 통과.
- `node --test scripts/frontend-studio-dist.test.mjs scripts/frontend-wasm-bindings.test.mjs`: 5 passed.
- `wasm-pack build --target web --out-dir pkg` 뒤 생성 바인딩 계약 검증: 1 passed.

## 경계 확인

최종 누적 diff에는 `.github/workflows` 변경이 없다. production bundle 검사에서 Subsecond/Dioxus vendor
이름과 개발 marker가 모두 부재하며, dev runtime만 동적 경로에서 이를 보유한다. renderer output과
문서 format은 변경하지 않는다.
