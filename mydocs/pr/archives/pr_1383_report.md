# PR #1383 최종 보고서 — 인쇄 시 혼합 용지 크기 보존 (msjang 첫 PR)

## 1. 결정

**merge 수용** — `local/devel` 에 PR 커밋 cherry-pick → `devel` push 방식.

## 2. 변경 본질 (검토 확정)

rhwp-studio 인쇄가 첫 페이지 크기만으로 `@page { size }` 를 단일 지정 → 용지 방향 혼합
문서 인쇄 시 모든 페이지가 첫 페이지 크기로 강제되는 버그 수정.

- 페이지별 named `@page rhwp-print-page-N` + `getPageInfo(0)`→`getPageInfo(i)`.
- 여러 SVG 합칠 때 id 충돌 방지(페이지별 네임스페이스 — `url(#id)`/hash 참조 치환).
- `print-pages.ts` 모듈 분리 + node:test 단위 테스트 4건.

3파일 +207/-50 (file.ts, print-pages.ts 신규, print-pages.test.ts 신규).

## 3. 검증

- cherry-pick `7f83124c` → local/devel: 충돌 없음 (author `msjang` 보존).
- `npx tsc --noEmit`: exit 0.
- `node --test tests/print-pages.test.ts`: 4/4 pass.
- `node --test tests/*.test.ts` 전체: 74/74 pass (회귀 없음).
- 시각 인쇄 결과: PR 첨부 before/after PDF + hoffice-mixed-ori.hwpx 샘플로 실증.

## 4. merge 방식 — cherry-pick 선택 이유

PR base 는 처음 `main` 이었다. 본 저장소는 외부 기여를 `devel` 로 받으므로 base 를
`devel` 로 변경(API PATCH)했으나, PR 브랜치가 **오래된 base 위**(BEHIND)라 GitHub
3-way merge 가 무관한 변경(`task_m100_1310_stage{7,8,9}.md` 파일 끝 빈 줄 3건)을 되돌리는
부작용이 있었다. 의미 없는 trailing blank line 이나 무관 변경을 섞지 않기 위해, 검증된
PR 커밋(`7f83124c`)만 `local/devel` 에 cherry-pick(무관 변경 0, author 보존)하고 devel
push 하는 방식을 택했다. PR 은 "devel 에 포함됨" 으로 close.

## 5. 후속

- 첫 기여자(msjang) 환영 + cherry-pick merge 안내 코멘트 후 close.
- 연결 이슈 부재 — 별도 결함 이슈 없이 수용(인쇄 혼합 용지 버그는 본 PR 로 종결).
