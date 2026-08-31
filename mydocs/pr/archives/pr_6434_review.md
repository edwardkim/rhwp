---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6434
author: kevin9327
---

# PR #6434 review - focusedPagePatch 전달

## Metadata

- 원 PR: [#6434](https://github.com/edwardkim/rhwp/pull/6434), source head
  `6fa7e481a0f0d2c71225d9732afc791607d939bf`.
- 작성자 `kevin9327`, external reviewer `jangster77` 요청 완료. CI green non-draft source를
  latest `upstream/devel` 위에 conflict 없이 적용했다.

## 변경과 검토

- Studio 본문의 국소 조판 결과인 `focusedPagePatch`를 renderer에 넘겨 전체 document 재조판을
  피한다.
- TypeScript nullable/numeric `charOffset`을 명시적으로 guard하여 `tsc`를 통과시킨다.
- Canvas render 내용이나 HWP/HWPX fixture를 변경하지 않아 visual sweep은 적용 대상이 아니다.

## 검증과 권고

- `rhwp-studio`에서 `node --test tests/local-text-replace-result.test.ts
  tests/cell-flow-boundary.test.ts`가 21/21 통과했고, `npx tsc --noEmit`도 통과했다.
- Rust 통합 branch full nextest는 `8772 passed, 43 skipped` (430.908초, exit 0)로
  완료했고, `fmt`, native/WASM/workspace all-target clippy, workspace build, manifest
  check도 보정 후 다시 통과했다.

**수용.** focused patch가 없는 결과로 fallback하지 않고 numeric offset guard까지 갖춘 범위로
수용한다.
