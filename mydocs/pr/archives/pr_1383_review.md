# PR #1383 검토 — 인쇄 시 혼합 용지 크기 보존 (fix studio mixed page sizes)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1383
- 작성자: `msjang` (Minseok Jang) — **rhwp 첫 PR**
- 상태: open / draft 아님
- base: `main` (head: `fix/mixed-page-print-sizes`, `7f83124c`)
- 연결 이슈: 없음 (`closes #` 비어있음)
- 변경: 3파일 +207/-50 (rhwp-studio 인쇄 경로, TypeScript)

## 2. 문제와 변경 요약

### 문제

rhwp-studio 인쇄는 **첫 페이지 크기만**으로 `@page { size }` 를 단일 지정한다
(`file.ts`: `getPageInfo(0)` → `widthMm/heightMm` 1쌍). 용지 방향이 섞인 문서
(세로+가로 혼합)를 인쇄하면 모든 페이지가 첫 페이지 크기로 강제돼 잘림/여백 오류 발생.
(첨부 before/after PDF + hoffice-mixed-ori.hwpx 샘플로 실증.)

현재 `devel` 코드에서 결함 확인: `file.ts:189 appendPrintStyle(doc, widthMm, heightMm)`
+ `getPageInfo(0)` 첫 페이지 고정.

### 변경 (3묶음)

1. **페이지별 named `@page`** (`print-pages.ts` 신규)
   - 각 페이지마다 `@page rhwp-print-page-N { size: WmmHmm }` + `.rhwp-print-page-N
     { page: rhwp-print-page-N; width/height }` 로 페이지별 크기 보존.
   - `getPageInfo(0)` → `getPageInfo(i)` 페이지별 호출로 교정.
2. **SVG id 네임스페이스 격리** (`namespaceSvgIds`/`namespaceSvgReferenceValue`)
   - 여러 SVG 를 한 인쇄 문서에 합칠 때 id 충돌(`url(#clip)`, gradient 등) 방지 —
     페이지별 prefix. (단일 `@page` 시절엔 없던, 페이지별 분리로 새로 필요해진 처리.)
3. **로직 분리 + 단위 테스트**
   - `file.ts` 인라인 함수(appendPrintStyle/appendSvgPage)를 `print-pages.ts` 모듈로 추출.
   - `tests/print-pages.test.ts` node:test 4건 (px→mm 변환, named page 생성, 혼합 방향
     @page 보존, SVG url/hash 참조 치환).

## 3. 검증 (로컬, `pr1383-review` = local/devel + cherry-pick `7f83124c`)

- cherry-pick: 충돌 없음 (devel 정합).
- `node --test tests/print-pages.test.ts`: **4/4 pass**.
- `npx tsc --noEmit`: **exit 0** (타입 오류 없음).
- `node --test tests/*.test.ts` 전체: **74/74 pass** (회귀 없음).

(rhwp-studio 인쇄는 브라우저 `window.open` + WASM 경로라 헤드리스 자동 검증은 단위
테스트 범위. 시각 인쇄 결과는 PR 첨부 before/after PDF 가 실증.)

## 4. 평가

### 장점

- 진단 정확 — 현재 devel 코드에서 첫 페이지 크기 고정 결함 확인.
- 해결 견고 — named `@page` per-page 는 혼합 용지 인쇄의 표준 접근. SVG id 격리는
  실제로 필요한 처리(여러 SVG 합칠 때 url(#id) 충돌). `break-after`/`page-break-after`
  신구 병기, 소수 mm 포맷 정리 등 디테일 양호.
- 모듈 분리 + 단위 테스트 4건 동반 — 회귀 봉인.
- TS/test 통과, 코드 품질 일관.

### 검토 포인트 (블로커 아님)

- **base 가 `main`** — 본 저장소는 외부 기여를 `devel` 로 받는다(CLAUDE.md 컨트리뷰터
  워크플로우). merge 시 devel 기준으로 처리 필요(cherry-pick 또는 base 변경). 단순 절차.
- **연결 이슈 없음** — 인쇄 혼합 용지 결함에 대응하는 open issue 부재. merge 시 별도
  이슈 없이 수용하거나, 결함 추적용 이슈를 사후 등록.
- 첫 PR 기여자 — 환영 + (필요 시) base/동기화 안내.

## 5. 판단

**merge 권고** (devel 기준). 결함이 실재하고, 해결이 표준적이며, 테스트·타입체크 통과.
base=main → devel 처리와 첫 기여자 안내만 동반하면 된다.

세부 merge 절차·코멘트는 `pr_1383_report.md` 에서 확정.
