---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6091 review - Studio 쪽/폭 맞춤 배율과 눈금자 갱신

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6091
- 작성자: `planet6897`
- 원 PR head: `f4093ef90187`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9` (#6142 merge 포함)
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. Studio의 쪽 맞춤/폭 맞춤 배율 저장과 문서 전환 시 눈금자 재계산을 보정한다. renderer
엔진의 페이지 페인트 계약을 바꾸는 PR은 아니지만, 실제 브라우저 E2E로 문서 전환·배율 모드·눈금자 상태를
확인해야 하는 UI 변경이다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/studio-zoom-ruler-6090/before_stale_portrait_ruler.png`,
  `mydocs/report/studio-zoom-ruler-6090/after_landscape_ruler.png`
- Studio 검증:
  - `node --test tests/zoom-fit-mode-persistence.test.ts tests/user-settings.test.ts tests/shortcut-map.test.ts tests/toolbox-visibility.test.ts tests/render-backend.test.ts tests/zoom-fit.test.ts tests/ruler-document-load-refresh.test.ts`
    결과 100 tests pass
  - `npm run build` 통과
  - `npm run e2e:manifest-check` 통과
  - `node e2e/run-with-vite.mjs -- node e2e/ruler-document-switch.test.mjs --mode=headless` 통과
  - `node e2e/run-with-vite.mjs -- node e2e/zoom-fit-mode-persistence.test.mjs --mode=headless` 통과
- 통합 후보 전체 Rust/WASM/native-Skia 검증 통과

## 후속

통합 PR에서 CI가 완료되면 원 PR에는 Studio E2E 증거를 포함해 close한다.
